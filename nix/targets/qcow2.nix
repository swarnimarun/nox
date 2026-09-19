{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}:
{
  imports = [ (modulesPath + "/profiles/qemu-guest.nix") ];
  nox.target = "qcow2";
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
  boot.loader.grub = {
    enable = true;
    device = "nodev";
    efiSupport = true;
    efiInstallAsRemovable = true;
  };
  boot.kernelParams = [ "console=ttyS0" ];
  boot.growPartition = true;
  system.build.noxQcow2 = import (modulesPath + "/../lib/make-disk-image.nix") {
    inherit config lib pkgs;
    format = "qcow2";
    partitionTableType = "efi";
    diskSize = 12288;
  };
}
