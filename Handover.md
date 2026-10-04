# Nox implementation handover

Updated: 2026-10-05

## Current status

PRs [#10](https://github.com/swarnimarun/nox/pull/10) and
[#11](https://github.com/swarnimarun/nox/pull/11) are merged. The verified release
source is `7b7b36ba38c33d8ccee045ddee78de32ccb57233`.

[Nox v0.2.0-alpha.43-1](https://github.com/swarnimarun/nox/releases/tag/v0.2.0-alpha.43-1)
is published with 26 uploaded assets: Hyprland and Niri ISO parts, WSL and OCI
images, image and part checksums, ordered part manifests, provenance, and ISO
console logs. The two ISOs exceed GitHub's per-asset limit and each uses five
ordered parts. Follow the README reassembly instructions before booting them.

[Main CI](https://github.com/swarnimarun/nox/actions/runs/37240014863) and the
[complete release workflow](https://github.com/swarnimarun/nox/actions/runs/37240291314)
passed. Cache retention confirmed exactly one Nox build-cache snapshot.

## Verified evidence

| Gate | Result |
|---|---|
| Rust formatting, workspace tests, locked CLI build, Clippy | Pass |
| Python CLI lifecycle and installer interlock suite | Pass |
| Release cleanup, process matching, multipart reassembly and failure tests | Pass |
| Nix formatting, flake checks and server VM boot | Pass |
| Generated Niri/qcow2 project with Home Manager dotfiles | Lock and evaluation pass |
| GTK installer dependency imports | Pass |
| Exact Hyprland and Niri ISO UEFI boot | Pass in the release workflow |
| WSL tarball packaging and required configuration files | Pass |
| OCI build and Docker runtime without network or host mounts | Pass |
| OCI configuration files and sudoers permissions | Pass |
| One cache snapshot with a 6 GiB pre-save limit | Pass |

Windows WSL2 import and launch, rootless Podman, physical GPU coverage, and
installed-system reboot remain external acceptance gates. CI evidence does not
establish those runtime claims.

## Release and cache behavior

Successful push CI on `main` starts the release workflow; manual dispatch is
also restricted to `main`. The workflow pins the tested source SHA and builds
OCI, WSL, Hyprland, and Niri serially. Assets remain in a draft until every build,
runtime/archive check, checksum, and upload passes. Failed drafts are removed;
already published releases are preserved after ambiguous publishing errors.

Publication creates a new immutable preview tag. Only after the complete release
is public does cleanup remove older Nox `v0.2.0-alpha.*` previews and stale
`nox-*` Actions artifacts. Stable releases and unrelated artifacts are preserved.
There are no duplicate Actions image archives.

CI restores the shared Nix cache without saving PR versions. The serialized image
job is its only writer. A hard total `/nix` size check prevents snapshots above
6 GiB; the following retention job keeps only the newest `nox-build-*` cache
across refs and versions. Replacement briefly overlaps the prior snapshot, then
steady state returns to one version.

ISO compression uses Zstd level 6. In the measured Hyprland CI runs, the ISO build
step fell from about 16 minutes at level 19 to 90 seconds. The faster compression
produces larger images, so the uploader streams 1 GiB parts, verifies the complete
image hash, and publishes an ordered manifest plus part checksums.

## Safety boundaries

- Desired state remains in Nix/TOML; `noxctl` does not rewrite arbitrary user Nix files.
- Generated OCI configuration replaces read-only package links inside the image
  build, without modifying the host or Nix store.
- Disk installation requires an explicit stable by-id destination and confirmation.
  CI tests never install onto a real host disk.
- Upgrade apply does not activate or reboot the system. General apply/rollback
  remains reserved until timed recovery, health checks, and failure-injection
  coverage exist.
- `noxd`, a web UI, clustering, and an app marketplace remain outside the current
  implementation scope.

See [testing and validation](docs/operations/testing.md) for reproducible commands,
artifact evidence, and remaining target acceptance requirements.
