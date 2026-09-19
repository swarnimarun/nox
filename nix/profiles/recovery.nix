{ pkgs, ... }:

{
  imports = [ ../modules/core ];

  nox.profile = "recovery";
  nox.capabilities = [ "recovery" "remote-management" ];

  environment.systemPackages = with pkgs; [
    cryptsetup
    disko
    smartmontools
  ];
  services.openssh.enable = true;
}

