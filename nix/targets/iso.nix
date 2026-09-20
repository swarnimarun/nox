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
  # The upstream installation profile otherwise boots to a console-only target.
  services.displayManager.enable = lib.mkForce true;
  systemd.defaultUnit = lib.mkForce "graphical.target";
  systemd.services.greetd.wantedBy = lib.mkForce [ "graphical.target" ];
  environment.systemPackages = [
    inputs.self.packages.${pkgs.system}.graphical-installer
    pkgs.gparted
    pkgs.nixos-install-tools
    pkgs.pciutils
  ];
  systemd.services.nox-live-ready = {
    description = "Verify that the Nox live compositor and installer started";
    wantedBy = [ "graphical.target" ];
    after = [
      "NetworkManager.service"
      "greetd.service"
    ];
    path = [
      pkgs.coreutils
      pkgs.procps
      pkgs.systemd
    ];
    serviceConfig.Type = "oneshot";
    script = ''
      compositor=${if config.nox.desktop.flavour == "hyprland" then "Hyprland" else "niri"}
      for attempt in $(seq 1 180); do
        if systemctl --quiet is-active NetworkManager.service \
          && systemctl --quiet is-active greetd.service \
          && pgrep --exact "$compositor" >/dev/null \
          && pgrep --full '[n]ox-installer.py' >/dev/null; then
          break
        fi
        if [ "$attempt" -eq 180 ]; then
          echo "Nox live session did not start compositor=$compositor and nox-installer" >&2
          systemctl --no-pager status NetworkManager.service greetd.service >&2 || true
          ps aux >&2
          exit 1
        fi
        sleep 1
      done
      message="NOX_LIVE_READY flavour=${config.nox.desktop.flavour}"
      echo "$message"
      if [ -w /dev/ttyS0 ]; then
        echo "$message" > /dev/ttyS0
      fi
    '';
  };
}
