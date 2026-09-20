# Nox implementation handover

Updated: 2026-09-20  
Intended recipient: Sol Max  
Repository: `swarnimarun/nox`

## Executive status

The canonical `main` branch is still at `0840104dc791f10828c37a00017e48fea6427ba7` (`style: normalize Rust and Nix formatting`). The Hyprland/Niri flavour, gaming defaults, GTK installer, local Disko installation path, and one-image release workflow described in the previous handoff were developed only in an earlier scratch workspace. They were not committed or pushed and are not present in this checkout.

Do not interpret the passing checks below as validation of that missing implementation. They validate only the recoverable `0840104` baseline.

## Verified repository state

- `origin/main` and local `main` both resolve to `0840104dc791f10828c37a00017e48fea6427ba7`.
- The checkout initially had no tracked or untracked product changes.
- `Handover.md` is the only product-facing file added in this follow-up. `.tooling/` is local validation tooling and must not be committed.
- The expected new paths are absent from Git, including:
  - `installer/nox-installer.py`
  - `.github/workflows/flavour-image.yml`
  - `examples/live-hyprland/` and `examples/live-niri/`
  - `nix/modules/boot/`, `desktop/`, `gaming/`, `hardware/`, `install/`, and `user/`
- The current flake exposes the existing generic `iso`, `qcow2`, `wsl`, and `oci` outputs; it does not expose `hyprland-iso` or `niri-iso`.

## Validation performed in this follow-up

Environment evidence:

```text
architecture: x86_64
rustc: 1.85.0 (4d91de4e4 2025-02-17)
cargo: 1.85.0 (d73d2caf9 2024-12-31)
python: 3.12.14
Cargo.lock SHA256: fb48625ad78a2e06188b1acf0ca68771d8586b0d49608868997d371df62f26f0
flake.lock SHA256: 8ae35abd1312e16297bd00e5e1f4846a2d26cfc01dc79825ee2c740e9e4c8433
```

Current-baseline results:

| Check | Result | Scope |
|---|---|---|
| `cargo fmt --all -- --check` | Pass | Baseline Rust sources |
| `cargo test --workspace --locked` | Pass | 4 `nox-config`, 2 `nox-core`, 1 `noxctl` unit test |
| `cargo build -p noxctl --locked` | Pass | Baseline CLI |
| `python3 -m unittest discover -s tests/cli -v` | Pass | 15 fake-process lifecycle tests |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass | Baseline workspace |
| TOML parsing | Pass | 56 files found in the checkout, including local tool metadata |
| `bash -n scripts/*.sh` | Pass | Baseline shell scripts |
| `git diff --check` | Pass | Current textual changes |
| `nix flake check --no-update-lock-file --show-trace` | Not run | `nix` is unavailable |
| ISO build | Not run | `nix` is unavailable and flavour outputs are missing |
| UEFI VM/live-session test | Not run | QEMU is unavailable and no flavour ISO exists |
| Destructive install/reboot test | Not run | Requires recovered code and a disposable disk/VM |

## Intended implementation that must be recovered or reapplied

The earlier scratch implementation was reported to contain the following. These are design notes, not claims about the current Git tree:

- Typed TOML/Rust settings for desktop flavour, graphics driver, bootloader, filesystem, locale, user, and install disk.
- Hyprland and Niri desktop modules with greetd, NetworkManager, PipeWire, portals, policy/keyring support, and starter configurations.
- Gaming defaults for Steam, Proton, GameScope, GameMode, MangoHud, Wine tooling, and 32-bit graphics.
- AMD, Intel, NVIDIA open/proprietary, and VM graphics selections.
- Explicit `/dev/disk/by-id/...` installation disks, Disko Btrfs/ext4 layouts, and systemd-boot/GRUB EFI choices.
- A GTK4 `nox-installer` that generates the declarative project through `noxctl`, validates before mutation, and requests confirmation before erasing a disk.
- `noxctl installer gui` and a guarded `noxctl installer local` flow.
- Separate `hyprland-iso` and `niri-iso` outputs.
- A manual-only workflow that builds and publishes exactly one selected flavour per invocation.

The earlier reported Rust/CLI tests included additional cases, but their sources are missing, so those results cannot be reproduced or treated as evidence.

## Next concrete action

Recover the prior scratch working tree or reapply the implementation onto a new branch from `0840104`. Do not start image builds against the current baseline: the requested flavour outputs do not exist.

Once the code is recovered:

1. Compare it with this checkout using `git diff --no-index` or apply it as a patch, excluding `.tooling/` and build outputs.
2. Re-run Rust formatting, unit tests, CLI lifecycle tests, and Clippy.
3. Run `nix flake check --no-update-lock-file --show-trace` in a Nix-enabled environment.
4. Build and test only one image set at a time:

   ```bash
   nix build .#hyprland-iso --no-update-lock-file
   # Boot, test, record evidence, then remove/archive its result.
   nix build .#niri-iso --no-update-lock-file
   ```

5. Boot each image in a disposable UEFI VM and verify the live session, networking, audio, selected compositor, installer generation, exact-disk confirmation, installation, reboot, and persistent system.
6. Update `README.md`, getting-started, testing, and operations documentation only after the verified interface is stable.
7. Review the full diff, commit the complete coherent changeset directly to `main`, and push once so normal CI runs once. Invoke the manual image workflow separately for each flavour.

## Safety and scope invariants

- `noxctl` may write `nox.toml`; it must not rewrite arbitrary Nix modules.
- A disk-changing command must require a stable by-id device, show the planned destructive action, and require an exact confirmation value.
- Flake archive/evaluation and system build must complete before disk mutation.
- Never count mock CLI tests, syntax checks, or flake evaluation as proof that an ISO boots or an installation survives reboot.
- Keep profiles, capabilities, and targets separate. Prefer upstream NixOS, Disko, and nixos-anywhere integration.
- Leave `apply`, rollback, daemon, web UI, clustering, and marketplace work deferred until configuration/build/install contracts have runtime evidence.

## Resume commands

```bash
git fetch origin main
git rev-parse HEAD origin/main
git status --short
rg -n "hyprland|niri|DesktopFlavour|installer local|flavour-image" . \
  --glob '!target/**' --glob '!.tooling/**'
```

If the search is empty after recovery was expected, stop and locate the missing patch/worktree before doing further release validation.
