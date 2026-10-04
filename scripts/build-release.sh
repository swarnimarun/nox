#!/usr/bin/env bash
set -euo pipefail
: "${SOURCE_SHA:?}" "${RELEASE_TAG:?}" "${GH_REPO:?}"
for artifact in hyprland niri wsl container; do
  rm -rf release
  case "$artifact" in
    hyprland|niri)
      FLAVOUR="$artifact"
      mkdir -p release
      nix build --no-update-lock-file ".#${FLAVOUR}-iso" --out-link result-iso
      source_iso="$(find -L result-iso/iso -maxdepth 1 -type f -name '*.iso' -print -quit)"
      test -n "$source_iso"
      image="release/nox-${FLAVOUR}-x86_64-linux.iso"
      cp -- "$source_iso" "$image"
      test "$(stat --format=%s "$image")" -gt 1073741824
      nix run --no-update-lock-file .#iso-smoke -- "$image" --expect-flavour "$FLAVOUR" --timeout 900 --log iso-boot-console.log
      image_sha256="$(sha256sum "$image" | cut -d' ' -f1)"
      printf '%s  %s\n' "$image_sha256" "$(basename "$image")" > "$image.sha256"
      jq -n --arg commit "$SOURCE_SHA" --arg artifact "$FLAVOUR-iso" --arg architecture "x86_64-linux" --arg image_sha256 "$image_sha256" --arg boot_evidence_sha256 "$(sha256sum iso-boot-console.log | cut -d' ' -f1)" --arg flake_lock_sha256 "$(sha256sum flake.lock | cut -d' ' -f1)" --arg cargo_lock_sha256 "$(sha256sum Cargo.lock | cut -d' ' -f1)" '{commit: $commit, artifact: $artifact, architecture: $architecture, image_sha256: $image_sha256, uefi_boot_test: "passed", boot_evidence_sha256: $boot_evidence_sha256, flake_lock_sha256: $flake_lock_sha256, cargo_lock_sha256: $cargo_lock_sha256}' > "release/nox-${FLAVOUR}-x86_64-linux.provenance.json"
      cp iso-boot-console.log "release/nox-${FLAVOUR}-boot-console.log"
      ;;
    wsl)
      mkdir -p release
      nix build --no-update-lock-file .#wsl --out-link result-wsl
      image="release/nox-wsl-x86_64-linux.wsl"
      sudo ./result-wsl/bin/nixos-wsl-tarball-builder "$PWD/$image"
      test -s "$image"
      tar -tzf "$image" > "$RUNNER_TEMP/wsl-contents.txt"
      test "$(stat --format=%s "$image")" -gt 104857600
      grep -qx './etc/wsl-distribution.conf' "$RUNNER_TEMP/wsl-contents.txt"
      grep -qx './etc/nixos/configuration.nix' "$RUNNER_TEMP/wsl-contents.txt"
      image_sha256="$(sha256sum "$image" | cut -d' ' -f1)"
      printf '%s  %s\n' "$image_sha256" "$(basename "$image")" > "$image.sha256"
      jq -n --arg commit "$SOURCE_SHA" --arg artifact "wsl" --arg architecture "x86_64-linux" --arg image_sha256 "$image_sha256" --arg flake_lock_sha256 "$(sha256sum flake.lock | cut -d' ' -f1)" --arg cargo_lock_sha256 "$(sha256sum Cargo.lock | cut -d' ' -f1)" '{commit: $commit, artifact: $artifact, architecture: $architecture, image_sha256: $image_sha256, archive_validation: "passed", runtime_test: "requires Windows WSL2", flake_lock_sha256: $flake_lock_sha256, cargo_lock_sha256: $cargo_lock_sha256}' > release/nox-wsl-x86_64-linux.provenance.json
      ;;
    container)
      mkdir -p release
      nix build --no-update-lock-file .#oci --out-link result-oci
      image="release/nox-container-x86_64-linux.oci.tar.gz"
      cp -L -- result-oci "$image"
      test -s "$image"
      test "$(stat --format=%s "$image")" -gt 104857600
      gzip -t "$image"
      docker load --input "$image"
      docker run --rm --network none nox-workspace:dev /bin/bash -lc '
        set -eu
        noxctl --help >/dev/null
        git --version >/dev/null
        nix --extra-experimental-features "nix-command flakes" --version >/dev/null
        test -f /etc/nix/nix.conf
        grep -q "experimental-features = nix-command flakes" /etc/nix/nix.conf
      '
      image_sha256="$(sha256sum "$image" | cut -d' ' -f1)"
      printf '%s  %s\n' "$image_sha256" "$(basename "$image")" > "$image.sha256"
      jq -n --arg commit "$SOURCE_SHA" --arg artifact "container-oci" --arg architecture "x86_64-linux" --arg image_sha256 "$image_sha256" --arg flake_lock_sha256 "$(sha256sum flake.lock | cut -d' ' -f1)" --arg cargo_lock_sha256 "$(sha256sum Cargo.lock | cut -d' ' -f1)" '{commit: $commit, artifact: $artifact, architecture: $architecture, image_sha256: $image_sha256, archive_validation: "passed", docker_runtime_test: "passed", network_isolation: "passed", flake_lock_sha256: $flake_lock_sha256, cargo_lock_sha256: $cargo_lock_sha256}' > release/nox-container-x86_64-linux.provenance.json
      ;;
  esac
  # The release stays draft until every target and upload passes.
  gh release upload "$RELEASE_TAG" release/*
  rm -rf release result-iso result-wsl result-oci
  docker image rm nox-workspace:dev 2>/dev/null || true
  # Keep dependencies for later targets while reclaiming image outputs.
  store_bytes="$(du -sb /nix/store | cut -f1)"
  if (( store_bytes > 8589934592 )); then
    nix store gc --max "$((store_bytes - 8589934592))"
  fi
done
