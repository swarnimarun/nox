{
  config,
  lib,
  ...
}:

let
  username = config.nox.user.name;
  live = config.nox.target == "iso";
in
{
  users.users.${username} = {
    isNormalUser = true;
    description = "Nox user";
    extraGroups = [
      "audio"
      "input"
      "networkmanager"
      "video"
      "wheel"
    ];
  }
  // lib.optionalAttrs live {
    initialPassword = "nox";
  }
  // lib.optionalAttrs (!live) {
    initialHashedPassword = "!";
  };

  security.sudo.wheelNeedsPassword = lib.mkDefault (!live);
}
