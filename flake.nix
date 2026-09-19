{
  description = "Nox: declarative Linux profiles, images and installation";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixos-wsl = {
      url = "github:nix-community/NixOS-WSL";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    disko = {
      url = "github:nix-community/disko";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixos-anywhere = {
      url = "github:nix-community/nixos-anywhere";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.disko.follows = "disko";
    };
  };
  outputs =
    inputs@{ self, nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      each = nixpkgs.lib.genAttrs systems;
      noxLib = import ./nix/lib { inherit inputs; };
      machine =
        system: name:
        noxLib.mkSystem {
          inherit system;
          configFile = ./examples/${name}/nox.toml;
        };
      cli =
        system:
        nixpkgs.legacyPackages.${system}.rustPlatform.buildRustPackage {
          pname = "noxctl";
          version = "0.2.0";
          src = nixpkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "-p"
            "noxctl"
          ];
          cargoTestFlags = [ "--workspace" ];
        };
    in
    {
      lib = noxLib;
      packages = each (system: {
        noxctl = cli system;
        default = self.packages.${system}.noxctl;
        iso = noxLib.artifact (machine system "recovery");
        qcow2 = noxLib.artifact (machine system "vm");
        wsl = noxLib.artifact (machine system "wsl");
        oci = noxLib.artifact (machine system "container");
        installer = inputs.nixos-anywhere.packages.${system}.default;
      });
      apps = each (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.noxctl}/bin/noxctl";
        };
      });
      nixosConfigurations = {
        nox-vm = machine "x86_64-linux" "vm";
        nox-recovery = machine "x86_64-linux" "recovery";
        nox-wsl = machine "x86_64-linux" "wsl";
      };
      devShells = each (system: {
        default = nixpkgs.legacyPackages.${system}.mkShell {
          packages = with nixpkgs.legacyPackages.${system}; [
            cargo
            clippy
            rustc
            rustfmt
            nixfmt
            python3
          ];
        };
      });
      formatter = each (system: nixpkgs.legacyPackages.${system}.nixfmt);
      checks = each (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          cli = cli system;
          server-boot = pkgs.testers.runNixOSTest (import ./tests/nix/server-boot.nix);
          target-evaluation = pkgs.runCommand "nox-target-evaluation" { } (
            builtins.deepSeq (map (name: (noxLib.artifact (machine system name)).drvPath) [
              "vm"
              "recovery"
              "wsl"
              "container"
            ]) "touch $out"
          );
        }
      );
    };
}
