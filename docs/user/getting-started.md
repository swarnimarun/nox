# Getting started

Install Nix with flakes enabled, clone the repository, and enter the
development shell:

```sh
nix develop
```

Create a machine definition in a new directory:

```sh
cargo run -p noxctl -- init examples/atlas
cargo run -p noxctl -- validate examples/atlas/nox.toml
cargo run -p noxctl -- plan examples/atlas/nox.toml
```

The example configuration is declarative input only. The current CLI does not
install, activate, repartition, or roll back a host. Those operations will be
introduced only with an explicit plan and safety contract.

