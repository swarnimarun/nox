{
  config,
  lib,
  pkgs,
  ...
}:

{
  options.nox = {
    profile = lib.mkOption {
      type = lib.types.enum [
        "server"
        "desktop"
        "gaming"
        "workspace"
        "recovery"
      ];
      default = "server";
      description = "The user-facing purpose preset for this machine.";
    };

    target = lib.mkOption {
      type = lib.types.enum [
        "metal"
        "iso"
        "qcow2"
        "wsl"
        "oci"
      ];
      default = "metal";
      description = "The runtime or artifact target for this machine.";
    };

    capabilities = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Composable capabilities selected by a profile or operator.";
    };

    desktop.flavour = lib.mkOption {
      type = lib.types.enum [
        "none"
        "hyprland"
        "niri"
      ];
      default = "none";
      description = "Wayland compositor selected for desktop and gaming systems.";
    };

    hardware.graphics = lib.mkOption {
      type = lib.types.enum [
        "auto"
        "amd"
        "intel"
        "nvidia-open"
        "nvidia-proprietary"
        "vm"
      ];
      default = "auto";
      description = "Graphics driver policy for the selected machine.";
    };

    boot.loader = lib.mkOption {
      type = lib.types.enum [
        "systemd-boot"
        "grub-efi"
      ];
      default = "systemd-boot";
      description = "EFI bootloader used for an installed metal system.";
    };

    install = {
      disk = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Stable /dev/disk/by-id device selected for destructive installation.";
      };
      filesystem = lib.mkOption {
        type = lib.types.enum [
          "btrfs"
          "ext4"
        ];
        default = "btrfs";
        description = "Root filesystem created by Disko.";
      };
    };

    locale = {
      locale = lib.mkOption {
        type = lib.types.str;
        default = "en_US.UTF-8";
      };
      timezone = lib.mkOption {
        type = lib.types.str;
        default = "UTC";
      };
      keymap = lib.mkOption {
        type = lib.types.str;
        default = "us";
      };
    };

    user.name = lib.mkOption {
      type = lib.types.str;
      default = "nox";
      description = "Primary non-root account.";
    };
  };

  config = {
    networking.hostName = lib.mkDefault "nox";
    environment.systemPackages = [ pkgs.git ];
    time.timeZone = config.nox.locale.timezone;
    i18n.defaultLocale = config.nox.locale.locale;
    console.keyMap = config.nox.locale.keymap;
    system.stateVersion = lib.mkDefault "26.05";
    assertions = [
      {
        assertion =
          config.nox.install.disk == null
          || (
            lib.hasPrefix "/dev/disk/by-id/" config.nox.install.disk
            && config.nox.install.disk != "/dev/disk/by-id/"
            && !(lib.hasInfix ".." config.nox.install.disk)
            && !(lib.hasInfix "REPLACE" config.nox.install.disk)
          );
        message = "nox.install.disk must be a concrete /dev/disk/by-id/... device";
      }
    ];
  };
}
