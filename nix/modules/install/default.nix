{
  config,
  lib,
  ...
}:

let
  cfg = config.nox.install;
  btrfs = {
    type = "btrfs";
    extraArgs = [ "-f" ];
    subvolumes = {
      "@root" = {
        mountpoint = "/";
        mountOptions = [
          "compress=zstd"
          "noatime"
        ];
      };
      "@home" = {
        mountpoint = "/home";
        mountOptions = [
          "compress=zstd"
          "noatime"
        ];
      };
      "@nix" = {
        mountpoint = "/nix";
        mountOptions = [
          "compress=zstd"
          "noatime"
        ];
      };
    };
  };
  ext4 = {
    type = "filesystem";
    format = "ext4";
    mountpoint = "/";
  };
in
{
  config = lib.mkMerge [
    {
      assertions = [
        {
          assertion = cfg.disk == null || config.nox.target == "metal";
          message = "A declared install disk requires the metal target";
        }
      ];
    }
    (lib.mkIf (config.nox.target == "metal" && cfg.disk != null) {
      disko.devices.disk.main = {
        type = "disk";
        device = cfg.disk;
        content = {
          type = "gpt";
          partitions = {
            ESP = {
              size = "1G";
              type = "EF00";
              content = {
                type = "filesystem";
                format = "vfat";
                mountpoint = "/boot";
                mountOptions = [ "umask=0077" ];
              };
            };
            root = {
              size = "100%";
              content = if cfg.filesystem == "btrfs" then btrfs else ext4;
            };
          };
        };
      };
    })
  ];
}
