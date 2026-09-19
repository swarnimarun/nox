{ ... }:
{
  imports = [ ../modules/core ];
  nox.profile = "workspace";
  nox.capabilities = [ "development" ];
}
