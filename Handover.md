# Nox implementation handover

Updated: 2026-09-20  
Recovery branch: `recovery/niri-wayland-installer`  
Validated implementation commit: `71c8c0ae9f76174c6e37875f5078fea76fe71cd0`

## Executive status

The missing Wayland/installer work was reconstructed on a dedicated recovery
branch and expanded with project setup, dependency upgrade, custom-module, and
Git/Home Manager dotfiles workflows. GitHub Actions run
[35510761589](https://github.com/swarnimarun/nox/actions/runs/35510761589)
passes both jobs at `71c8c0a`.

The recovery branch is ready for a pull request into `main`. The image workflow
intentionally runs only after successful `main` CI, so exact ISO build/boot and
prerelease publication are the remaining release gates.

## Implemented surface

- Typed Hyprland/Niri, graphics, bootloader, filesystem, locale, user, and
  stable install-disk settings in Rust, TOML, and Nix.
- Hyprland and Niri live systems with greetd, NetworkManager, PipeWire,
  portals, keyring/polkit, Waybar, starter compositor configs, and GTK4
  installer autostart.
- Gaming defaults for Steam/Proton, GameScope, GameMode, MangoHud, Lutris,
  Wine tooling, and 32-bit graphics.
- AMD, Intel, NVIDIA open/proprietary, automatic, and VM graphics selections.
- Disko Btrfs/ext4 layouts, systemd-boot/GRUB EFI, and guarded local or remote
  installation using one exact `/dev/disk/by-id/...` device.
- `noxctl setup`, offline `init`, automatic Nix experimental flags, guarded
  upgrade preview/apply, project-local module registration, and a Git-ready
  Home Manager dotfiles flake scaffold.
- Serialized CI builds for Hyprland ISO, Niri ISO, and WSL. A full release run
  deletes stale Actions artifacts and other prereleases, then publishes image,
  SHA-256, and provenance files under `v0.2.0-alpha.1`.

## Verified evidence

| Gate | Result |
|---|---|
| Rust formatting | Pass |
| Rust workspace tests | Pass |
| `noxctl` locked build | Pass |
| Python CLI lifecycle suite | Pass, including setup/modules/upgrades/dotfiles and installer interlocks |
| Python syntax checks | Pass |
| Clippy with warnings denied | Pass |
| Nix formatting | Pass |
| `nix flake check --no-update-lock-file --show-trace` | Pass |
| NixOS server VM test | Pass as part of flake checks |
| Generated Niri/qcow2 project with generated Home Manager dotfiles flake | Lock and evaluation pass |
| Non-mutating upgrade preview against a real generated flake | Pass |
| Exact Hyprland/Niri ISO UEFI boot | Pending post-merge release workflow |
| WSL archive packaging/validation | Pending post-merge release workflow |
| Windows WSL2 launch, physical GPU coverage, installed-system reboot | External runtime evidence still required |

The ISO readiness service now withholds its serial marker until NetworkManager,
greetd, the expected compositor process, and `nox-installer` are all running.
The release workflow boots the exact copied release ISO under QEMU/TCG with
UEFI and records the image and boot-log hashes in provenance.

## Safety boundaries

- `noxctl` may update `nox.toml` and `flake.lock`; it creates extension modules
  only in a new empty project and never rewrites user-owned `.nix` files.
- Upgrade preview resolves to a temporary alternate lock. Upgrade apply restores
  the previous lock if the new toplevel does not build.
- Upgrade apply does not activate or reboot the system.
- Disk mutation requires a stable by-id disk, an exact confirmation, a locked
  immutable flake snapshot, single-disk verification, and a successful preflight
  toplevel build.
- `apply` and `rollback` remain reserved until timed recovery, health checks,
  and failure-injection coverage exist.

## Release sequence

1. Open and review the recovery pull request.
2. Merge only while branch CI is green.
3. Confirm the merge commit's `main` CI succeeds.
4. Let `Flavour images` build Hyprland, Niri, and WSL serially.
5. Verify both exact ISO UEFI smoke tests and WSL archive validation.
6. Confirm the repository has exactly one prerelease,
   `v0.2.0-alpha.1`, with nine current assets and no stale Actions artifacts.
