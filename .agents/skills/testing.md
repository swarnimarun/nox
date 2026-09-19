# Testing skill

Every change should have the narrowest useful test. Run Rust unit tests and
formatting locally, then run Nix formatting and flake evaluation. For host or
storage behavior, use a disposable VM or test machine and document the exact
destructive boundary before implementing it.
