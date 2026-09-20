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
      liveMachine = system: flavour: machine system "live-${flavour}";
      cli =
        system:
        nixpkgs.legacyPackages.${system}.rustPlatform.buildRustPackage {
          pname = "noxctl";
          version = "0.1.0";
          src = nixpkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "-p"
            "noxctl"
          ];
          cargoTestFlags = [ "--workspace" ];
        };
      graphicalInstaller =
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          python = pkgs.python3.withPackages (packages: [ packages.pygobject3 ]);
        in
        pkgs.writeShellApplication {
          name = "nox-installer";
          runtimeInputs = [
            (cli system)
            pkgs.coreutils
            pkgs.gtk4
            pkgs.systemd
            pkgs.util-linux
            python
          ];
          text = ''
            export GI_TYPELIB_PATH="${pkgs.gtk4}/lib/girepository-1.0:${pkgs.glib}/lib/girepository-1.0:${pkgs.gdk-pixbuf}/lib/girepository-1.0''${GI_TYPELIB_PATH:+:''${GI_TYPELIB_PATH}}"
            export NOX_SOURCE="path:${self.outPath}"
            exec ${python}/bin/python3 ${./installer/nox-installer.py} "$@"
          '';
        };
    in
    {
      lib = noxLib;
      packages = each (
        system:
        {
          noxctl = cli system;
          graphical-installer = graphicalInstaller system;
          disko = inputs.disko.packages.${system}.disko;
          default = self.packages.${system}.noxctl;
          iso = noxLib.artifact (machine system "recovery");
          qcow2 = noxLib.artifact (machine system "vm");
          wsl = noxLib.artifact (machine system "wsl");
          oci = noxLib.artifact (machine system "container");
          installer = inputs.nixos-anywhere.packages.${system}.default;
          vm-smoke = nixpkgs.legacyPackages.${system}.writeShellApplication {
            name = "nox-vm-smoke";
            runtimeInputs = with nixpkgs.legacyPackages.${system}; [
              python3
              qemu
            ];
            text = ''
              exec python3 ${./tests/boot_qcow2.py} ${self.packages.${system}.qcow2}/nixos.qcow2 ${
                nixpkgs.legacyPackages.${system}.OVMF.fd.firmware
              } "$@"
            '';
          };
        }
        // nixpkgs.lib.optionalAttrs (system == "x86_64-linux") {
          hyprland-iso = noxLib.artifact (liveMachine system "hyprland");
          niri-iso = noxLib.artifact (liveMachine system "niri");
          iso-smoke = nixpkgs.legacyPackages.${system}.writeShellApplication {
            name = "nox-iso-smoke";
            runtimeInputs = with nixpkgs.legacyPackages.${system}; [
              python3
              qemu
            ];
            text = ''
              if [ "$#" -lt 1 ]; then
                echo "usage: nox-iso-smoke IMAGE.iso [OPTIONS]" >&2
                exit 2
              fi
              image="$1"
              shift
              exec python3 ${./tests/boot_iso.py} "$image" ${
                nixpkgs.legacyPackages.${system}.OVMF.fd.firmware
              } "$@"
            '';
          };
        }
      );
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
        nox-hyprland-live = liveMachine "x86_64-linux" "hyprland";
        nox-niri-live = liveMachine "x86_64-linux" "niri";
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
          installer-syntax =
            pkgs.runCommand "nox-installer-syntax" { nativeBuildInputs = [ pkgs.python3 ]; }
              ''
                python3 -m py_compile ${./installer/nox-installer.py}
                touch $out
              '';
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
        // nixpkgs.lib.optionalAttrs (system == "x86_64-linux") {
          flavour-contract =
            let
              hyprland = liveMachine system "hyprland";
              niri = liveMachine system "niri";
            in
            pkgs.runCommand "nox-flavour-contract" { } (
              builtins.deepSeq [
                hyprland.config.system.build.isoImage.drvPath
                hyprland.config.nox.desktop.flavour
                niri.config.system.build.isoImage.drvPath
                niri.config.nox.desktop.flavour
              ] "touch $out"
            );
        }
      );
    };
}
