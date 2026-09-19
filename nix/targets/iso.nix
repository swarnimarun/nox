{ modulesPath, ... }:
{
  imports = [ (modulesPath + "/installer/cd-dvd/installation-cd-minimal.nix") ];
  nox.target = "iso";
  isoImage.isoBaseName = "nox";
}
