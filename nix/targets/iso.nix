{ modulesPath, lib, ... }:
{
  imports = [ (modulesPath + "/installer/cd-dvd/installation-cd-minimal.nix") ];
  nox.target = "iso";
  image.baseName = lib.mkForce "nox";
}
