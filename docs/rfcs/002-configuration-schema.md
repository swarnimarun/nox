# RFC 002: configuration ownership and schema

## Status

Accepted for the initial scaffold.

## Decision

`nox.toml` is the stable, human-facing configuration layer. It has an
explicit `schema_version`, profile, target, capabilities, and Nix settings.
Advanced Nix remains available through `flake.nix`, host modules, and custom
modules, but Nox tooling must not rewrite those files.

## Compatibility

Schema changes require a migration or a new schema version. Validation should
fail with actionable errors rather than silently dropping fields.

