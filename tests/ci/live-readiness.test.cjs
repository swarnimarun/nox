const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync, existsSync } = require('node:fs');
const { spawn, execFileSync } = require('node:child_process');
const source = readFileSync('nix/targets/iso.nix', 'utf8');
const template = source.match(/pgrep --full "([^"]+)" >\/dev\/null/)[1];

async function runProcess(name, check) {
  // A Nix wrapper can preserve argv[0] while the kernel comm is the real binary.
  const child = spawn('bash', ['-c', 'exec -a "$1" sleep 30', 'fixture', `/nix/store/fixture/bin/${name}`]);
  try {
    for (let attempt = 0; attempt < 100; attempt++) {
      if (readFileSync(`/proc/${child.pid}/cmdline`, 'utf8').startsWith(`/nix/store/fixture/bin/${name}\0`)) break;
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    assert.equal(readFileSync(`/proc/${child.pid}/comm`, 'utf8').trim(), 'sleep');
    await check(child.pid);
  } finally {
    child.kill();
    await new Promise((resolve) => child.once('exit', resolve));
  }
}

function matches(compositor, pid) {
  try {
    return execFileSync('pgrep', ['--full', template.replace('$compositor', compositor)], { encoding: 'utf8' }).trim().split('\n').includes(String(pid));
  } catch (error) {
    if (error.status === 1) return false;
    throw error;
  }
}

for (const compositor of ['Hyprland', 'niri']) {
  test(`readiness finds wrapped ${compositor} despite a different comm name`, { skip: !existsSync('/proc') }, async () => {
    await runProcess(compositor, (pid) => assert.equal(matches(compositor, pid), true));
  });
}
for (const [command, compositor] of [['start-hyprland', 'Hyprland'], ['niri-session', 'niri'], ['Hyprland-other', 'Hyprland']]) {
  test(`readiness rejects launcher or different executable ${command}`, { skip: !existsSync('/proc') }, async () => {
    await runProcess(command, (pid) => assert.equal(matches(compositor, pid), false));
  });
}

// The sandbox lacks /proc; verify the captured command-line contract here too.
for (const [command, compositor, expected] of [
  ['Hyprland', 'Hyprland', true],
  ['niri', 'niri', true],
  ['start-hyprland', 'Hyprland', false],
  ['niri-session', 'niri', false],
  ['Hyprland-other', 'Hyprland', false],
]) {
  test(`executable boundary ${command} for ${compositor}`, () => {
    const expression = new RegExp(template.replace('$compositor', compositor).replace('[[:space:]]', '\\s'));
    assert.equal(expression.test(`/nix/store/fixture/bin/${command} --config /etc/nox/config`), expected);
  });
}
