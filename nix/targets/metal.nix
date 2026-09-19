{ lib, ... }:

{
  nox.target = "metal";

  # Initial evaluation defaults. Real hardware layouts belong in a host
  # module generated or composed with Disko before an install is attempted.
  fileSystems."/" = lib.mkDefault {
    device = "/dev/disk/by-label/nox";
    fsType = "ext4";
  };
  boot.loader.grub.devices = [ "/dev/sda" ];
}
