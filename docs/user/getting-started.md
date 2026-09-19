# Build and test Nox

Use a Linux builder with Nix, flakes enabled, ample free disk space and network
access to upstream inputs/caches. x86_64-linux is the initial test host.
ARM outputs need an ARM builder or correctly configured emulation.

## Bootstrap the CLI

From this checkout:

```sh
bash scripts/bootstrap.sh
./result/bin/noxctl --help
```

Bootstrap explicitly resolves lockfiles. Review and commit both locks before
sharing a reproducible configuration. Install the built CLI if desired:

```sh
nix profile install .#noxctl
```

The source repository is private: remote flake access needs GitHub
credentials. For local development use the absolute checkout reference below.

## Build the supplied examples

```sh
nix build --no-update-lock-file .#qcow2 --out-link result-vm
nix build --no-update-lock-file .#iso --out-link result-iso
nix build --no-update-lock-file .#wsl --out-link result-wsl
nix build --no-update-lock-file .#oci --out-link result-oci
```

| Output | Next step |
|---|---|
| `result-vm/*.qcow2` | Attach to an EFI VM with virtio storage and a serial console |
| `result-iso/iso/*.iso` | Boot as removable installer media in a disposable VM |
| `result-wsl/bin/nixos-wsl-tarball-builder` | Run as root to produce `nox.wsl`, then import on Windows |
| `result-oci` | Load archive into Podman/Docker and start a workspace |

The example VM has local console user `nox`, password `nox`. SSH passwords
are disabled. This access module is only for disposable local testing.

For an ephemeral x86 VM (supply your OVMF firmware path):

```sh
bash scripts/run-vm.sh result-vm/nixos.qcow2 /path/to/OVMF_CODE.fd
```

The runner uses a snapshot and does not persist guest disk writes. To test
installation persistence, use a separate disposable guest with its own disk.

## Create a machine project

```sh
noxctl init /tmp/nox-lab --profile server --target qcow2 --source "path:$PWD"
noxctl validate --config /tmp/nox-lab/nox.toml
noxctl plan --config /tmp/nox-lab/nox.toml
noxctl lock --config /tmp/nox-lab/nox.toml
noxctl image build --config /tmp/nox-lab/nox.toml --dry-run
noxctl image build --config /tmp/nox-lab/nox.toml --out-link result-lab
```

`init` requires an empty directory. Before building your own VM, add an access
module defining a user/password or public SSH key and list it in
`[nix].extra_modules`. Generated systems have no universal default password.
Add hardware modules through the same explicit list. No command rewrites
existing user-owned `.nix` files.

For WSL/OCI use `--profile workspace --target wsl` or `--target oci`.
Keep project paths free of `#` and `?`. Review code imported via extra modules:
Nix modules are executable configuration and can override system settings.

## WSL

On a Linux Nix builder:

```sh
sudo ./result-wsl/bin/nixos-wsl-tarball-builder ./nox.wsl
```

Copy `nox.wsl` to Windows with WSL2 installed. In PowerShell:

```powershell
./scripts/import-wsl.ps1 -Image .\nox.wsl -InstallLocation C:\WSL\Nox -Name Nox
wsl -d Nox
```

Inside Nox, verify `whoami`, `systemctl is-system-running`, and `noxctl --help`.
A built WSL tarball builder is not itself an importable root filesystem.
See [upstream WSL build instructions](https://github.com/nix-community/NixOS-WSL/blob/main/docs/src/building.md).

## Workspace on an existing distro

```sh
bash scripts/run-workspace.sh result-oci
```

This is an ephemeral rootless Podman userspace workspace. It shares the host
kernel and does not boot NixOS systemd or change the host distribution. No
home directory or host device is mounted. Direct Distrobox integration and
persistent Nix package management inside this image still need runtime tests.
For a full NixOS environment use the VM target.

## Install a disposable VM or physical test machine

1. Create a `server`/`metal` project using `noxctl init`.
2. Copy and review `examples/metal/{disk,hardware,access}.nix`; reference them
   in `nox.toml`. Replace the placeholder disk ID and template assertions with
   the destination's real hardware configuration and your public SSH key.
3. Lock the project and inspect the destination with `lsblk` over SSH. This
   installer supports one Disko disk and EFI boot only in the supplied preset.
4. First run the upstream installer with `--vm-test`:

```sh
nix run path:/tmp/nox-metal#installer -- --flake path:/tmp/nox-metal#nox --vm-test
```

5. Review Nox's plan, then explicitly execute only against your test machine:

```sh
noxctl install --config /tmp/nox-metal/nox.toml --host root@192.0.2.10
noxctl install --config /tmp/nox-metal/nox.toml --host root@192.0.2.10 \
  --execute --confirm-host root@192.0.2.10 \
  --confirm-disk /dev/disk/by-id/YOUR-TEST-DISK
```

The destination OS and disk contents are replaced. After reboot verify SSH,
`findmnt /`, `noxctl --help`, and generation inspection. Execution uses an immutable snapshot of the locked project. Keep the machine flake/locks under version
control for subsequent management.

`apply` and `rollback` still fail explicitly; their health-check and timed
recovery contract must be implemented and tested before host activation is
exposed. See the [implementation plan](../operations/implementation-plan.md).
