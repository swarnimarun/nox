{ lib, ... }:
{
  imports = [
    ../modules/core
    ../modules/desktop
    ../modules/gaming
  ];
  nox.profile = "gaming";
  nox.desktop.flavour = lib.mkDefault "hyprland";
  nox.capabilities = [
    "desktop"
    "gaming"
  ];
}
