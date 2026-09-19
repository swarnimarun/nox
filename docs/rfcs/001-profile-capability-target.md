# RFC 001: profile, capability, and target model

## Status

Accepted for the initial scaffold.

## Decision

Nox models a machine as one profile plus a set of capabilities and one target.
Profiles are user-facing presets; capabilities are composable features; a
target describes the runtime or artifact.

## Why

The original “five modes” combined purpose, deployment location, and isolation
backend. Keeping those axes independent allows combinations such as a server
with a desktop, or a workspace built as both WSL and OCI, without branching
the whole operating system.

## Consequences

Nix modules should expose small options and profiles should mostly compose
those modules. The CLI and schema must use the same vocabulary. “Bootable”
becomes a recovery/server profile plus an ISO target.

