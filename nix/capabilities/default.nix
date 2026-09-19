{
  config,
  lib,
  pkgs,
  ...
}:
let
  has = name: builtins.elem name config.nox.capabilities;
in
{
  config = lib.mkMerge [
    (lib.mkIf (has "development") {
      environment.systemPackages = with pkgs; [
        git
        curl
        jq
        gcc
        gnumake
      ];
    })
    (lib.mkIf (has "storage") {
      environment.systemPackages = with pkgs; [
        btrfs-progs
        smartmontools
      ];
    })
    (lib.mkIf (has "shares") {
      environment.systemPackages = with pkgs; [
        samba
        nfs-utils
      ];
    })
    (lib.mkIf (has "apps") { virtualisation.podman.enable = true; })
    (lib.mkIf (has "virtualization") { virtualisation.libvirtd.enable = true; })
    (lib.mkIf (has "remote-management") { services.openssh.enable = true; })
    (lib.mkIf (has "desktop") {
      services.xserver.enable = true;
      services.displayManager.sddm.enable = true;
      services.desktopManager.plasma6.enable = true;
    })
    (lib.mkIf (has "gaming") {
      programs.steam.enable = true;
      nixpkgs.config.allowUnfree = true;
    })
    (lib.mkIf (has "recovery") {
      environment.systemPackages = with pkgs; [
        cryptsetup
        smartmontools
      ];
    })
  ];
}
