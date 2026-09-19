{ inputs }:
let
  inherit (inputs.nixpkgs) lib;
  readConfig = configFile:
    let
      c = builtins.fromTOML (builtins.readFile configFile);
      caps = c.capabilities or [ ];
      validName = builtins.match "[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?" c.name != null;
    in
    assert lib.assertMsg (c.schema_version == 1) "Unsupported Nox schema";
    assert lib.assertMsg (validName && builtins.stringLength c.name <= 63) "Invalid hostname";
    assert lib.assertMsg (builtins.elem c.profile [ "server" "desktop" "gaming" "workspace" "recovery" ]) "Unknown profile";
    assert lib.assertMsg (builtins.elem c.target [ "metal" "iso" "qcow2" "wsl" "oci" ]) "Unknown target";
    assert lib.assertMsg (lib.all (x: builtins.elem x [ "desktop" "gaming" "storage" "shares" "apps" "virtualization" "development" "remote-management" "recovery" ]) caps) "Unknown capability";
    assert lib.assertMsg (c.target != "wsl" || c.profile == "workspace") "WSL requires workspace profile";
    assert lib.assertMsg (c.target != "oci" || (c.profile == "workspace" && builtins.elem "development" caps)) "OCI requires workspace/development";
    assert lib.assertMsg (c.profile != "gaming" || builtins.elem "gaming" caps) "Gaming capability required";
    assert lib.assertMsg (c.profile != "recovery" || builtins.elem "recovery" caps) "Recovery capability required";
    assert lib.assertMsg ((c.nix.channel or "nixos-26.05") == "nixos-26.05") "Change the flake input and schema together to change channels";
    c;
  mkSystem = { configFile, system ? "x86_64-linux", extraModules ? [ ] }:
    let c = readConfig configFile;
    in inputs.nixpkgs.lib.nixosSystem {
      inherit system;
      modules = [
        ../profiles/${c.profile}.nix
        ../targets/${c.target}.nix
        ../capabilities
        ({ ... }: {
          networking.hostName = c.name;
          environment.systemPackages = [ inputs.self.packages.${system}.noxctl ];
          nox.capabilities = c.capabilities or [ ];
          nix.settings.experimental-features = [ "nix-command" "flakes" ];
          services.openssh.settings.PasswordAuthentication = false;
          services.openssh.settings.KbdInteractiveAuthentication = false;
        })
      ] ++ lib.optionals (c.target == "wsl") [ inputs.nixos-wsl.nixosModules.default ]
        ++ lib.optionals (c.target == "metal") [ inputs.disko.nixosModules.disko ]
        ++ map (p: builtins.toPath "${toString (builtins.dirOf configFile)}/${p}") (c.nix.extra_modules or [ ])
        ++ extraModules;
    };
  artifact = machine:
    let c = machine.config;
    in {
      metal = c.system.build.toplevel;
      iso = c.system.build.isoImage;
      qcow2 = c.system.build.noxQcow2;
      wsl = c.system.build.tarballBuilder;
      oci = c.system.build.noxOci;
    }.${c.nox.target};
in { inherit readConfig mkSystem artifact; }
