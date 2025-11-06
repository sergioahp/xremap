# Xremap Config Splitter

A utility to split xremap configuration files with embedded descriptions into two separate files:
1. **Clean config file** - Ready to use with xremap (without descriptions)
2. **Dmenu/Rofi file** - Contains keybindings with descriptions for use with menu launchers

## Purpose

This tool allows you to maintain a single source of truth for your xremap configuration that includes human-readable descriptions of what each keybinding does. You can then generate:

- A clean config file for xremap to use
- A searchable menu file for dmenu/rofi to help you discover and remember your keybindings

## Installation

Build the utility:

```bash
cargo build --release --bin xremap-config-splitter
```

The binary will be available at `target/release/xremap-config-splitter`.

## Usage

```bash
xremap-config-splitter INPUT_FILE \
  --config-output OUTPUT_CONFIG.yml \
  --dmenu-output DMENU_FILE.yml
```

### Example

```bash
xremap-config-splitter \
  example/config_with_descriptions.yml \
  -c ~/.config/xremap/config.yml \
  -d ~/.config/xremap/bindings.yml
```

This will:
- Read `example/config_with_descriptions.yml`
- Generate clean config at `~/.config/xremap/config.yml`
- Generate dmenu-ready file at `~/.config/xremap/bindings.yml`

## Input Format

The input YAML file uses standard xremap syntax with an additional `description` field:

```yaml
keymap:
  - name: Launch applications
    mode: default
    remap:
      Super-t:
        launch: [xterm]
        description: "Open terminal"

      Super-b:
        launch: [firefox]
        description: "Open web browser"

  - name: Emacs bindings
    application:
      not: [Emacs, Terminal]
    remap:
      C-a:
        action: Home
        description: "Move to beginning of line"

      C-e:
        action: End
        description: "Move to end of line"
```

### Description Syntax

For actions that are objects (like `launch`), add `description` as a sibling field:

```yaml
Super-t:
  launch: [xterm]
  description: "Open terminal"
```

For actions that need wrapping, use the `action` field:

```yaml
C-a:
  action: Home
  description: "Move to beginning of line"
```

## Output Formats

### Clean Config Output

The clean config removes all `description` fields and any `action` wrapper, producing valid xremap configuration:

```yaml
keymap:
  - name: Launch applications
    mode: default
    remap:
      Super-t:
        launch:
        - xterm
      Super-b:
        launch:
        - firefox
```

### Dmenu Output

The dmenu file contains a flat list of all bindings with metadata:

```yaml
bindings:
  - binding: Super-t
    description: Open terminal
    action:
      launch: [xterm]
    mode: default
    name: Launch applications

  - binding: C-a
    description: Move to beginning of line
    action: Home
    mode: default
    name: Emacs bindings
    application:
      not: [Emacs, Terminal]
```

Each binding includes:
- `binding` - The key combination (e.g., "C-a", "Super-t")
- `description` - Human-readable description (if provided)
- `action` - The action to perform
- `mode` - The mode(s) this binding is active in (if specified)
- `name` - The keymap/modmap name (if specified)
- `application` - Application filter (if specified)
- `window` - Window title filter (if specified)
- `device` - Device filter (if specified)
- `exact_match` - Exact modifier matching flag (if specified)

## Integration with Rofi/Dmenu

### Basic Rofi Integration

Create a script to show keybindings in rofi:

```bash
#!/bin/bash
# show-keybindings.sh

BINDINGS_FILE="$HOME/.config/xremap/bindings.yml"

# Parse YAML and format for rofi
yq -r '.bindings[] | "\(.binding)\t\(.description // "No description")\t\(.name // "")\t\(.mode // "default")"' \
  "$BINDINGS_FILE" | \
  column -t -s $'\t' | \
  rofi -dmenu -i -p "Keybindings" -no-custom
```

### Advanced Rofi Script

For a more sophisticated integration that groups by category and shows metadata:

