# Nox

A NixOS-based platform with a Rust CLI and one declarative machine model:
profiles describe purpose, capabilities compose features, and targets select
metal, ISO, qcow2, WSL, or OCI userspace outputs.

## Implemented contracts

- `noxctl init`, `validate`, `plan`, `doctor`, `lock`, `build`, `image build`,
  guarded SSH `install`, and local `generations`.
- TOML-to-NixOS composition, five profiles and capability modules.
- ISO and EFI qcow2 derivations, upstream WSL tarball builder, OCI workspace.
- Disko/nixos-anywhere integration with explicit host/disk confirmation.
- Rust and CLI process-boundary tests, Nix target evaluation and server VM test.

These are implementation contracts, not certification that every target has
booted. WSL/metal/desktop runtime verification is still required. `apply` and
`rollback` remain reserved until their recovery contracts pass tests. The
runtime daemon, web UI and marketplace remain deferred by repository policy.

## Start testing

```sh
bash scripts/bootstrap.sh
./result/bin/noxctl --help
nix build --no-update-lock-file .#qcow2 --out-link result-vm
```

Bootstrap generates missing Cargo/Nix locks; review and commit them. See
[getting started](docs/user/getting-started.md) for exact VM, WSL, container and
metal installation steps, including test credentials and destructive boundaries.

The repository is private; use an authenticated GitHub checkout or local
`path:` flake reference. OCI is a userspace workspace sharing the host kernel;
use qcow2 to test a complete bootable NixOS system.

## Plan and evidence

[Implementation plan](docs/operations/implementation-plan.md) maps every
component to its acceptance gate and sequences the remaining lifecycle work.
[Testing guide](docs/operations/testing.md) distinguishes evaluation, artifact
build, and runtime evidence. [PLAN.md](PLAN.md) retains the product architecture.

Apache-2.0; see [LICENSE](LICENSE).
