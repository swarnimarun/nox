{
  description = "Nox: a declarative Linux platform with profiles and targets";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      profileSystem =
        system: profile:
        nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            ./nix/profiles/${profile}.nix
            ./nix/targets/metal.nix
          ];
        };
    in
    {
      devShells = forAllSystems (system: {
        default = nixpkgs.legacyPackages.${system}.mkShell {
          packages = with nixpkgs.legacyPackages.${system}; [
            cargo
            clippy
            nixfmt-rfc-style
            rustc
            rustfmt
          ];
          shellHook = ''
            echo "Nox development shell"
            echo "Run: cargo test --workspace && nix flake check"
          '';
        };
      });

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-rfc-style);

      nixosConfigurations = {
        example-server = profileSystem "x86_64-linux" "server";
        example-desktop = profileSystem "x86_64-linux" "desktop";
        example-recovery = profileSystem "x86_64-linux" "recovery";
      };

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          # Force evaluation of the module graph without building an image.
          evaluated = profileSystem system "server";
        in
        {
          nixos-module-evaluation = pkgs.runCommand "nox-nixos-module-evaluation" { } (
            builtins.deepSeq evaluated.config.system.stateVersion ''
              printf '%s\n' "NixOS module evaluation succeeded" > $out
            ''
          );
        }
      );
    };
}
