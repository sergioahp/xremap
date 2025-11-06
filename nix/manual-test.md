# Manual Testing Guide for Nix Config Splitter

Since Nix installation is complex, this guide shows how to manually verify the config splitter logic.

## Test Structure

The `config-splitter.nix` exports several functions. The main one is `splitConfig`.

## Test 1: Simple Config

### Input:
```nix
{
  keymap = [
    {
      name = "Basic";
      remap = {
        "C-a" = "Home";
        "C-e" = "End";
      };
    }
  ];
}
```

### Expected Output:

**cleanConfig:**
```nix
{
  keymap = [
    {
      name = "Basic";
      remap = {
        "C-a" = "Home";
        "C-e" = "End";
      };
    }
  ];
}
```

**dmenuBindings:**
```nix
{
  bindings = [
    { binding = "C-a"; action = "Home"; name = "Basic"; }
    { binding = "C-e"; action = "End"; name = "Basic"; }
  ];
  count = 2;
}
```

## Test 2: With Descriptions

### Input:
```nix
{
  keymap = [
    {
      name = "Launch";
      mode = "default";
      remap = {
        "Super-t" = {
          launch = ["xterm"];
          description = "Open terminal";
        };
      };
    }
  ];
}
```

### Expected Output:

**cleanConfig:**
```nix
{
  keymap = [
    {
      name = "Launch";
      mode = "default";
      remap = {
        "Super-t" = {
          launch = ["xterm"];
          # description removed!
        };
      };
    }
  ];
}
```

**dmenuBindings:**
```nix
{
  bindings = [
    {
      binding = "Super-t";
      action = { launch = ["xterm"]; };
      description = "Open terminal";
      name = "Launch";
      mode = "default";
    }
  ];
  count = 1;
}
```

## Test 3: Application Filters

### Input:
```nix
{
  keymap = [
    {
      name = "Emacs";
      application.not = ["Emacs" "Terminal"];
      remap = {
        "C-a" = {
          action = "Home";
          description = "Beginning of line";
        };
      };
    }
  ];
}
```

### Expected Output:

**cleanConfig:**
```nix
{
  keymap = [
    {
      name = "Emacs";
      application.not = ["Emacs" "Terminal"];
      remap = {
        "C-a" = "Home";
      };
    }
  ];
}
```

**dmenuBindings:**
```nix
{
  bindings = [
    {
      binding = "C-a";
      action = "Home";
      description = "Beginning of line";
      name = "Emacs";
      application = { not = ["Emacs" "Terminal"]; };
    }
  ];
  count = 1;
}
```

## Logic Verification

### The splitter should:

1. **Extract descriptions** from action attribute sets
2. **Remove descriptions** from clean config
3. **Preserve all metadata** (name, mode, application, window, device)
4. **Flatten bindings** into a list for dmenu
5. **Count bindings** correctly

### Key Functions:

- `extractDescription` - Finds and removes description field
- `processRemapEntry` - Handles individual key->action pairs
- `processRemap` - Processes entire remap attribute set
- `processKeymapItem` - Processes full keymap/modmap items
- `splitConfig` - Main entry point

### Utility Functions:

- `filterByMode` - Get bindings for specific mode
- `filterByApplication` - Get bindings for specific app
- `groupByName` - Group bindings by category
- `formatBindingsForRofi` - Format for rofi display

## Expected Test Results

All 10 tests should pass:

1. ✓ Simple keymap without descriptions
2. ✓ Keymap with descriptions
3. ✓ Modmap processing
4. ✓ Application filters
5. ✓ Mixed modmap and keymap
6. ✓ Complex action types
7. ✓ Empty config
8. ✓ Multiple modes
9. ✓ Utility functions
10. ✓ Application filtering utility

## Usage in NixOS/Home-Manager

Once tested, use like this:

```nix
{ lib, ... }:

let
  splitter = import ./config-splitter.nix { inherit lib; };
  myConfig = import ./example-config.nix { inherit lib; };
  result = splitter.splitConfig myConfig;
in {
  # Use clean config for xremap service
  services.xremap.config = result.cleanConfig;

  # Export dmenu bindings as JSON
  environment.etc."xremap-bindings.json".text = builtins.toJSON result.dmenuBindings;

  # Or as formatted text for rofi
  environment.etc."xremap-bindings.txt".text = splitter.formatBindingsForRofi result.dmenuBindings;
}
```
