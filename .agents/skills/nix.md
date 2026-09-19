# Nix skill: modules and flakes

- Treat profiles as composition over small modules.
- Keep targets separate from profiles.
- Do not hide destructive disk changes in a default module.
- Pin nixpkgs in `flake.lock` once the supported channel is selected.
- Use `nix flake check` for evaluation and add NixOS VM tests before runtime
  integrations are considered stable.
