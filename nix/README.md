# Xremap Nix Config Splitter

A Nix library for splitting xremap configurations with embedded descriptions into two attribute sets:
1. **cleanConfig** - Ready to use with xremap (descriptions removed)
2. **dmenuBindings** - Flat list of bindings with metadata for dmenu/rofi

## Purpose

This enables maintaining a single source of truth for your xremap configuration in Nix with human-readable descriptions, then automatically generating both:
- A clean config for the xremap service
- A searchable menu data structure for dmenu/rofi to help discover keybindings

## Installation

Add to your NixOS or Home-Manager configuration:

```nix
{ lib, ... }:

let
  xremapSplitter = import /path/to/config-splitter.nix { inherit lib; };
in
  # ... use xremapSplitter
```

## Usage

### Basic Example

```nix
{ lib, ... }:

let
  splitter = import ./config-splitter.nix { inherit lib; };

  myConfig = {
    keymap = [
      {
        name = "Launch";
        mode = "default";
        remap = {
          "Super-t" = {
            launch = ["xterm"];
            description = "Open terminal";
          };
          "Super-b" = {
            launch = ["firefox"];
            description = "Open browser";
          };
        };
      }
    ];
  };

  result = splitter.splitConfig myConfig;
in {
  # Use clean config for xremap
  services.xremap.config = result.cleanConfig;

  # Export bindings for dmenu/rofi
  environment.etc."xremap-bindings.json".text =
    builtins.toJSON result.dmenuBindings;
}
```

### Complete Example with Home-Manager

```nix
{ config, lib, pkgs, ... }:

let
  xremapSplitter = import ./nix/config-splitter.nix { inherit lib; };

  # Your xremap config with descriptions
  xremapConfig = {
    default_mode = "default";

    modmap = [
      {
        name = "Global modifiers";
        remap = {
          "CapsLock" = "Esc";
          "Alt_L" = "Ctrl_L";
        };
      }
    ];

    keymap = [
      {
        name = "Launch applications";
        mode = "default";
        remap = {
          "Super-t" = {
            launch = ["${pkgs.alacritty}/bin/alacritty"];
            description = "Open terminal";
          };
          "Super-b" = {
            launch = ["${pkgs.firefox}/bin/firefox"];
            description = "Open browser";
          };
        };
      }

      {
        name = "Emacs bindings";
        application.not = ["Emacs" "Terminal"];
        remap = {
          "C-a" = {
            action = "Home";
            description = "Move to beginning of line";
          };
          "C-e" = {
            action = "End";
            description = "Move to end of line";
          };
        };
      }
    ];
  };

  splitResult = xremapSplitter.splitConfig xremapConfig;
in {
  # Configure xremap service
  services.xremap = {
    enable = true;
    withX11 = true;
    config = splitResult.cleanConfig;
  };

  # Export bindings for rofi
  home.file.".config/xremap/bindings.json".text =
    builtins.toJSON splitResult.dmenuBindings;

  # Or as formatted text
  home.file.".config/xremap/bindings.txt".text =
    xremapSplitter.formatBindingsForRofi splitResult.dmenuBindings;

  # Add rofi script to show keybindings
  home.packages = [
    (pkgs.writeShellScriptBin "show-keybindings" ''
      ${pkgs.jq}/bin/jq -r '.bindings[] | "\(.binding)\t\(.description // "No description")\t[\(.name // "General")]"' \
        ~/.config/xremap/bindings.json | \
        ${pkgs.coreutils}/bin/column -t -s $'\t' | \
        ${pkgs.rofi}/bin/rofi -dmenu -i -p "Search keybindings"
    '')
  ];
}
```

## Configuration Format

### Input Config with Descriptions

The input follows standard xremap Nix syntax with optional `description` fields:

