# Architecture overview

```text
                 noxctl
                   │
       ┌───────────┴───────────┐
       │                       │
  desired state           runtime state
  TOML + Nix              future noxd API
       │                       │
   nox-config              adapters
   nox-core          Incus · Podman · storage
       │
  NixOS evaluation
       │
  metal · ISO · qcow2 · WSL · OCI
```

The first implementation keeps the desired-state side pure and testable.
Runtime adapters and a daemon are intentionally later work. This gives the
CLI a stable domain model before it starts making changes to real machines.

## Repository map

- `crates/nox-config`: versioned TOML schema and validation.
- `crates/nox-core`: profile/capability/target planning with no I/O.
- `crates/noxctl`: CLI parsing and safe filesystem/process boundaries.
- `nix/modules`: reusable NixOS options.
- `nix/profiles`: purpose presets.
- `nix/targets`: artifact/runtime targets.
- `docs/rfcs`: decisions that should remain reviewable.
- `tests`: smoke and integration test homes.
