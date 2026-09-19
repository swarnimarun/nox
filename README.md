# Nox

Nox is a NixOS-based Linux platform for machines that need a reproducible
system, a useful CLI, and composable profiles for servers, desktops, gaming,
workspaces, and recovery environments.

The project is intentionally one platform, not five distributions:

- **Profiles** describe purpose: `server`, `desktop`, `gaming`, `workspace`,
  and `recovery`.
- **Capabilities** describe composition: storage, shares, apps,
  virtualization, development, and remote management.
- **Targets** describe where the result runs: metal, ISO, qcow2, WSL, or OCI.

That separation is the foundation for building a Proxmox/TrueNAS-like server
experience, a Nix-friendly desktop, WSL support, and isolated workspaces
without maintaining unrelated operating systems.

## Current milestone

This repository is the first working contract, not a finished operating
system. It currently provides:

- a versioned TOML machine schema in `nox-config`;
- a pure planning layer in `nox-core`;
- a small Rust CLI named `noxctl` with `init`, `validate`, `plan`, and `doctor`;
- starter NixOS profiles and target modules;
- a Nix development shell and flake evaluation check;
- architecture, state-ownership, testing, and contribution documentation.

`build`, `image build`, `apply`, `generations`, and `rollback` are reserved
commands. They fail explicitly until the corresponding safety contracts are
implemented; they do not pretend to mutate a host.

## Quick start

With Nix installed:

```sh
nix develop
cargo test --workspace
cargo run -p noxctl -- init examples/atlas
cargo run -p noxctl -- validate examples/atlas/nox.toml
cargo run -p noxctl -- plan examples/atlas/nox.toml
nix flake check
```

The first `nix flake check` will create or refresh `flake.lock` when run by a
developer with permission to update the working tree. Commit that lock file
once the project chooses its first supported nixpkgs revision.

## Design rules

1. Nix owns desired system state; a future `noxd` owns observed runtime state.
2. `noxctl` may edit TOML but must never rewrite user-owned Nix.
3. Nox delegates package management, filesystems, VMs, containers, secrets,
   and backups to mature upstream projects.
4. Remote changes must evaluate, build, show a plan, activate temporarily,
   health-check, and only then become permanent.
5. Every new feature needs a testable contract before an integration.

See [PLAN.md](PLAN.md) for scope and sequencing, and [AGENTS.md](AGENTS.md)
for repository-specific development guidance.

## License

Nox is distributed under the Apache License 2.0. See [LICENSE](LICENSE).

