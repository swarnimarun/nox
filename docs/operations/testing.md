# Testing and validation

## Fast contract checks

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo build -p noxctl --locked
python3 -m unittest discover -s tests/cli -v
cargo clippy --workspace --all-targets --locked -- -D warnings
nix fmt .
nix flake check --no-update-lock-file --show-trace
```

The CLI tests substitute a fake `nix` executable and verify arguments, failure
propagation, no-overwrite behavior, dry-run behavior and installer interlocks.
They never invoke a real disk operation. This does not establish that the real
installer can boot a destination.

The Nix check forces each artifact derivation, rather than just stateVersion.
The server VM test boots NixOS, waits for multi-user and SSH, and checks Git and
Podman. CI also creates a real generated Niri/qcow2 project, attaches the
generated Home Manager dotfiles flake, locks it, evaluates its toplevel and
Home Manager user, and runs the non-mutating upgrade preview.

The release workflow is the exact-image gate. It starts after successful
`main` CI, removes stale Actions artifacts, and builds Hyprland, Niri, WSL, and OCI
container artifacts serially. Each ISO boots with QEMU/TCG and UEFI; its marker is emitted only after
NetworkManager, greetd, the expected compositor, and `nox-installer` run.

## Target release evidence

| Target | Required runtime evidence |
|---|---|
| Hyprland/Niri ISO | Exact ISO boots with UEFI; NetworkManager, greetd, selected compositor, and GTK installer run |
| qcow2 | Exact image boots, local login works, filesystem persists on a non-snapshot test |
| WSL | Build tarball, Windows WSL2 import, default user, systemd and noxctl |
| OCI | Rootless Podman load/run; shell, Git and noxctl; no host filesystem changes |
| metal | Disposable disk install via SSH, reboot and SSH reachability, correct root/ESP |
| desktop/gaming | Display-manager login and hardware/graphics tests on target devices |

Record commit SHA, flake.lock, Cargo.lock, architecture, image SHA256, command,
exit status and console logs. A failed or skipped test never counts as support.
The workflow may also be dispatched for `all`, one compositor, WSL, or the container. A full
run deletes other prereleases and publishes image, SHA-256, and provenance files
to `v0.2.0-alpha.1`. The OCI image is loaded and run with Docker in a network-isolated container without host mounts. Heavyweight jobs use `max-parallel: 1`. WSL packaging runs
as root in CI; Windows WSL2 launch validation remains external.

## Before enabling apply or rollback

Implement and test: prior generation capture; diff before activation; rollback
scheduled independently of the CLI/SSH process; temporary activation; timeout,
health failure and interrupted-session recovery; explicit confirmation; boot
selection persistence; locking against simultaneous operations. Include
firewall/network changes and reboot failure cases. Never infer recovery from a
mocked successful command.
