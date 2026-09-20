{
  config,
  lib,
  pkgs,
  ...
}:

let
  enabled = config.nox.profile == "gaming" || builtins.elem "gaming" config.nox.capabilities;
in
{
  config = lib.mkIf enabled {
    nixpkgs.config.allowUnfree = true;
    hardware.graphics = {
      enable = true;
      enable32Bit = true;
    };
    programs = {
      steam = {
        enable = true;
        gamescopeSession.enable = true;
        protontricks.enable = true;
        extraCompatPackages = [ pkgs.proton-ge-bin ];
      };
      gamescope.enable = true;
      gamemode.enable = true;
    };
    environment.systemPackages = with pkgs; [
      gamescope
      lutris
      mangohud
      wineWow64Packages.staging
      winetricks
    ];
    boot.kernel.sysctl."vm.max_map_count" = 2147483642;
  };
}
