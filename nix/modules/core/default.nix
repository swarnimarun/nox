{ lib, pkgs, ... }:

{
  options.nox = {
    profile = lib.mkOption {
      type = lib.types.enum [
        "server"
        "desktop"
        "gaming"
        "workspace"
        "recovery"
      ];
      default = "server";
      description = "The user-facing purpose preset for this machine.";
    };

    target = lib.mkOption {
      type = lib.types.enum [
        "metal"
        "iso"
        "qcow2"
        "wsl"
        "oci"
      ];
      default = "metal";
      description = "The runtime or artifact target for this machine.";
    };

    capabilities = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Composable capabilities selected by a profile or operator.";
    };
  };

  config = {
    networking.hostName = lib.mkDefault "nox";
    environment.systemPackages = [ pkgs.git ];
    system.stateVersion = lib.mkDefault "26.05";
  };
}
