{ ... }:
{
  nox.target = "wsl";
  wsl.enable = true;
  wsl.defaultUser = "nox";
  users.users.nox = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
  };
}
