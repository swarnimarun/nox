#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v nix >/dev/null || { echo 'Install Nix with nix-command and flakes enabled first.' >&2; exit 1; }
# Resolve dependencies explicitly; subsequent build commands refuse lock updates.
nix flake lock
nix develop --command cargo generate-lockfile
nix build --no-update-lock-file .#noxctl
printf 'CLI built: %s/result/bin/noxctl\nReview and commit Cargo.lock and flake.lock.\n' "$PWD"
