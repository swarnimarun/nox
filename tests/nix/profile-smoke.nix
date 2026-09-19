{ config, ... }:

{
  imports = [
    ../../nix/profiles/server.nix
    ../../nix/targets/metal.nix
  ];
  system.stateVersion = "26.05";
  assertions = [
    {
      assertion = config.nox.profile == "server";
      message = "server profile must set nox.profile to server";
    }
    {
      assertion = config.nox.target == "metal";
      message = "metal target must set nox.target to metal";
    }
  ];
}
