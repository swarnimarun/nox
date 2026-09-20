{
  config,
  lib,
  ...
}:

let
  driver = config.nox.hardware.graphics;
  graphical = config.nox.desktop.flavour != "none";
  nvidia = builtins.elem driver [
    "nvidia-open"
    "nvidia-proprietary"
  ];
in
{
  config = lib.mkMerge [
    (lib.mkIf graphical {
      hardware.graphics.enable = true;
    })
    (lib.mkIf (driver == "amd") {
      hardware.graphics.enable = true;
      boot.initrd.kernelModules = [ "amdgpu" ];
    })
    (lib.mkIf (driver == "intel") {
      hardware.graphics.enable = true;
      boot.initrd.kernelModules = [ "i915" ];
    })
    (lib.mkIf nvidia {
      nixpkgs.config.allowUnfree = true;
      services.xserver.videoDrivers = [ "nvidia" ];
      hardware.nvidia = {
        modesetting.enable = true;
        open = driver == "nvidia-open";
        package = config.boot.kernelPackages.nvidiaPackages.stable;
      };
    })
    (lib.mkIf (driver == "vm") {
      hardware.graphics.enable = true;
      services.qemuGuest.enable = true;
    })
  ];
}
