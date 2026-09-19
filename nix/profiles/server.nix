{ ... }:
{
  imports = [ ../modules/core ];
  nox.profile = "server";
  nox.capabilities = [ "remote-management" ];
}
