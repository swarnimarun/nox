{
  config,
  lib,
  pkgs,
  ...
}:

let
  flavour = config.nox.desktop.flavour;
  enabled = flavour != "none";
  hyprland = flavour == "hyprland";
  live = config.nox.target == "iso";
  installerHyprland = lib.optionalString live ''
    exec-once = nox-installer
  '';
  installerNiri = lib.optionalString live ''
    spawn-at-startup "nox-installer"
  '';
  waybarConfig = pkgs.writeText "nox-waybar.json" (
    builtins.toJSON {
      layer = "top";
      position = "top";
      modules-left = [ (if hyprland then "hyprland/workspaces" else "niri/workspaces") ];
      modules-center = [ "clock" ];
      modules-right = [
        "pulseaudio"
        "network"
        "battery"
        "tray"
      ];
      clock.format = "{:%Y-%m-%d %H:%M}";
      network.format-wifi = "{essid} {signalStrength}%";
      network.format-ethernet = "{ifname}";
      pulseaudio.format = "{volume}% {icon}";
    }
  );
  hyprlandConfig = pkgs.writeText "nox-hyprland.conf" ''
    monitor = , preferred, auto, 1
    $mainMod = SUPER
    input {
      kb_layout = ${config.nox.locale.keymap}
      follow_mouse = 1
    }
    general {
      gaps_in = 5
      gaps_out = 10
      border_size = 2
      layout = dwindle
    }
    decoration {
      rounding = 8
    }
    exec-once = waybar --config /etc/nox/waybar.json
    exec-once = nm-applet --indicator
    ${installerHyprland}
    bind = $mainMod, Return, exec, foot
    bind = $mainMod, D, exec, fuzzel
    bind = $mainMod SHIFT, Q, killactive
    bind = $mainMod SHIFT, E, exit
    bind = $mainMod, F, fullscreen
    bind = $mainMod, V, togglefloating
  '';
  niriConfig = pkgs.writeText "nox-niri.kdl" ''
    input {
      keyboard {
        xkb {
          layout "${config.nox.locale.keymap}"
        }
      }
    }
    layout {
      gaps 12
    }
    spawn-at-startup "waybar" "--config" "/etc/nox/waybar.json"
    spawn-at-startup "nm-applet" "--indicator"
    ${installerNiri}
    binds {
      Mod+Return { spawn "foot"; }
      Mod+D { spawn "fuzzel"; }
      Mod+Shift+Q { close-window; }
      Mod+Shift+E { quit; }
      Mod+F { fullscreen-window; }
      Mod+V { toggle-window-floating; }
    }
  '';
  sessionCommand =
    if hyprland then
      "start-hyprland --config /etc/nox/hyprland.conf"
    else
      "env NIRI_CONFIG=/etc/nox/niri.kdl niri-session";
in
{
  config = lib.mkIf enabled (
    lib.mkMerge [
      {
        assertions = [
          {
            assertion = builtins.elem config.nox.profile [
              "desktop"
              "gaming"
            ];
            message = "A desktop flavour requires the desktop or gaming profile";
          }
        ];

        programs = {
          dconf.enable = true;
          hyprland.enable = hyprland;
          niri.enable = !hyprland;
        };
        networking = {
          networkmanager.enable = true;
          wireless.enable = lib.mkForce false;
        };
        services = {
          dbus.enable = true;
          gnome.gnome-keyring.enable = true;
          greetd = {
            enable = true;
            settings.default_session = {
              command = "${pkgs.tuigreet}/bin/tuigreet --time --remember --remember-user-session --cmd '${sessionCommand}'";
              user = "greeter";
            };
          };
          pipewire = {
            enable = true;
            alsa = {
              enable = true;
              support32Bit = true;
            };
            pulse.enable = true;
          };
        };
        security = {
          polkit.enable = true;
          rtkit.enable = true;
        };
        xdg.portal = {
          enable = true;
          extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
        };
        environment = {
          etc = {
            "nox/hyprland.conf".source = hyprlandConfig;
            "nox/niri.kdl".source = niriConfig;
            "nox/waybar.json".source = waybarConfig;
          };
          sessionVariables = {
            MOZ_ENABLE_WAYLAND = "1";
            NIXOS_OZONE_WL = "1";
          };
          systemPackages = with pkgs; [
            foot
            fuzzel
            grim
            networkmanagerapplet
            pavucontrol
            slurp
            swaybg
            swaylock
            waybar
            wl-clipboard
          ];
        };
        fonts.packages = [ pkgs.nerd-fonts.jetbrains-mono ];
      }
      (lib.mkIf live {
        services.greetd.settings.initial_session = {
          command = sessionCommand;
          user = config.nox.user.name;
        };
      })
    ]
  );
}
