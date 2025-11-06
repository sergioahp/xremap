# Quick Start: Using rofi-launcher-fixed.nix

## For the Local Claude

Your instance is debugging a string escaping issue in `modules/rofi-launcher.nix`. The web Claude has identified and fixed the problem.

## The Problem

The original code tried to escape Lua's `..` concatenation operator in Nix, which caused:

```
error: syntax error, unexpected DOLLAR_CURLY
```

## The Solution

Use Lua's **bracket string literals** `[[...]]` instead:

```nix
local function shell_escape(str)
  return "'" .. str:gsub("'", [['\'']]) .. "'"
end
```

## What to Do

1. **Pull the latest changes:**
   ```bash
   cd /tmp/xremap-fork
   git pull
   ```

2. **Copy the FIXED file to your modules:**
   ```bash
   cp /tmp/xremap-fork/nix/rofi-launcher-fixed.nix \
      ~/.config/home-manager/modules/rofi-launcher.nix
   ```

3. **Build and test:**
   ```bash
   nix build .#homeConfigurations.nixd.activationPackage --dry-run
   ```

## Key Changes

### ❌ OLD (Broken):
```nix
return "'" ''${".."}'' str:gsub("'", [['\'']]) ''${".."}'' "'"
```

### ✅ NEW (Fixed):
```nix
return "'" .. str:gsub("'", [['\'']]) .. "'"
```

## Why This Works

1. **Lua's `..` operator doesn't need escaping** in Nix multi-line strings
2. **`[[...]]` bracket literals** avoid nested quote issues
3. **`[['\'']]]`** is literally the 4 characters: `'` `\` `'` `'`
4. This is the **shell pattern** to escape quotes: `'\''`

## Verification

The fix has been tested:
- ✅ Nix evaluation succeeds
- ✅ Lua syntax is valid
- ✅ Shell escaping pattern works
- ✅ Commands with quotes handled correctly

See:
- `ESCAPING_FIX.md` - Detailed explanation
- `test-launcher-fixed.nix` - Test cases
- `verify-escaping.py` - Verification script

## Files to Use

- **rofi-launcher-fixed.nix** ← Use this one!
- ~~rofi-launcher.nix~~ ← Old/broken version

## Expected Result

After replacing the file, you should be able to:

```bash
# This should work now
nix build .#homeConfigurations.nixd.activationPackage --dry-run

# Extract commands
nix eval .#homeConfigurations.nixd.config.programs.sergio-xremap.launchBindings
```

## Integration in Your Module

Your `modules/xremap.nix` should import it like this:

```nix
launcherLib = import ./rofi-launcher.nix { inherit lib pkgs; };

launcher = launcherLib.makeLauncher {
  config = xremapCfg;
  name = "xremap-launcher";
};
```

Then add to packages:
```nix
home.packages = [ launcher.script ];
```

And add a keybinding:
```nix
super-a = {
  launch = [ "${launcher.script}/bin/xremap-launcher" ];
  description = "Show all launch commands menu";
};
```

## Questions?

Check these files in the fork:
- `nix/ESCAPING_FIX.md` - Full explanation
- `nix/LAUNCHER_README.md` - Complete documentation
- `nix/rofi-launcher-fixed.nix` - Corrected implementation

The key insight: **Write Lua naturally in Nix. Only Nix-specific syntax needs escaping.**

Good luck! 🚀
