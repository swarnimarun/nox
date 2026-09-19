# Agent guidance for Nox

## Repository intent

Nox is a NixOS-based platform with one declarative model and separate
profiles, capabilities, and targets. Keep that separation visible in code,
docs, and CLI output.

## Required invariants

- `noxctl` may modify `nox.toml`; it must not rewrite arbitrary `.nix` files.
- Desired state belongs in Nix/TOML. Runtime observations belong in a future
  daemon and must not become canonical configuration.
- Destructive operations must be explicit, planned, and guarded by tests.
- Prefer upstream NixOS, Disko, nixos-anywhere, Incus, Podman, ZFS/Btrfs,
  sops-nix, and Home Manager integrations over new implementations.

## Validation

Use the focused guidance in `.agents/skills/` and run the checks in
`docs/operations/testing.md`. Changes that cannot run locally because Nix or
Rust is missing must still leave CI coverage and a reproducible command for
the next developer.

## Scope control

Do not start `noxd`, the web UI, clustering, or an app marketplace until the
CLI/configuration/build/apply/rollback contracts are stable and covered by
tests.