```nix
{
  default_mode = "default";

  modmap = [
    {
      name = "Global";
      remap = {
        "CapsLock" = "Esc";  # No description - simple mapping
      };
    }
  ];

  keymap = [
    {
      name = "Launch apps";
      mode = "default";
      remap = {
        # Simple launch with description
        "Super-t" = {
          launch = ["xterm"];
          description = "Open terminal";
        };

        # Action wrapper with description
        "C-a" = {
          action = "Home";
          description = "Beginning of line";
        };

        # Complex action with description
        "C-space" = {
          action.set_mark = true;
          description = "Set mark (Emacs)";
        };

        # Nested remap with description
        "C-x" = {
          action.remap = {
            "C-c" = "Esc";
            "C-s" = "C-w";
          };
          action.timeout_millis = 1000;
          description = "Emacs C-x prefix";
        };
      };
    }
  ];
}
```

### Output: Clean Config

Descriptions and action wrappers are removed:

```nix
{
  default_mode = "default";

  modmap = [
    {
      name = "Global";
      remap = {
        "CapsLock" = "Esc";
      };
    }
  ];

  keymap = [
    {
      name = "Launch apps";
      mode = "default";
      remap = {
        "Super-t" = { launch = ["xterm"]; };
        "C-a" = "Home";  # Action wrapper removed
        "C-space" = { set_mark = true; };
        "C-x" = {
          remap = { "C-c" = "Esc"; "C-s" = "C-w"; };
          timeout_millis = 1000;
        };
      };
    }
  ];
}
```

### Output: Dmenu Bindings

A flat list with all metadata:

```nix
{
  bindings = [
    {
      binding = "CapsLock";
      action = "Esc";
      name = "Global";
    }
    {
      binding = "Super-t";
      action = { launch = ["xterm"]; };
      description = "Open terminal";
      mode = "default";
      name = "Launch apps";
    }
    {
      binding = "C-a";
      action = "Home";
      description = "Beginning of line";
      mode = "default";
      name = "Launch apps";
    }
    # ... more bindings
  ];
  count = 4;
}
```

## API Reference

### Main Functions

#### `splitConfig`

Split a config into clean and dmenu versions.

**Signature:** `AttrSet -> { cleanConfig :: AttrSet, dmenuBindings :: AttrSet }`

**Example:**
```nix
result = splitter.splitConfig myConfig;
# result.cleanConfig - for xremap
# result.dmenuBindings - for dmenu/rofi
```

### Utility Functions

#### `formatBindingsForRofi`

Format bindings as tab-separated text for rofi.

**Signature:** `AttrSet -> String`

**Example:**
```nix
text = splitter.formatBindingsForRofi result.dmenuBindings;
# Output: "C-a\tBeginning of line\tEmacs\tdefault\n..."
```

#### `filterByMode`

Filter bindings by mode.

**Signature:** `String -> AttrSet -> [AttrSet]`

**Example:**
```nix
normalBindings = splitter.filterByMode "normal" result.dmenuBindings;
```

#### `filterByApplication`

Filter bindings by application name.

**Signature:** `String -> AttrSet -> [AttrSet]`

**Example:**
```nix
# Get bindings for Google Chrome
chromeBindings = splitter.filterByApplication "Google-chrome" result.dmenuBindings;
```

#### `groupByName`

Group bindings by category (keymap name).

**Signature:** `AttrSet -> AttrSet`

**Example:**
```nix
grouped = splitter.groupByName result.dmenuBindings;
# grouped.Launch - all Launch category bindings
# grouped.Emacs - all Emacs category bindings
```

## Rofi/Dmenu Integration

### Basic Rofi Script

```bash
#!/usr/bin/env bash
jq -r '.bindings[] | "\(.binding)\t\(.description // "No description")"' \
  ~/.config/xremap/bindings.json | \
  column -t -s $'\t' | \
  rofi -dmenu -i -p "Keybindings"
```

### Advanced with Grouping

