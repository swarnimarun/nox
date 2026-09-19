# Rust skill: CLI and domain crates

- Keep `nox-config` free of filesystem and process side effects except its
  explicit `load` helper.
- Keep planning in `nox-core` deterministic and independently testable.
- Keep `noxctl` responsible for argument parsing, user-facing errors, and
  guarded effects.
- Prefer typed enums for profiles, targets, and capabilities.
- Add unit tests for schema compatibility and plan rendering with every
  behavior change.
