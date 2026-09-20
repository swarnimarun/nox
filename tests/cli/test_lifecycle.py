"""Process-boundary tests; Nix is a fake executable, never the real installer."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

BINARY = Path(os.environ.get('NOXCTL', 'target/debug/noxctl')).resolve()


class Lifecycle(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.project = self.root / 'machine'
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.log = self.root / 'calls.jsonl'
        fake = self.bin / 'nix'
        fake.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['CALL_LOG'], 'a') as f: f.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:3] == ['flake', 'archive']: print('{\"path\":\"/nix/store/frozen-nox\"}')
if sys.argv[1] == 'eval': print(os.environ.get('DISKS', '{"main":{"device":"/dev/disk/by-id/virtio-test"}}'))
if sys.argv[1] == os.environ.get('FAIL_COMMAND'): sys.exit(23)
''')
        fake.chmod(0o755)
        helper = '''#!/usr/bin/env python3
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
with open(os.environ['CALL_LOG'], 'a') as f: f.write(json.dumps([name, *sys.argv[1:]]) + '\\n')
if name == 'nixos-enter': sys.stdin.read()
'''
        for name in ['cp', 'nixos-install', 'nixos-enter', 'nox-installer']:
            path = self.bin / name
            path.write_text(helper)
            path.chmod(0o755)
        self.install_root = self.root / 'target-root'
        self.env = dict(os.environ, PATH=f'{self.bin}:{os.environ["PATH"]}',
                        CALL_LOG=str(self.log), NOX_TEST_INSTALL_ROOT=str(self.install_root))

    def tearDown(self):
        self.temp.cleanup()

    def run_cli(self, *args, ok=True, input_text=None):
        r = subprocess.run([str(BINARY), *map(str, args)], env=self.env, text=True,
                           input=input_text, capture_output=True)
        self.assertEqual(r.returncode == 0, ok, r.stdout + r.stderr)
        return r

    def init(self, target='metal', profile='server', *extra):
        self.run_cli('init', self.project, '--target', target, '--profile', profile, *extra)
        (self.project / 'flake.lock').write_text('{}')
        return self.project / 'nox.toml'

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def test_workspace_does_not_enable_system_services(self):
        import tomllib
        config = self.init('oci', 'workspace')
        self.assertEqual(tomllib.loads(config.read_text())['capabilities'], ['development'])

    def test_init_records_niri_hardware_and_install_settings(self):
        import tomllib
        config = self.init('metal', 'gaming', '--flavour', 'niri', '--graphics', 'amd',
                           '--bootloader', 'grub-efi', '--filesystem', 'ext4',
                           '--disk', '/dev/disk/by-id/virtio-test', '--username', 'player')
        parsed = tomllib.loads(config.read_text())
        self.assertEqual(parsed['desktop']['flavour'], 'niri')
        self.assertEqual(parsed['hardware']['graphics'], 'amd')
        self.assertEqual(parsed['boot']['loader'], 'grub-efi')
        self.assertEqual(parsed['install']['filesystem'], 'ext4')
        self.assertEqual(parsed['install']['disk'], '/dev/disk/by-id/virtio-test')
        self.assertEqual(parsed['user']['name'], 'player')

    def test_archive_failure_never_evaluates_or_installs(self):
        config = self.init()
        self.env['FAIL_COMMAND'] = 'flake'
        self.execute(config, False)
        self.assertEqual([x[0] for x in self.calls()], ['flake'])

    def test_init_never_overwrites(self):
        self.init()
        before = (self.project / 'nox.toml').read_bytes()
        self.run_cli('init', self.project, ok=False)
        self.assertEqual(before, (self.project / 'nox.toml').read_bytes())

    def test_invalid_init_writes_nothing(self):
        self.run_cli('init', self.project, '--target', 'wsl', ok=False)
        self.assertFalse(self.project.exists())

    def test_build_dry_run_has_no_process_effect(self):
        config = self.init('qcow2')
        self.run_cli('build', '--config', config, '--dry-run')
        self.assertEqual(self.calls(), [])

    def test_build_passes_path_as_one_argument(self):
        config = self.init('qcow2')
        self.run_cli('image', 'build', '--config', config, '--out-link', self.root / 'output with spaces')
        self.assertEqual(self.calls()[0][-1], str(self.root / 'output with spaces'))
        self.assertIn('--no-update-lock-file', self.calls()[0])

    def test_missing_lock_stops_build(self):
        config = self.init()
        (self.project / 'flake.lock').unlink()
        self.run_cli('build', '--config', config, ok=False)
        self.assertEqual(self.calls(), [])

    def test_install_default_only_plans(self):
        config = self.init()
        self.run_cli('install', '--config', config, '--host', 'root@test')
        self.assertEqual(self.calls(), [])

    def test_install_confirmation_before_process(self):
        config = self.init()
        self.run_cli('install', '--config', config, '--host', 'root@test', '--execute', ok=False)
        self.assertEqual(self.calls(), [])

    def test_install_rejects_shell_destination(self):
        config = self.init()
        self.run_cli('install', '--config', config, '--host', 'root@test;id', ok=False)
        self.assertEqual(self.calls(), [])

    def execute(self, config, ok):
        return self.run_cli('install', '--config', config, '--host', 'root@test', '--execute',
                            '--confirm-host', 'root@test',
                            '--confirm-disk', '/dev/disk/by-id/virtio-test', ok=ok)

    def test_install_rejects_wrong_disk(self):
        config = self.init()
        self.env['DISKS'] = '{"main":{"device":"/dev/disk/by-id/virtio-other"}}'
        self.execute(config, False)
        self.assertEqual([x[0] for x in self.calls()], ['flake', 'eval'])

    def test_install_rejects_multiple_disks(self):
        config = self.init()
        self.env['DISKS'] = ('{"a":{"device":"/dev/disk/by-id/virtio-test"},'
                             '"b":{"device":"/dev/disk/by-id/virtio-other"}}')
        self.execute(config, False)
        self.assertEqual([x[0] for x in self.calls()], ['flake', 'eval'])

    def test_failed_build_never_runs_installer(self):
        config = self.init()
        self.env['FAIL_COMMAND'] = 'build'
        self.execute(config, False)
        self.assertEqual([x[0] for x in self.calls()], ['flake', 'eval', 'build'])

    def test_install_evaluates_then_builds_then_runs(self):
        config = self.init()
        self.execute(config, True)
        self.assertEqual([x[0] for x in self.calls()], ['flake', 'eval', 'build', 'run'])
        self.assertIn('path:/nix/store/frozen-nox#nox', self.calls()[-1])

    def test_non_metal_never_installs(self):
        config = self.init('wsl', 'workspace')
        self.execute(config, False)
        self.assertEqual(self.calls(), [])

    def test_installer_gui_forwards_dry_run(self):
        self.run_cli('installer', 'gui', '--dry-run')
        self.assertEqual(self.calls(), [['nox-installer', '--dry-run']])

    def test_local_installer_only_plans_without_execute(self):
        config = self.init('metal', 'gaming', '--disk', '/dev/disk/by-id/virtio-test')
        result = self.run_cli('installer', 'local', '--config', config)
        self.assertIn('DESTRUCTIVE', result.stdout)
        self.assertEqual(self.calls(), [])

    def test_local_installer_rejects_confirmation_before_process(self):
        config = self.init('metal', 'gaming', '--disk', '/dev/disk/by-id/virtio-test')
        self.run_cli('installer', 'local', '--config', config, '--execute',
                     '--confirm-disk', '/dev/disk/by-id/virtio-other', ok=False,
                     input_text='secret-password\n')
        self.assertEqual(self.calls(), [])

    def test_local_installer_builds_before_disk_mutation(self):
        config = self.init('metal', 'gaming', '--disk', '/dev/disk/by-id/virtio-test')
        self.env['FAIL_COMMAND'] = 'build'
        self.run_cli('installer', 'local', '--config', config, '--execute',
                     '--confirm-disk', '/dev/disk/by-id/virtio-test', ok=False,
                     input_text='secret-password\n')
        self.assertEqual([call[0] for call in self.calls()], ['flake', 'eval', 'build'])

    def test_local_installer_runs_guarded_sequence(self):
        config = self.init('metal', 'gaming', '--disk', '/dev/disk/by-id/virtio-test')
        self.run_cli('installer', 'local', '--config', config, '--execute',
                     '--confirm-disk', '/dev/disk/by-id/virtio-test',
                     input_text='secret-password\n')
        self.assertEqual([call[0] for call in self.calls()],
                         ['flake', 'eval', 'build', 'run', 'cp', 'nixos-install', 'nixos-enter'])
        self.assertIn('--yes-wipe-all-disks', self.calls()[3])
        self.assertTrue((self.install_root / 'etc/nox').is_dir())


if __name__ == '__main__':
    unittest.main()
