# Contributing

Start with a contract. If a change affects profiles, capabilities, targets,
configuration ownership, application manifests, or lifecycle safety, write or
update an RFC in `docs/rfcs/` before implementing it.

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo test --workspace
nix develop --command bash -lc 'nixfmt --check flake.nix $(find nix tests -name "*.nix")'
nix flake check
```

Keep changes narrow. Do not add a new runtime, storage, or orchestration
abstraction when an upstream project already owns that problem.
