#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 1 || ! -f "$1" ]]; then
  echo 'Usage: bash scripts/run-workspace.sh OCI-ARCHIVE' >&2
  exit 2
fi
podman load --input "$1"
# No host root, device, or home mounts. This workspace is intentionally ephemeral.
exec podman run --rm -it --name nox-workspace localhost/nox-workspace:dev
