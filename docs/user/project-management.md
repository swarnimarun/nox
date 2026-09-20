# Project setup, upgrades, and dotfiles

## Create a locked project

`setup` is the normal entry point. It writes the machine model, generated
flake, and a user-owned extension module, then resolves `flake.lock`:

```sh
noxctl setup ./workstation \
  --profile desktop \
  --target metal \
  --flavour niri \
  --graphics amd \
  --disk /dev/disk/by-id/EXACT-DISK-ID
```

Use `noxctl init` instead when creating the files offline; follow it with
`noxctl lock --config ./workstation/nox.toml`. Both commands require an empty
destination and refuse to overwrite files.

Nox passes `--extra-experimental-features "nix-command flakes"` to every Nix
subprocess. Produced systems also set those features in `nix.settings`, so no
separate experimental-feature setup is required for Nox commands.

Commit `nox.toml`, `flake.nix`, `flake.lock`, and `modules/` together. The lock
is part of the machine definition, not a build by-product.

## Add machine-specific configuration

New projects register `modules/system.nix` in `[nix].extra_modules`. Nox
creates that file once and never rewrites it. Put normal NixOS options there:

```nix
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    helix
    ripgrep
  ];

  services.printing.enable = true;
}
```

Additional modules must already exist below the project directory. Registration
updates only `nox.toml`; removal does not delete the module:

```sh
noxctl module add --config ./workstation/nox.toml modules/printing.nix
noxctl module list --config ./workstation/nox.toml
noxctl module remove --config ./workstation/nox.toml modules/printing.nix
```

Absolute paths, parent-directory escapes, non-`.nix` files, missing files, and
symlinks resolving outside the project are rejected.

The generated flake forwards additional direct inputs to custom modules through
the `inputs` argument. Add an input explicitly in `flake.nix`, then consume it
from a registered module. Nox does not rewrite `flake.nix` or arbitrary Nix
source after project creation.

## Git-backed Home Manager dotfiles

Create a standalone dotfiles flake:

```sh
noxctl dotfiles init ./dotfiles --username alice
git -C ./dotfiles init
git -C ./dotfiles add flake.nix flake.lock home.nix .gitignore
git -C ./dotfiles commit -m "Initial Home Manager dotfiles"
```

The scaffold follows NixOS/Home Manager 26.05 and exports
`nixosModules.default`. Edit `home.nix` to add user packages, programs, and
managed files. Push it to a Git host, then attach it while creating a machine:

```sh
noxctl setup ./laptop \
  --profile desktop \
  --target metal \
  --flavour niri \
  --username alice \
  --dotfiles-flake github:alice/dotfiles
```

A local checkout works during development:

```sh
noxctl setup ./laptop \
  --profile desktop \
  --target qcow2 \
  --username alice \
  --dotfiles-flake "path:$PWD/dotfiles"
```

The machine lock pins both Nox and the dotfiles revision. The dotfiles flake is
imported as a NixOS module, which wires Home Manager into the same system
generation instead of running an unrelated post-install copy script.

## Inspect and apply dependency upgrades

Preview all input changes without modifying the real lock:

```sh
noxctl upgrade list --config ./workstation/nox.toml
```

The command asks Nix to resolve an alternate temporary lock with
`--output-lock-file`, prints Nix's input changes, and removes the preview. It
refuses to reuse an existing preview file.

Apply an upgrade to the project:

```sh
noxctl upgrade apply --config ./workstation/nox.toml
git -C ./workstation diff -- flake.lock
```

Nox updates the lock using the explicit project flake, then evaluates and builds
`nixosConfigurations.nox.config.system.build.toplevel` with lock updates
disabled. If that validation fails, the previous `flake.lock` is restored. On
success, review and commit the lock.

This command upgrades pinned dependencies only. It deliberately does not run
`nixos-rebuild switch`, change the booted generation, or reboot. Safe system
activation remains separate until timeout, health-check, and automatic rollback
contracts are implemented and tested.
