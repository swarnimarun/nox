{ lib, ... }:

{
  imports = [ ../modules/core ];

  nox.profile = "desktop";
  nox.capabilities = [
    "desktop"
    "development"
  ];

  services.xserver.enable = lib.mkDefault true;
  services.displayManager.sddm.enable = lib.mkDefault true;
  services.desktopManager.plasma6.enable = lib.mkDefault true;
}
