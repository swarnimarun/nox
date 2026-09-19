{ lib, ... }:

{
  imports = [ ../modules/core ];

  nox.profile = "server";
  nox.capabilities = [ "storage" "shares" "apps" "virtualization" "remote-management" ];

  services.openssh.enable = lib.mkDefault true;
}

