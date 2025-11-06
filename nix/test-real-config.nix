{ lib, pkgs, inputs, system, ... }:

# Real user config with descriptions added for testing the splitter
let
  splitter = import ./config-splitter.nix { inherit lib; };

  # Your actual config with descriptions added
  configWithDescriptions = {
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
        name = "main remaps";
        remap = {
          super-m = {
            remap = {
              super-l = {
                launch = [
                  "${pkgs.uwsm}/bin/uwsm" "app" "--"
                  "${pkgs.hyprland}/bin/hyprctl" "dispatch" "--" "exec"
                  "${pkgs.kitty}/bin/kitty"
                ];
                description = "Launch Kitty terminal";
              };
              super-f = {
                launch = [
                  "${pkgs.uwsm}/bin/uwsm" "app" "--"
                  "${pkgs.hyprland}/bin/hyprctl" "dispatch" "--" "exec"
                  "${pkgs.firefox}/bin/firefox"
                ];
                description = "Launch Firefox browser";
              };
              super-e = {
                launch = [
                  "${pkgs.uwsm}/bin/uwsm" "app" "--"
                  "${pkgs.hyprland}/bin/hyprctl" "dispatch" "--" "exec"
                  "${pkgs.kitty}/bin/kitty"
                  "${pkgs.ranger}/bin/ranger"
                ];
                description = "Launch Ranger file manager";
              };
              super-o = {
                launch = [
                  "${pkgs.uwsm}/bin/uwsm" "app" "--"
                  "${pkgs.hyprland}/bin/hyprctl" "dispatch" "--" "exec"
                  "${pkgs.kitty}/bin/kitty"
                  "${pkgs.btop}/bin/btop"
                ];
                description = "Launch btop system monitor";
              };
              super-m = {
                launch = [
                  "${pkgs.rofi}/bin/rofi" "-show" "drun"
                  "-theme-str" "window {width: 20%;}"
                ];
                description = "Show application launcher (Rofi)";
              };
              super-k = {
                launch = [
                  "${inputs.rofi-switch-rust.packages.${system}.default}/bin/quick-start"
                ];
                description = "Quick start menu";
              };
              super-i = {
                launch = [
                  "${pkgs.uwsm}/bin/uwsm" "app" "--"
                  "${pkgs.bitwarden-desktop}/bin/bitwarden"
                ];
                description = "Launch Bitwarden password manager";
              };
            };
          };

          super-s = {
            remap = {
              super-e = {
                launch = [
                  "${pkgs.bash}/bin/sh" "-c"
                  ''
                    ${pkgs.wl-clipboard}/bin/wl-paste --type image/png |
                    ${pkgs.swappy}/bin/swappy -f -
                  ''
                ];
                description = "Edit clipboard screenshot";
              };
              super-d = {
                launch = [
                  "${pkgs.bash}/bin/sh" "-c"
                  ''
                    ${pkgs.grim}/bin/grim -g "$(${pkgs.slurp}/bin/slurp -w 0)" - |
                    ${pkgs.swappy}/bin/swappy -f -
                  ''
                ];
                description = "Take selection screenshot";
              };
            };
          };

          super-u = {
            remap = {
              super-f = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "history-pop" ];
                description = "Show notification history";
              };
              super-d = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "close" ];
                description = "Close current notification";
              };
              super-s = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "close-all" ];
                description = "Close all notifications";
              };
              super-c = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "action" ];
                description = "Trigger notification action";
              };
              super-e = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "context" ];
                description = "Show notification context menu";
              };
              super-t = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "set-paused" "toggle" ];
                description = "Toggle notifications pause";
              };
              super-i = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "set-paused" "true" ];
                description = "Pause notifications";
              };
              super-o = {
                launch = [ "${pkgs.dunst}/bin/dunstctl" "set-paused" "false" ];
                description = "Resume notifications";
              };
            };
          };

          super-f = {
            remap = {
              super-u = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "1"];
                description = "Switch to workspace 1";
              };
              super-i = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "2"];
                description = "Switch to workspace 2";
              };
              super-o = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "3"];
                description = "Switch to workspace 3";
              };
              super-p = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "4"];
                description = "Switch to workspace 4";
              };
              super-j = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "5"];
                description = "Switch to workspace 5";
              };
              super-k = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "6"];
                description = "Switch to workspace 6";
              };
              super-l = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "7"];
                description = "Switch to workspace 7";
              };
              super-semicolon = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "8"];
                description = "Switch to workspace 8";
              };
              super-m = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "9"];
                description = "Switch to workspace 9";
              };
              super-comma = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "10"];
                description = "Switch to workspace 10";
              };
            };
          };

          super-e = {
            remap = {
              super-u = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "1"];
                description = "Move window to workspace 1 (silent)";
              };
              super-i = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "2"];
                description = "Move window to workspace 2 (silent)";
              };
              super-o = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "3"];
                description = "Move window to workspace 3 (silent)";
              };
              super-p = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "4"];
                description = "Move window to workspace 4 (silent)";
              };
              super-j = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "5"];
                description = "Move window to workspace 5 (silent)";
              };
              super-k = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "6"];
                description = "Move window to workspace 6 (silent)";
              };
              super-l = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "7"];
                description = "Move window to workspace 7 (silent)";
              };
              super-semicolon = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "8"];
                description = "Move window to workspace 8 (silent)";
              };
              super-m = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "9"];
                description = "Move window to workspace 9 (silent)";
              };
              super-comma = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspacesilent" "10"];
                description = "Move window to workspace 10 (silent)";
              };
            };
          };

          super-v = {
            remap = {
              super-u = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "1"];
                description = "Move window to workspace 1 (follow)";
              };
              super-i = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "2"];
                description = "Move window to workspace 2 (follow)";
              };
              super-o = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "3"];
                description = "Move window to workspace 3 (follow)";
              };
              super-p = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "4"];
                description = "Move window to workspace 4 (follow)";
              };
              super-j = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "5"];
                description = "Move window to workspace 5 (follow)";
              };
              super-k = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "6"];
                description = "Move window to workspace 6 (follow)";
              };
              super-l = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "7"];
                description = "Move window to workspace 7 (follow)";
              };
              super-semicolon = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "8"];
                description = "Move window to workspace 8 (follow)";
              };
              super-m = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "9"];
                description = "Move window to workspace 9 (follow)";
              };
              super-comma = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "movetoworkspace" "10"];
                description = "Move window to workspace 10 (follow)";
              };
            };
          };

          super-d = {
            remap = {
              super-c = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "killactive" ];
                description = "Close active window";
              };
              super-j = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "swapnext" ];
                description = "Swap with next window";
              };
              super-k = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "swapnext" "prev" ];
                description = "Swap with previous window";
              };
              super-u = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "togglefloating" ];
                description = "Toggle floating mode";
              };
              super-f = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "fullscreen" ];
                description = "Toggle fullscreen";
              };
              super-comma = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "fullscreenstate" "1" ];
                description = "Maximize window";
              };
              super-m = {
                launch = [
                  "${pkgs.hyprland}/bin/hyprctl" "dispatch"
                  "movewindow" "mon:+1"
                ];
                description = "Move window to next monitor";
              };
              super-p = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "swapactiveworkspaces" "0" "1" ];
                description = "Swap workspaces between monitors";
              };
              super-t = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "layoutmsg" "togglesplit" ];
                description = "Toggle split direction";
              };
              super-g = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "focusurgentorlast" ];
                description = "Focus urgent or last window";
              };
              super-s = {
                launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "layoutmsg" "swapsplit" ];
                description = "Swap split";
              };
            };
          };

          super-comma = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "focusmonitor" "+1" ];
            description = "Focus next monitor";
          };

          super-j = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "cyclenext" ];
            description = "Cycle to next window";
          };
          super-k = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "cyclenext" "prev" ];
            description = "Cycle to previous window";
          };
          super-n = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "r+1"];
            description = "Next workspace";
          };
          super-p = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "r-1" ];
            description = "Previous workspace";
          };
          super-semicolon = {
            launch = [ "${pkgs.hyprland}/bin/hyprctl" "dispatch" "workspace" "previous_per_monitor" ];
            description = "Previous workspace on current monitor";
          };

          super-y = {
            remap = {
              super-s = {
                launch = [ "${pkgs.playerctl}/bin/playerctl" "play-pause" ];
                description = "Play/Pause media";
              };
              super-d = {
                launch = [ "${pkgs.playerctl}/bin/playerctl" "next" ];
                description = "Next track";
              };
              super-f = {
                launch = [ "${pkgs.playerctl}/bin/playerctl" "previous" ];
                description = "Previous track";
              };
              super-e = {
                launch = [ "${pkgs.wireplumber}/bin/wpctl" "set-mute" "@DEFAULT_AUDIO_SINK@" "toggle" ];
                description = "Toggle mute";
              };
            };
          };

          super-i = {
            launch = [ "${pkgs.wireplumber}/bin/wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "5%-" ];
            description = "Decrease volume";
          };
          super-o = {
            launch = [ "${pkgs.wireplumber}/bin/wpctl" "set-volume" "@DEFAULT_AUDIO_SINK@" "5%+" ];
            description = "Increase volume";
          };
        };
      }

      {
        name = "to normal";
        remap = {
          super-space = {
            action.set_mode = "normal";
            description = "Enter normal mode (Vim-like)";
          };
        };
        mode = "default";
      }

      {
        name = "to default";
        remap = {
          super-space = {
            action.set_mode = "default";
            description = "Return to default mode";
          };
        };
        mode = "normal";
      }

      {
        mode = "normal";
        name = "Normal mode window management";
        remap = {
          shift-h = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "moveactive"
              "-100" "0"
            ];
            description = "Move window left";
          };
          shift-j = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "moveactive"
              "0" "100"
            ];
            description = "Move window down";
          };
          shift-k = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "moveactive"
              "0" "-100"
            ];
            description = "Move window up";
          };
          shift-l = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "moveactive"
              "100" "0"
            ];
            description = "Move window right";
          };
          h = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "resizeactive"
              "-100" "0"
            ];
            description = "Resize window left";
          };
          j = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "resizeactive"
              "0" "100"
            ];
            description = "Resize window down";
          };
          k = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "resizeactive"
              "0" "-100"
            ];
            description = "Resize window up";
          };
          l = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "resizeactive"
              "100" "0"
            ];
            description = "Resize window right";
          };
          c = {
            launch = [
              "${pkgs.hyprland}/bin/hyprctl"
              "dispatch"
              "centerwindow"
            ];
            description = "Center window";
          };
          semicolon = {
            launch = [
              "${pkgs.brightnessctl}/bin/brightnessctl"
              "set"
              "10%+"
            ];
            description = "Increase brightness";
          };
          comma = {
            launch = [
              "${pkgs.lua}/bin/lua"
              "-e"
              ''
                local handle = io.popen("${pkgs.brightnessctl}/bin/brightnessctl get")
                local current = tonumber(handle:read("*a"))
                handle:close()

                local handle_max = io.popen("${pkgs.brightnessctl}/bin/brightnessctl max")
                local max = tonumber(handle_max:read("*a"))
                handle_max:close()

                local current_percent = (current / max) * 100

                if current_percent > 15 then
                  os.execute("${pkgs.brightnessctl}/bin/brightnessctl set 10%-")
                elseif current_percent > 10 then
                  os.execute("${pkgs.brightnessctl}/bin/brightnessctl set 10%")
                end
              ''
            ];
            description = "Decrease brightness (smart)";
          };
          "shift-semicolon" = {
            launch = [
              "${pkgs.brightnessctl}/bin/brightnessctl"
              "set"
              "100%"
            ];
            description = "Set brightness to 100%";
          };
          "shift-comma" = {
            launch = [
              "${pkgs.brightnessctl}/bin/brightnessctl"
              "set"
              "10%"
            ];
            description = "Set brightness to 10%";
          };
        };
      }

      {
        name = "firefox remaps";
        remap = {
          super-b = {
            action = [ "c-l" "shift-5" "space" ];
            description = "Jump to search in Firefox";
          };
          super-c = {
            remap = {
              super-c = {
                action = "c-alt-z";
                description = "Reader mode";
              };
              super-u = {
                action = "alt-1";
                description = "Switch to tab 1";
              };
              super-i = {
                action = "alt-2";
                description = "Switch to tab 2";
              };
              super-o = {
                action = "alt-3";
                description = "Switch to tab 3";
              };
              super-p = {
                action = "alt-4";
                description = "Switch to tab 4";
              };
              super-leftbrace = {
                action = "alt-5";
                description = "Switch to tab 5";
              };
              super-j = {
                action = "alt-6";
                description = "Switch to tab 6";
              };
              super-k = {
                action = "alt-7";
                description = "Switch to tab 7";
              };
              super-l = {
                action = "alt-8";
                description = "Switch to tab 8";
              };
              super-semicolon = {
                action = "alt-9";
                description = "Switch to last tab";
              };
            };
          };
          super-z = {
            action = "c-shift-tab";
            description = "Previous tab";
          };
          super-x = {
            action = "c-tab";
            description = "Next tab";
          };
        };
        application = {
          "only" = "firefox";
        };
      }
    ];
  };

  # Split the config
  result = splitter.splitConfig configWithDescriptions;

in {
  # Output for inspection
  inherit result;

  # Clean config (ready for xremap)
  cleanConfig = result.cleanConfig;

  # Dmenu bindings
  dmenuBindings = result.dmenuBindings;

  # Binding count
  bindingCount = result.dmenuBindings.count;

  # Group by category
  groupedBindings = splitter.groupByName result.dmenuBindings;

  # Filter examples
  normalModeBindings = splitter.filterByMode "normal" result.dmenuBindings;
  firefoxBindings = splitter.filterByApplication "firefox" result.dmenuBindings;

  # Formatted for rofi
  rofiText = splitter.formatBindingsForRofi result.dmenuBindings;
}