```bash
#!/usr/bin/env bash
jq -r '.bindings[] | "\(.name // "General"): \(.binding)\t\(.description // "")"' \
  ~/.config/xremap/bindings.json | \
  sort | \
  column -t -s $'\t' | \
  rofi -dmenu -i -p "Search keybindings" -theme-str 'window {width: 80%;}'
```

### Using Nix to Generate Script

```nix
{ pkgs, ... }:

let
  xremapBindingsScript = pkgs.writeShellScriptBin "xremap-bindings" ''
    ${pkgs.jq}/bin/jq -r '.bindings[] |
      "\(.binding)\t|\t\(.description // "No description")\t|\t[\(.name // "General")]"' \
      ~/.config/xremap/bindings.json | \
      ${pkgs.coreutils}/bin/column -t -s $'\t' | \
      ${pkgs.rofi}/bin/rofi -dmenu -i \
        -p "Search keybindings" \
        -theme-str 'window {width: 80%;}' \
        -theme-str 'listview {lines: 20;}'
  '';
in {
  home.packages = [ xremapBindingsScript ];
}
```

Then bind it to a key:

```nix
services.xremap.config.keymap = [{
  remap = {
    "Super-slash" = {
      launch = ["xremap-bindings"];
      description = "Show all keybindings";
    };
  };
}];
```

## Features

### Supported xremap Elements

- ✅ Modmaps and keymaps
- ✅ Application/window/device filters
- ✅ Modes (default, custom modes)
- ✅ Launch commands
- ✅ Nested remaps (key sequences)
- ✅ Multi-purpose keys (held/alone)
- ✅ Set mark/with mark (Emacs-style)
- ✅ Mode switching
- ✅ All keymap actions
- ✅ Complex nested structures

### Metadata Preservation

All metadata is preserved in dmenu bindings:
- `name` - Keymap/modmap category
- `mode` - Active mode(s)
- `application` - Application filter
- `window` - Window title filter
- `device` - Device filter
- `exact_match` - Exact modifier matching

## Testing

The implementation has been tested with 10+ test cases covering:

1. Simple keymaps without descriptions
2. Keymaps with descriptions
3. Modmap processing
4. Application/window/device filters
5. Mixed modmap and keymap
6. Complex action types
7. Empty configs
8. Multiple modes
9. Utility functions
10. Application filtering

Run verification:

```bash
python3 nix/verify-logic.py
```

Expected output:
```
==================================================
NIX CONFIG SPLITTER - LOGIC VERIFICATION
==================================================

=== Test 1: Simple keymap ===
✓ PASS

=== Test 2: With descriptions ===
✓ PASS

... (7 more tests)

==================================================
Results: 7 passed, 0 failed
==================================================

🎉 All tests passed!
```

## Example: Full NixOS Configuration

See `example-config.nix` for a complete example with:
- Application launchers
- Emacs-like bindings
- Window management
- Key sequences (Emacs C-x prefix)
- Vim-like navigation modes
- Mode switching

## Troubleshooting

### Description not being removed

Make sure your description is at the same level as the action:

```nix
# ✓ Correct
"C-a" = {
  action = "Home";
  description = "Move home";
};

# ✗ Wrong - description nested inside action
"C-a" = {
  action = {
    key = "Home";
    description = "Move home";  # This won't work
  };
};
```

### Action wrapper not being removed

The splitter automatically unwraps the `action` field when only `action` and `description` are present:

```nix
# Input
"C-a" = {
  action = "Home";
  description = "Move home";
};

# Output (clean config)
"C-a" = "Home";  # Action unwrapped!
```

### Null values in dmenu output

Null values are automatically filtered from the dmenu bindings. Only non-null metadata is included.

## Related

- [config-splitter.nix](./config-splitter.nix) - Main implementation
- [config-splitter-tests.nix](./config-splitter-tests.nix) - Nix-native tests
- [example-config.nix](./example-config.nix) - Example configuration
- [verify-logic.py](./verify-logic.py) - Python verification script
- [manual-test.md](./manual-test.md) - Manual testing guide

## License

Same as xremap (MIT)
