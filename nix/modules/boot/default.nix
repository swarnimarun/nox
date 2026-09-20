{ config, lib, ... }:

let
  cfg = config.nox.boot;
  isMetal = config.nox.target == "metal";
in
{
  config = lib.mkIf isMetal (
    lib.mkMerge [
      {
        boot.loader.efi.canTouchEfiVariables = false;
      }
      (lib.mkIf (cfg.loader == "systemd-boot") {
        boot.loader.systemd-boot.enable = true;
      })
      (lib.mkIf (cfg.loader == "grub-efi") {
        boot.loader.grub = {
          enable = true;
          device = "nodev";
          efiSupport = true;
          efiInstallAsRemovable = true;
        };
      })
    ]
  );
}
