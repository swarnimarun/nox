# Testing and validation

## Fast contract checks

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo build -p noxctl --locked
python3 -m unittest discover -s tests/cli -v
cargo clippy --workspace --all-targets --locked -- -D warnings
nix fmt .
nix flake check --no-update-lock-file --show-trace
```

The CLI tests substitute a fake `nix` executable and verify arguments, failure
propagation, no-overwrite behavior, dry-run behavior and installer interlocks.
They never invoke a real disk operation. This does not establish that the real
installer can boot a destination.

The Nix check forces each artifact derivation, rather than just stateVersion.
The server VM test boots NixOS, waits for multi-user and SSH, and checks Git and
Podman. CI also creates a real generated Niri/qcow2 project, attaches the
generated Home Manager dotfiles flake, locks it, evaluates its toplevel and
Home Manager user, and runs the non-mutating upgrade preview.

The release workflow is the exact-image gate. It starts only after successful
push CI on `main`, or a manual dispatch on `main`. It pins the tested source
commit and checks the installer's GTK imports before building Hyprland, Niri,
WSL, and OCI serially in one runner. Each ISO boots with QEMU (KVM when available,
TCG otherwise) and UEFI; its marker requires NetworkManager, greetd, the selected
compositor, and `nox-installer` to run. PR CI also boot-tests Hyprland and validates
WSL, but cannot publish images or write caches.

## Target release evidence

| Target | Required runtime evidence |
|---|---|
| Hyprland/Niri ISO | Exact ISO boots with UEFI; NetworkManager, greetd, selected compositor, and GTK installer run |
| qcow2 | Exact image boots, local login works, filesystem persists on a non-snapshot test |
| WSL | Build tarball, Windows WSL2 import, default user, systemd and noxctl |
| OCI | Rootless Podman load/run; shell, Git and noxctl; no host filesystem changes |
| metal | Disposable disk install via SSH, reboot and SSH reachability, correct root/ESP |
| desktop/gaming | Display-manager login and hardware/graphics tests on target devices |

Record commit SHA, flake.lock, Cargo.lock, architecture, image SHA256, command,
exit status and console logs. A failed or skipped test never counts as support.
A manual dispatch rebuilds the full set. Assets upload to a new draft release,
`v0.2.0-alpha.<run-number>-<attempt>`, pinned to the source SHA. Only after all four
targets pass does CI publish it, then delete older Nox `v0.2.0-alpha.*` preview
releases and stale `nox-*` Actions artifacts. Stable releases and unrelated
prereleases/artifacts are preserved. A failed build removes its draft and keeps
the previous published images. Tags are never moved. The release includes four
images, their checksums and provenance, and both ISO console logs (at least 14 assets). ISOs use Zstd level 6 to avoid
level 19's lengthy compression step. Images at or above 2 GiB upload as 1 GiB
parts with an ordered manifest and part checksums, retaining the original image
checksum. Parts stream to GitHub one at a time so local disk does not hold two
full sets. Publication still waits for every part upload and hash verification.
OCI is loaded and exercised with Docker without network or host mounts. WSL
packaging runs as root; Windows WSL2 runtime validation remains external.

## Build cache retention

CI restores one shared Nix cache without saving. The image job is the only
writer, serialized across runs. Its unique `nox-build-v1-*` key refreshes the
snapshot each successful build; fallback restoration reuses the previous
snapshot even when locks or source change. Nix's content-addressed store decides
which derivations can be reused. Images and local result roots are removed after
upload, before cache saving. Garbage collection targets a store below 6 GiB and
an explicit total `/nix` size check skips saving if rooted paths exceed the 6 GiB
limit. Cache storage is a performance optimization and is not release evidence.

After the save finishes, a separate job retains only the newest `nox-build-*`
cache across every ref and cache version. Replacement can briefly overlap the
previous snapshot; steady state is one cache, at most 6 GiB before compression.
If a build or upload fails, the prior cache remains. Other cache namespaces are
untouched. No Actions image artifacts are staged, so there is no duplicate image
archive retention cost.

Validate cleanup boundaries locally with:

```sh
node --test tests/ci/*.test.cjs
python3 -m unittest discover -s tests/ci -p 'test_*.py' -v
bash -n scripts/build-release.sh
```

## Before enabling apply or rollback

Implement and test: prior generation capture; diff before activation; rollback
scheduled independently of the CLI/SSH process; temporary activation; timeout,
health failure and interrupted-session recovery; explicit confirmation; boot
selection persistence; locking against simultaneous operations. Include
firewall/network changes and reboot failure cases. Never infer recovery from a
mocked successful command.
