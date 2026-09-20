{ lib, ... }:

{
  imports = [
    ../modules/core
    ../modules/desktop
  ];

  nox.profile = "desktop";
  nox.capabilities = [
    "desktop"
    "development"
  ];

  nox.desktop.flavour = lib.mkDefault "hyprland";
}
