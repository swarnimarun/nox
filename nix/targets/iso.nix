{
  config,
  inputs,
  lib,
  modulesPath,
  pkgs,
  ...
}:
{
  imports = [ (modulesPath + "/installer/cd-dvd/installation-cd-graphical-base.nix") ];
  nox.target = "iso";
  image.baseName = lib.mkForce "nox-${config.nox.desktop.flavour}";
  isoImage = {
    makeEfiBootable = true;
    makeUsbBootable = true;
  };
  boot.kernelParams = [ "console=ttyS0" ];
  environment.systemPackages = [
    inputs.self.packages.${pkgs.system}.graphical-installer
    pkgs.gparted
    pkgs.nixos-install-tools
    pkgs.pciutils
  ];
  systemd.services.nox-live-ready = {
    description = "Record that the Nox live image reached graphical target";
    wantedBy = [ "graphical.target" ];
    after = [ "greetd.service" ];
    serviceConfig.Type = "oneshot";
    script = ''
      message="NOX_LIVE_READY flavour=${config.nox.desktop.flavour}"
      echo "$message"
      if [ -w /dev/ttyS0 ]; then
        echo "$message" > /dev/ttyS0
      fi
    '';
  };
}
