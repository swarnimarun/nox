# Implementation and acceptance plan

## Goal and release boundary

Produce reproducible Nox systems from one versioned TOML model, with a usable
CLI, explicit installation, and independently verified target artifacts.
An evaluation, successful derivation build, and successful boot are three
separate gates. A target is supported only after all relevant gates pass.

The base is NixOS 26.05. Profiles remain separate from capabilities and
runtime targets. Nix owns desired state; user-written Nix modules are never
rewritten by the CLI. The release must commit Cargo.lock and flake.lock.

## Component map

| Component | Implementation | Acceptance gate |
|---|---|---|
| Schema | Rust enums, strict fields, hostname/channel/module validation | Rust schema tests and matching Nix rejection tests |
| Planning | Pure nox-core plan | Stable ordering and target-specific actions |
| Composition | nox.lib.mkSystem reads TOML and imports modules | Full toplevel evaluation, not merely stateVersion |
| CLI setup | init creates TOML and a new machine flake; lock resolves inputs | Existing files preserved; invalid combinations write nothing |
| CLI build | build/image build invokes locked Nix image output | Exit propagation, argv safety, no activation |
| CLI packaging | buildRustPackage, flake app, included in system packages | Cargo tests, CLI executable in every image |
| Profiles | server, desktop, gaming, workspace, recovery | Each profile evaluates; representative VM boots |
| Capabilities | development, apps, virtualization, desktop/gaming, recovery; storage/share tools | Feature-specific runtime tests before stable status |
| ISO | upstream minimal NixOS installer composition | Build ISO, boot under EFI, use recovery tools |
| qcow2 | upstream make-disk-image, EFI/ext4 | Build and boot exact produced disk image |
| WSL | NixOS-WSL module and tarballBuilder | Build tarball as root, import on Windows, launch systemd and CLI |
| OCI workspace | dockerTools userspace image | podman/docker load and shell/CLI test; no kernel or systemd claim |
| Metal installer | Disko + nixos-anywhere, one explicit disk | Disposable guest installation, reboot, SSH login, disk mismatch refusal |
| Generations | Read-only local system profile listing | Test on running NixOS |
| Apply/rollback | Still reserved | Temporary activation, failure injection, reboot recovery and confirmation timeout |
| Daemon/API/UI | Deferred by AGENTS.md | Only after lifecycle contracts pass; observed state never edits desired state |

Storage/shares currently install operator tools; they do not create pools,
exports, credentials or firewall rules. OCI provides a workspace, not a full
NixOS container. Direct Distrobox compatibility is unverified and is not a
release claim. Prefer the supplied rootless Podman workspace for the first
host-distro test.

## Ordered work packages

1. **Build contracts:** strict schema parity, committed lockfiles, CLI package,
   TOML bridge, exact target outputs and process-boundary tests.
2. **Boot evidence:** run server NixOS test, then exact ISO/qcow2 boot tests;
   record architecture, locked revision, artifact hash and console log.
3. **WSL and workspaces:** Windows import test, rootless Podman execution,
   persistence and UID mapping contract, then investigate Distrobox integration.
4. **Installation:** review hardware and SSH modules; run nixos-anywhere's
   `--vm-test`; execute on a disposable guest; verify reboot and access. Add
   multi-disk support only with a complete confirmation manifest.
5. **Activation/recovery:** evaluate/build/diff; schedule rollback before
   temporary activation; health checks; explicit confirmation; persistent boot
   selection only after confirmation. Test crashes, timeouts, reboot and loss of
   SSH. Delegate mechanisms upstream where practical.
6. **Profile depth:** desktop login, graphics, gaming drivers, storage presets,
   share configuration, backups, Home Manager, Incus and secrets integration.
7. **Runtime control:** noxd Unix-socket API and observed-state database;
   authorization and operation log; CLI client; then web UI. No clustering or
   marketplace in the initial release.

## Current evidence and limitations

This implementation was authored in an environment without Nix, Rust, QEMU,
Podman or WSL. Local checks cover shell/Python syntax, TOML parsing and diff
hygiene. GitHub CI supplies compilation, contract tests and Nix evaluation/VM
checks. CI status and uploaded artifacts are the authority for what actually
ran. Windows WSL, real hardware, graphical sessions, and exact disk-image
boot must be recorded independently before claiming support.

## Installer safety boundary

`install` prints a plan by default. `--execute`, matching `--confirm-host`, and
matching `--confirm-disk` are mandatory. Nox evaluates the Disko disk set,
rejects multiple disks, builds the full system, and only then invokes the
locked nixos-anywhere package. SSH host authentication remains upstream's
responsibility. Execution archives the locked machine flake into the immutable Nix store before checking disks and uses that same snapshot for the build and installer.
The destination disk is erased; there is no automatic restoration of its old
contents. The confirmation does not prove that an operator selected the correct
physical disk: inspect its stable ID on the destination first.
