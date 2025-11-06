{ lib ? import <nixpkgs/lib> }:

# Example xremap configuration with descriptions
# This demonstrates the input format for the config splitter

{
  default_mode = "default";

  modmap = [
    {
      name = "Global modifiers";
      remap = {
        "CapsLock" = "Esc";
        "Alt_L" = "Ctrl_L";
      };
    }

    {
      name = "Multi-purpose Space";
      remap = {
        "Space" = {
          held = "Shift_L";
          alone = "Space";
          alone_timeout_millis = 200;
        };
      };
    }
  ];

  keymap = [
    # Application launchers
    {
      name = "Launch applications";
      mode = "default";
      remap = {
        "Super-t" = {
          launch = ["xterm"];
          description = "Open terminal";
        };

        "Super-b" = {
          launch = ["firefox"];
          description = "Open web browser";
        };

        "Super-e" = {
          launch = ["emacs"];
          description = "Open Emacs editor";
        };

        "Super-f" = {
          launch = ["thunar"];
          description = "Open file manager";
        };
      };
    }

    # Emacs-like bindings (except in terminals and Emacs)
    {
      name = "Emacs bindings";
      mode = "default";
      application.not = ["Emacs" "Terminal" "Alacritty" "kitty"];
      remap = {
        "C-a" = {
          action = "Home";
          description = "Move to beginning of line";
        };

        "C-e" = {
          action = "End";
          description = "Move to end of line";
        };

        "C-f" = {
          action = "Right";
          description = "Move forward one character";
        };

        "C-b" = {
          action = "Left";
          description = "Move backward one character";
        };

        "C-n" = {
          action = "Down";
          description = "Move to next line";
        };

        "C-p" = {
          action = "Up";
          description = "Move to previous line";
        };

        "C-k" = {
          action = ["Shift-End" "C-x"];
          description = "Kill line (cut to end of line)";
        };

        "C-space" = {
          action.set_mark = true;
          description = "Set mark (start selection)";
        };

        "C-g" = {
          action.set_mark = false;
          description = "Cancel selection";
        };
      };
    }

    # Window management
    {
      name = "Window management";
      mode = "default";
      remap = {
        "Super-h" = {
          action = "Super-Left";
          description = "Tile window left";
        };

        "Super-l" = {
          action = "Super-Right";
          description = "Tile window right";
        };

        "Super-k" = {
          action = "Super-Up";
          description = "Maximize window";
        };

        "Super-j" = {
          action = "Super-Down";
          description = "Restore window";
        };

        "Super-q" = {
          action = "Alt-F4";
          description = "Close window";
        };
      };
    }

    # Key sequences (Emacs C-x prefix)
    {
      name = "Emacs C-x sequences";
      mode = "default";
      application.not = ["Emacs"];
      remap = {
        "C-x" = {
          action.remap = {
            "C-c" = "Alt-F4";
            "C-s" = "C-w";
            "C-f" = "C-o";
          };
          action.timeout_millis = 1000;
          description = "Emacs C-x prefix commands";
        };
      };
    }

    # Vim-like navigation (in normal mode)
    {
      name = "Vim navigation";
      mode = "normal";
      remap = {
        "h" = {
          action = "Left";
          description = "Move left";
        };

        "j" = {
          action = "Down";
          description = "Move down";
        };

        "k" = {
          action = "Up";
          description = "Move up";
        };

        "l" = {
          action = "Right";
          description = "Move right";
        };

        "w" = {
          action = "C-Right";
          description = "Move forward one word";
        };

        "b" = {
          action = "C-Left";
          description = "Move backward one word";
        };

        "i" = {
          action.set_mode = "insert";
          description = "Enter insert mode";
        };
      };
    }

    # Mode switching
    {
      name = "Mode switching";
      mode = "insert";
      remap = {
        "C-Esc" = {
          action.set_mode = "normal";
          description = "Enter normal mode (Vim-like)";
        };
      };
    }
  ];
}
