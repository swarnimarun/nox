{ inputs }:
let
  inherit (inputs.nixpkgs) lib;
  readConfig =
    configFile:
    let
      raw = builtins.fromTOML (builtins.readFile configFile);
      desktop = raw.desktop or { };
      hardware = raw.hardware or { };
      boot = raw.boot or { };
      install = raw.install or { };
      locale = raw.locale or { };
      user = raw.user or { };
      c = raw // {
        desktop = desktop // {
          flavour =
            desktop.flavour or
              (if builtins.elem raw.profile [
                "desktop"
                "gaming"
              ] then
                "hyprland"
              else
                "none");
        };
        hardware = hardware // {
          graphics = hardware.graphics or "auto";
        };
        boot = boot // {
          loader = boot.loader or "systemd-boot";
        };
        install = install // {
          disk = install.disk or null;
          filesystem = install.filesystem or "btrfs";
        };
        locale = locale // {
          locale = locale.locale or "en_US.UTF-8";
          timezone = locale.timezone or "UTC";
          keymap = locale.keymap or "us";
        };
        user = user // {
          name = user.name or "nox";
        };
      };
      caps = c.capabilities or [ ];
      validName = builtins.match "[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?" c.name != null;
      keysOnly = value: allowed: lib.all (key: builtins.elem key allowed) (builtins.attrNames value);
      validUser =
        builtins.isString c.user.name
        && builtins.match "[a-z_][a-z0-9_-]*" c.user.name != null
        && builtins.stringLength c.user.name <= 32
        && c.user.name != "root";
      validDisk =
        c.install.disk == null
        || (
          builtins.isString c.install.disk
          && lib.hasPrefix "/dev/disk/by-id/" c.install.disk
          && c.install.disk != "/dev/disk/by-id/"
          && !(lib.hasInfix ".." c.install.disk)
          && !(lib.hasInfix "REPLACE" c.install.disk)
        );
    in
    assert lib.assertMsg (lib.all (
      key:
      builtins.elem key [
        "schema_version"
        "name"
        "profile"
        "target"
        "capabilities"
        "desktop"
        "hardware"
        "boot"
        "install"
        "locale"
        "user"
        "nix"
      ]
    ) (builtins.attrNames raw)) "Unknown Nox field";
    assert lib.assertMsg (keysOnly desktop [ "flavour" ]) "Unknown desktop setting";
    assert lib.assertMsg (keysOnly hardware [ "graphics" ]) "Unknown hardware setting";
    assert lib.assertMsg (keysOnly boot [ "loader" ]) "Unknown boot setting";
    assert lib.assertMsg (keysOnly install [
      "disk"
      "filesystem"
    ]) "Unknown install setting";
    assert lib.assertMsg (keysOnly locale [
      "locale"
      "timezone"
      "keymap"
    ]) "Unknown locale setting";
    assert lib.assertMsg (keysOnly user [ "name" ]) "Unknown user setting";
    assert lib.assertMsg (lib.all (
      key:
      builtins.elem key [
        "channel"
        "extra_modules"
      ]
    ) (builtins.attrNames (c.nix or { }))) "Unknown Nix setting";
    assert lib.assertMsg (lib.all
      (
        p:
        builtins.isString p
        && p != ""
        && !(lib.hasPrefix "/" p)
        && lib.hasSuffix ".nix" p
        && lib.all (part: part != "" && part != "." && part != "..") (lib.splitString "/" p)
      )
      (c.nix.extra_modules or [ ])
    ) "Extra modules must be relative .nix paths within the machine project";
    assert lib.assertMsg (c.schema_version == 1) "Unsupported Nox schema";
    assert lib.assertMsg (validName && builtins.stringLength c.name <= 63) "Invalid hostname";
    assert lib.assertMsg (builtins.elem c.profile [
      "server"
      "desktop"
      "gaming"
      "workspace"
      "recovery"
    ]) "Unknown profile";
    assert lib.assertMsg (builtins.elem c.target [
      "metal"
      "iso"
      "qcow2"
      "wsl"
      "oci"
    ]) "Unknown target";
    assert lib.assertMsg (builtins.elem c.desktop.flavour [
      "none"
      "hyprland"
      "niri"
    ]) "Unknown desktop flavour";
    assert lib.assertMsg (builtins.elem c.hardware.graphics [
      "auto"
      "amd"
      "intel"
      "nvidia-open"
      "nvidia-proprietary"
      "vm"
    ]) "Unknown graphics driver";
    assert lib.assertMsg (builtins.elem c.boot.loader [
      "systemd-boot"
      "grub-efi"
    ]) "Unknown bootloader";
    assert lib.assertMsg (builtins.elem c.install.filesystem [
      "btrfs"
      "ext4"
    ]) "Unknown filesystem";
    assert lib.assertMsg validDisk "Install disk must be a concrete /dev/disk/by-id/... device";
    assert lib.assertMsg (c.install.disk == null || c.target == "metal") "Install disk requires metal target";
    assert lib.assertMsg validUser "Invalid primary username";
    assert lib.assertMsg (
      !(builtins.elem c.target [
        "wsl"
        "oci"
      ])
      || c.desktop.flavour == "none"
    ) "WSL and OCI do not support desktop flavours";
    assert lib.assertMsg (lib.all (
      x:
      builtins.elem x [
        "desktop"
        "gaming"
        "storage"
        "shares"
        "apps"
        "virtualization"
        "development"
        "remote-management"
        "recovery"
      ]
    ) caps) "Unknown capability";
    assert lib.assertMsg (
      c.target != "wsl" || c.profile == "workspace"
    ) "WSL requires workspace profile";
    assert lib.assertMsg (
      c.target != "oci" || (c.profile == "workspace" && builtins.elem "development" caps)
    ) "OCI requires workspace/development";
    assert lib.assertMsg (
      c.profile != "gaming" || builtins.elem "gaming" caps
    ) "Gaming capability required";
    assert lib.assertMsg (
      c.profile != "recovery" || builtins.elem "recovery" caps
    ) "Recovery capability required";
    assert lib.assertMsg (
      (c.nix.channel or "nixos-26.05") == "nixos-26.05"
    ) "Change the flake input and schema together to change channels";
    assert lib.assertMsg (
      c.target != "oci"
      || lib.all (
        cap:
        builtins.elem cap [
          "development"
          "storage"
          "shares"
        ]
      ) caps
    ) "OCI supports userspace capabilities only";
    c;
  mkSystem =
    {
      configFile,
      system ? "x86_64-linux",
      extraModules ? [ ],
    }:
    let
      c = readConfig configFile;
    in
    inputs.nixpkgs.lib.nixosSystem {
      inherit system;
      specialArgs = { inherit inputs; };
      modules = [
        ../modules/core
        ../modules/boot
        ../modules/desktop
        ../modules/gaming
        ../modules/hardware
        ../modules/user
        ../profiles/${c.profile}.nix
        ../targets/${c.target}.nix
        ../capabilities
        ({ ... }: {
          networking.hostName = c.name;
          environment.systemPackages = [ inputs.self.packages.${system}.noxctl ];
          nox = {
            capabilities = c.capabilities or [ ];
            desktop.flavour = c.desktop.flavour;
            hardware.graphics = c.hardware.graphics;
            boot.loader = c.boot.loader;
            install = {
              inherit (c.install) disk filesystem;
            };
            locale = {
              inherit (c.locale) locale timezone keymap;
            };
            user.name = c.user.name;
          };
          nix.settings.experimental-features = [
            "nix-command"
            "flakes"
          ];
          services.openssh.settings.PasswordAuthentication = false;
          services.openssh.settings.KbdInteractiveAuthentication = false;
        })
      ]
      ++ lib.optionals (c.target == "wsl") [ inputs.nixos-wsl.nixosModules.default ]
      ++ lib.optionals (c.target == "metal") [
        inputs.disko.nixosModules.disko
        ../modules/install
      ]
      ++ map (p: builtins.toPath "${toString (builtins.dirOf configFile)}/${p}") (
        c.nix.extra_modules or [ ]
      )
      ++ extraModules;
    };
  artifact =
    machine:
    let
      c = machine.config;
    in
    {
      metal = c.system.build.toplevel;
      iso = c.system.build.isoImage;
      qcow2 = c.system.build.noxQcow2;
      wsl = c.system.build.tarballBuilder;
      oci = c.system.build.noxOci;
    }
    .${c.nox.target};
in
{
  inherit readConfig mkSystem artifact;
}
