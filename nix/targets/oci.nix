{
  config,
  lib,
  pkgs,
  ...
}:
let
  root = pkgs.buildEnv {
    name = "nox-workspace-root";
    paths =
      config.environment.systemPackages
      ++ (with pkgs; [
        bashInteractive
        coreutils
        cacert
        nix
        shadow
        sudo
        util-linux
        procps
        findutils
        gnugrep
        gnused
        gnutar
        gzip
        which
        less
      ]);
    pathsToLink = [
      "/bin"
      "/etc"
      "/share"
    ];
    ignoreCollisions = true;
  };
in
{
  nox.target = "oci";
  boot.isContainer = true;
  system.build.noxOci = pkgs.dockerTools.buildLayeredImage {
    name = "nox-workspace";
    tag = "dev";
    includeNixDB = true;
    contents = [
      root
      pkgs.dockerTools.usrBinEnv
      pkgs.dockerTools.binSh
    ];
    extraCommands = ''
      mkdir -p etc tmp root home nix/var/nix/profiles nix/var/nix/gcroots
      chmod 1777 tmp
      echo 'root:x:0:0:root:/root:/bin/bash' > etc/passwd
      echo 'root:x:0:' > etc/group
      echo 'root:!:1::::::' > etc/shadow
      chmod 600 etc/shadow
      echo 'root ALL=(ALL:ALL) ALL' > etc/sudoers
      chmod 440 etc/sudoers
      echo 'NAME=Nox' > etc/os-release
      echo 'ID=nox' >> etc/os-release
      echo 'ID_LIKE=nixos' >> etc/os-release
      mkdir -p etc/nix
      echo 'experimental-features = nix-command flakes' > etc/nix/nix.conf
      echo 'sandbox = false' >> etc/nix/nix.conf
    '';
    config = {
      Cmd = [ "/bin/bash" ];
      Env = [
        "PATH=/bin"
        "USER=root"
        "HOME=/root"
        "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
        "NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
        "NIX_REMOTE=local"
      ];
      WorkingDir = "/root";
      Labels."org.opencontainers.image.description" =
        "Nox userspace workspace; host kernel, no NixOS systemd boot";
    };
  };
}
