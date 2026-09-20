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
  config = lib.mkMerge [
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
      };
    }
    (lib.mkIf live {
      security.sudo.wheelNeedsPassword = lib.mkForce false;
    })
  ];
}
