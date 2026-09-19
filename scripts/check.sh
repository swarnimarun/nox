#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

python3 - <<'PY'
import pathlib
import tomllib

for path in [pathlib.Path("nox.toml"), pathlib.Path("Cargo.toml")]:
    with path.open("rb") as handle:
        tomllib.load(handle)
print("ok: TOML files parse")

with pathlib.Path("nox.toml").open("rb") as handle:
    config = tomllib.load(handle)
assert config["schema_version"] == 1
assert config["profile"] in {"server", "desktop", "gaming", "workspace", "recovery"}
assert config["target"] in {"metal", "iso", "qcow2", "wsl", "oci"}
print("ok: example Nox configuration matches schema 1")
PY

if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all -- --check
  cargo test --workspace
else
  echo "skip: cargo is not installed"
fi

if command -v nix >/dev/null 2>&1; then
  nix develop --command bash -lc 'nixfmt --check flake.nix $(find nix tests -name "*.nix")'
  nix flake check
else
  echo "skip: nix is not installed"
fi

echo "validation complete"
