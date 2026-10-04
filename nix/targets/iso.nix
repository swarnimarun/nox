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
    # Level 19 dominates build time; use the upstream fast-compression example.
    squashfsCompression = "zstd -Xcompression-level 6";
  };
  boot.kernelParams = [ "console=ttyS0" ];
  # The upstream installation profile otherwise boots to a console-only target.
  services.displayManager.enable = lib.mkForce true;
  systemd.defaultUnit = lib.mkForce "graphical.target";
  systemd.services.greetd.wantedBy = lib.mkForce [ "multi-user.target" ];
  environment.systemPackages = [
    inputs.self.packages.${pkgs.system}.graphical-installer
    pkgs.gparted
    pkgs.nixos-install-tools
    pkgs.pciutils
  ];
  systemd.services.nox-live-ready = {
    description = "Verify that the Nox live compositor and installer started";
    wantedBy = [ "multi-user.target" ];
    requires = [
      "NetworkManager.service"
      "greetd.service"
    ];
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
      # Nix wrappers preserve argv[0] but can change the kernel's comm name.
      # Match the executable path with a boundary, not start-hyprland/niri-session.
      for attempt in $(seq 1 180); do
        if systemctl --quiet is-active NetworkManager.service \
          && systemctl --quiet is-active greetd.service \
          && pgrep --full "[/]bin/$compositor([[:space:]]|$)" >/dev/null \
          && pgrep --full '[n]ox-installer.py' >/dev/null; then
          break
        fi
        if [ "$attempt" -eq 180 ]; then
          {
            echo "Nox live session did not start compositor=$compositor and nox-installer"
            systemctl --no-pager status NetworkManager.service greetd.service || true
            ps -eo pid,comm,args
            echo "Compositor match:"
            pgrep --full "[/]bin/$compositor([[:space:]]|$)" || true
            echo "Installer match:"
            pgrep --full '[n]ox-installer.py' || true
            if [ -s /tmp/nox-installer.log ]; then
              echo "GTK installer output:"
              cat /tmp/nox-installer.log
            fi
            echo "NOX_LIVE_FAILURE flavour=${config.nox.desktop.flavour}"
          } 2>&1 | tee /dev/ttyS0 >&2
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