```bash
#!/bin/bash
# keybinding-browser.sh

BINDINGS_FILE="$HOME/.config/xremap/bindings.yml"

# Format: BINDING | DESCRIPTION | CATEGORY | MODE
yq -r '.bindings[] |
  "\(.binding) | \(.description // "No description") | \(.name // "General") | \(.mode // "default")"' \
  "$BINDINGS_FILE" | \
  rofi -dmenu -i \
    -p "Search keybindings" \
    -format "s" \
    -theme-str 'window {width: 80%;}' \
    -theme-str 'listview {columns: 1; lines: 20;}' \
    -mesg "Search across all your xremap keybindings"
```

### Using fzf

If you prefer fzf:

```bash
#!/bin/bash
# keybinding-search.sh

BINDINGS_FILE="$HOME/.config/xremap/bindings.yml"

yq -r '.bindings[] |
  "\(.binding)\t\(.description // "No description")\t\(.name // "")"' \
  "$BINDINGS_FILE" | \
  column -t -s $'\t' | \
  fzf --header="Search Keybindings" \
      --preview-window=hidden \
      --bind='ctrl-/:toggle-preview'
```

## Features

### Supports All xremap Config Elements

- ✅ Modmaps and keymaps
- ✅ Application/window/device filters
- ✅ Modes (default, custom modes)
- ✅ Launch commands
- ✅ Nested remaps (key sequences)
- ✅ Multi-purpose keys (held/alone)
- ✅ Mark and selection (set_mark, with_mark)
- ✅ Mode switching (set_mode)
- ✅ All keymap actions (press, repeat, release, sleep, etc.)

### Preserves Metadata

All filtering and mode information is preserved in the dmenu output, allowing you to:
- Filter bindings by application
- Show which mode a binding is active in
- Group bindings by category (keymap name)

## Example Workflow

1. **Edit your config with descriptions:**
   ```bash
   vim ~/.config/xremap/config_with_descriptions.yml
   ```

2. **Generate clean and dmenu files:**
   ```bash
   xremap-config-splitter \
     ~/.config/xremap/config_with_descriptions.yml \
     -c ~/.config/xremap/config.yml \
     -d ~/.config/xremap/bindings.yml
   ```

3. **Reload xremap:**
   ```bash
   sudo systemctl restart xremap
   ```

4. **Browse your keybindings:**
   ```bash
   ./show-keybindings.sh
   ```

## Automation

You can automate the splitting process with a file watcher:

```bash
#!/bin/bash
# watch-config.sh

CONFIG_WITH_DESC="$HOME/.config/xremap/config_with_descriptions.yml"
CLEAN_CONFIG="$HOME/.config/xremap/config.yml"
DMENU_FILE="$HOME/.config/xremap/bindings.yml"

# Watch for changes and regenerate
while inotifywait -e modify "$CONFIG_WITH_DESC"; do
  xremap-config-splitter \
    "$CONFIG_WITH_DESC" \
    -c "$CLEAN_CONFIG" \
    -d "$DMENU_FILE"

  echo "✓ Config regenerated"

  # Optionally restart xremap
  # sudo systemctl restart xremap
done
```

## Testing

Run the included unit tests:

```bash
cargo test --bin xremap-config-splitter
```

Tests cover:
- Description extraction
- Keymap and modmap processing
- Application/window/device filter preservation
- Mode handling
- All action types

## See Also

- [CONFIG_GRAMMAR.md](CONFIG_GRAMMAR.md) - Formal grammar specification for xremap config files
- [xremap README](../README.md) - Main xremap documentation
- [example/config_with_descriptions.yml](../example/config_with_descriptions.yml) - Full example config

## Troubleshooting

### "Action has extra 'action' wrapper"

If your clean config has unnecessary nesting like:

```yaml
C-a:
  action: Home
```

This is correct! It will be processed properly by xremap. However, if you want cleaner output, use the description syntax without the action wrapper for simple actions in your source file.

### "Description not appearing in dmenu file"

Make sure your description is:
1. At the same level as `action` or `launch`
2. Spelled correctly as `description` (not `desc` or `Description`)
3. A string value in quotes

### "YAML parsing error"

Ensure your input file is valid YAML. You can validate it with:

```bash
yq eval . your_config.yml > /dev/null
```

## License

Same as xremap (MIT)
