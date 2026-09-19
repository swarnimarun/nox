#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v nix >/dev/null || { echo 'Install Nix with nix-command and flakes enabled first.' >&2; exit 1; }
# Resolve dependencies explicitly; subsequent build commands refuse lock updates.
if [[ ! -f flake.lock ]]; then nix flake lock; fi
if [[ ! -f Cargo.lock ]]; then nix develop --command cargo generate-lockfile; fi
nix build --no-update-lock-file .#noxctl
printf 'CLI built: %s/result/bin/noxctl\nReview and commit Cargo.lock and flake.lock.\n' "$PWD"
