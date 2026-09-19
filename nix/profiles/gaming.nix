{ ... }:
{
  imports = [ ../modules/core ];
  nox.profile = "gaming";
  nox.capabilities = [ "desktop" "gaming" ];
}
