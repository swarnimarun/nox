# Testing and validation

## Local checks

Run these from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace
nix develop --command bash -lc 'nix fmt .'
git diff --exit-code -- . ':!flake.lock'
nix flake check
```

The repository also includes `scripts/check.sh`, which runs the checks that
are available and reports skipped toolchains explicitly. It always validates
the example TOML with Python's standard-library TOML parser.

## CI checks

GitHub Actions runs the Rust format/tests and Nix formatter/flake checks on
Linux. A later milestone should add NixOS VM tests for each profile and
integration tests for installation, activation timeout, rollback, WSL, and
storage adapters.

## Safety tests to add before `apply`

- invalid schemas never reach evaluation;
- plans are deterministic for identical inputs;
- disk operations require explicit confirmation;
- remote activation rolls back after health-check timeout;
- SSH/firewall/network changes are covered by a reachable test machine;
- secrets never appear in plans, logs, or generated artifacts.
