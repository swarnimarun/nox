# Nox

Nox is a NixOS-based platform with a Rust CLI and one declarative machine
model. Profiles describe purpose, capabilities compose features, and targets
select metal, ISO, qcow2, WSL, or OCI outputs.

## Implemented contracts

- `noxctl setup` creates and locks a complete machine project; `init` remains
  available for an offline scaffold.
- `validate`, `plan`, `doctor`, `build`, `image build`, guarded SSH and local
  installation, dependency upgrade preview/apply, module registration,
  dotfiles scaffolding, and read-only generation listing.
- Hyprland and Niri Wayland live ISOs, EFI qcow2, an upstream WSL image
  builder, and an OCI workspace.
- GTK4 live installer with stable-disk discovery, immutable preflight build,
  explicit erase confirmation, and Disko-backed Btrfs/ext4 layouts.
- Git/flake-backed Home Manager dotfiles and user-owned NixOS extension modules.
- Rust and process-boundary tests, Nix target evaluation, a NixOS server VM
  test, and exact UEFI ISO smoke tests in the release workflow.

Nox supplies `nix-command` and `flakes` for every Nix subprocess and enables
them in produced systems. A lock upgrade is retained only after the complete
NixOS toplevel evaluates and builds; a failed validation restores the previous
`flake.lock`. It does not activate the new generation.

`apply` and `rollback` remain reserved until timed recovery and health-check
contracts pass failure-injection tests. WSL runtime, installed-system reboot,
and hardware-specific graphics still need target-specific evidence.

## Quick start

```sh
bash scripts/bootstrap.sh

./result/bin/noxctl setup ./my-nox \
  --profile desktop \
  --target qcow2 \
  --flavour niri \
  --source "path:$PWD"

./result/bin/noxctl validate --config ./my-nox/nox.toml
./result/bin/noxctl plan --config ./my-nox/nox.toml
./result/bin/noxctl image build --config ./my-nox/nox.toml --out-link result-vm
```

Every new project includes `modules/system.nix`, which is user-owned and never
rewritten by Nox. Register more project-local modules with:

```sh
noxctl module add --config ./my-nox/nox.toml modules/hardware.nix
noxctl module list --config ./my-nox/nox.toml
```

Preview and validate dependency updates without switching the running system:

```sh
noxctl upgrade list --config ./my-nox/nox.toml
noxctl upgrade apply --config ./my-nox/nox.toml
git -C ./my-nox diff -- flake.lock
```

See [project management](docs/user/project-management.md) for custom flake
inputs, Git/Home Manager dotfiles, and the upgrade safety model. See
[getting started](docs/user/getting-started.md) for VM, live ISO, WSL,
container, and metal installation procedures.

## Images and release evidence

Hyprland/Niri ISOs, an importable WSL image, and an OCI workspace image are
published together on the single [GitHub prerelease](https://github.com/swarnimarun/nox/releases).
Download the `.iso`, `.wsl`, or `.oci.tar.gz` file and verify its adjacent SHA-256 file. Each
provenance JSON records the source commit, lock hashes, image hash,
architecture, and validation performed by CI.

The release workflow builds heavyweight artifacts serially. Its ISO gate boots
the exact image with UEFI and waits for NetworkManager, greetd, the selected
compositor, and the GTK installer process.

## Plan and evidence

The [implementation plan](docs/operations/implementation-plan.md) maps
components to acceptance gates. The [testing guide](docs/operations/testing.md)
separates evaluation, artifact build, and runtime evidence.
[PLAN.md](PLAN.md) retains the longer product architecture.

Apache-2.0; see [LICENSE](LICENSE).
