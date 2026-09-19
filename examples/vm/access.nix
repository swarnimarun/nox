{ ... }:
{
  # Disposable local VM only. Never use this module for a reachable server.
  users.users.nox = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    initialPassword = "nox";
  };
  services.openssh.settings.PasswordAuthentication = false;
}
