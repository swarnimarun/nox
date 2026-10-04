#!/usr/bin/env bash
set -euo pipefail
image="${1:?usage: test-container.sh IMAGE.tar.gz}"
gzip -t "$image"
docker load --input "$image"
docker run --rm --network none nox-workspace:dev /bin/bash -lc '
  set -eu
  noxctl --help >/dev/null
  git --version >/dev/null
  nix --extra-experimental-features "nix-command flakes" --version >/dev/null
  test -f /etc/nix/nix.conf
  grep -q "experimental-features = nix-command flakes" /etc/nix/nix.conf
  test ! -L /etc/nix/nix.conf
  test ! -L /etc/sudoers
  test "$(stat -c %a /etc/sudoers)" = 440
'
