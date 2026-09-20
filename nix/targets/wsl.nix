{ lib, pkgs, ... }:
{
  nox.target = "wsl";
  wsl.enable = true;
  wsl.defaultUser = "nox";

  # Keep the WSL guest useful as a complete development distribution instead
  # of exposing only the upstream image builder.
  programs.bash.completion.enable = true;
  programs.command-not-found.enable = true;
  environment.systemPackages = with pkgs; [
    file
    openssh
    ripgrep
    unzip
    zip
  ];

  users.users.nox = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
  };

  security.sudo.wheelNeedsPassword = lib.mkDefault false;
}
