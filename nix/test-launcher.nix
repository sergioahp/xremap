{ lib ? import <nixpkgs/lib>
, pkgs ? import <nixpkgs> {}
, inputs ? {}
, system ? builtins.currentSystem
}:

# Test the rofi launcher with real user config

let
  launcherLib = import ./rofi-launcher.nix { inherit lib pkgs; };

  # Simplified version of user's config (for testing)
  # In production, this would be your full config
  testConfig = {
    keypress_delay_ms = 20;

    modmap = [
      {
        name = "main modmaps";
        remap = {
          shift_r = "alt_l";
        };
      }
    ];

    keymap = [
      {
        name = "Application Launchers";
        remap = {
          # Nested remap - super-m prefix
          super-m = {
            remap = {
              super-l = {
                launch = [ "kitty" ];
                description = "Launch Kitty terminal";
              };
              super-f = {
                launch = [ "firefox" ];
                description = "Launch Firefox browser";
              };
              super-e = {
                launch = [ "kitty" "ranger" ];
                description = "Launch Ranger file manager";
              };
              super-o = {
                launch = [ "kitty" "btop" ];
                description = "Launch btop system monitor";
              };
              super-m = {
                launch = [ "rofi" "-show" "drun" "-theme-str" "window {width: 20%;}" ];
                description = "Show application launcher (Rofi)";
              };
            };
          };

          # Screenshot tools - super-s prefix
          super-s = {
            remap = {
              super-e = {
                launch = [ "sh" "-c" "wl-paste --type image/png | swappy -f -" ];
                description = "Edit clipboard screenshot";
              };
              super-d = {
                launch = [ "sh" "-c" "grim -g \"$(slurp -w 0)\" - | swappy -f -" ];
                description = "Take selection screenshot";
              };
            };
          };

          # Notification control - super-u prefix
          super-u = {
            remap = {
              super-f = {
                launch = [ "dunstctl" "history-pop" ];
                description = "Show notification history";
              };
              super-d = {
                launch = [ "dunstctl" "close" ];
                description = "Close current notification";
              };
              super-s = {
                launch = [ "dunstctl" "close-all" ];
                description = "Close all notifications";
              };
              super-t = {
                launch = [ "dunstctl" "set-paused" "toggle" ];
                description = "Toggle notifications pause";
              };
            };
          };

          # Direct bindings (not nested)
          super-comma = {
            launch = [ "hyprctl" "dispatch" "focusmonitor" "+1" ];
            description = "Focus next monitor";
          };

          super-j = {
            launch = [ "hyprctl" "dispatch" "cyclenext" ];
            description = "Cycle to next window";
          };

          super-k = {
            launch = [ "hyprctl" "dispatch" "cyclenext" "prev" ];
            description = "Cycle to previous window";
          };
        };
      }

      {
        name = "Mode Switching";
        mode = "default";
        remap = {
          super-space = {
            action.set_mode = "normal";
            description = "Enter normal mode";
          };
        };
      }

      {
        name = "Normal Mode Commands";
        mode = "normal";
        remap = {
          h = {
            launch = [ "hyprctl" "dispatch" "resizeactive" "-100" "0" ];
            description = "Resize window left";
          };
          j = {
            launch = [ "hyprctl" "dispatch" "resizeactive" "0" "100" ];
            description = "Resize window down";
          };
          c = {
            launch = [ "hyprctl" "dispatch" "centerwindow" ];
            description = "Center window";
          };
        };
      }
    ];
  };

  # Generate launcher
  launcher = launcherLib.makeLauncher {
    config = testConfig;
    name = "test-xremap-launcher";
  };

in {
  # The launcher script package
  inherit (launcher) script;

  # The extracted commands
  inherit (launcher) commands commandsJson commandsLua;

  # Statistics
  inherit (launcher) stats;

  # For inspection
  commandCount = launcher.stats.totalCommands;

  # Show sample commands
  sampleCommands = lib.take 5 launcher.commands;

  # Test: verify nested remaps are flattened correctly
  nestedRemapTest = let
    superMLCommands = lib.filter (c: lib.hasPrefix "super-m " c.keySequence) launcher.commands;
  in {
    found = builtins.length superMLCommands;
    expected = 5;  # super-m has 5 sub-bindings
    samples = lib.take 3 superMLCommands;
  };
}
