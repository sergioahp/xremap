# Xremap Configuration File Grammar

This document provides a formal grammar specification for xremap configuration files in Extended Backus-Naur Form (EBNF) notation.

## File Formats

Xremap supports both YAML and TOML formats. The format is auto-detected by file extension:
- `.yml`, `.yaml` → YAML format
- `.toml` → TOML format

## EBNF Notation

- `::=` defines a production rule
- `|` denotes alternation (OR)
- `[]` denotes optional elements
- `{}` denotes zero or more repetitions
- `()` denotes grouping
- `""` denotes literal strings
- `<>` denotes non-terminal symbols

---

## Root Grammar

```ebnf
<config> ::= {<config-field>}

<config-field> ::= <modmap-field>
                 | <keymap-field>
                 | <default-mode-field>
                 | <virtual-modifiers-field>
                 | <keypress-delay-field>
                 | <enable-wheel-field>
                 | <shared-field>

<modmap-field> ::= "modmap:" <modmap-list>
<keymap-field> ::= "keymap:" <keymap-list>
<default-mode-field> ::= "default_mode:" <string>
<virtual-modifiers-field> ::= "virtual_modifiers:" <key-list>
<keypress-delay-field> ::= "keypress_delay_ms:" <unsigned-integer>
<enable-wheel-field> ::= "enable_wheel:" <boolean>
<shared-field> ::= "shared:" <any>
```

---

## Modmap Grammar

```ebnf
<modmap-list> ::= "[" {<modmap>} "]"

<modmap> ::= "{" {<modmap-property>} "}"

<modmap-property> ::= <name-property>
                    | <remap-property>
                    | <application-property>
                    | <window-property>
                    | <device-property>
                    | <mode-property>

<name-property> ::= "name:" <string>

<remap-property> ::= "remap:" "{" {<key> ":" <modmap-action>} "}"

<modmap-action> ::= <key>
                  | <key-list>
                  | <multi-purpose-key>
                  | <hook-action>

<multi-purpose-key> ::= "{"
                         "held:" <modmap-action-target> ","
                         "alone:" <modmap-action-target>
                         ["," "alone_timeout_millis:" <unsigned-integer>]
                         ["," "free_hold:" <boolean>]
                        "}"

<modmap-action-target> ::= <key>
                         | <key-list>

<hook-action> ::= "{"
                   ["skip_key_event:" <boolean> ","]
                   ["press:" <keymap-action-list> ","]
                   ["repeat:" <keymap-action-list> ","]
                   ["release:" <keymap-action-list>]
                  "}"
```

---

## Keymap Grammar

```ebnf
<keymap-list> ::= "[" {<keymap>} "]"

<keymap> ::= "{" {<keymap-property>} "}"

<keymap-property> ::= <name-property>
                    | <keymap-remap-property>
                    | <application-property>
                    | <window-property>
                    | <device-property>
                    | <mode-property>
                    | <exact-match-property>

<keymap-remap-property> ::= "remap:" "{" {<key-press> ":" <keymap-action>} "}"

<exact-match-property> ::= "exact_match:" <boolean>

<keymap-action> ::= <key-press>
                  | <keymap-action-list>
                  | <nested-remap>
                  | <launch-action>
                  | <set-mark-action>
                  | <with-mark-action>
                  | <set-mode-action>
                  | <escape-next-key-action>
                  | <sleep-action>
                  | <press-action>
                  | <repeat-action>
                  | <release-action>
                  | <null-action>

<keymap-action-list> ::= "[" {<keymap-action>} "]"

<nested-remap> ::= "{"
                    "remap:" "{" {<key-press> ":" <keymap-action>} "}"
                    ["," "timeout_millis:" <unsigned-integer>]
                    ["," "timeout_key:" (<key-press> | <keymap-action-list>)]
                   "}"

<launch-action> ::= "{" "launch:" <string-list> "}"

<set-mark-action> ::= "{" "set_mark:" <boolean> "}"

<with-mark-action> ::= "{" "with_mark:" <key-press> "}"

<set-mode-action> ::= "{" "set_mode:" <string> "}"

<escape-next-key-action> ::= "{" "escape_next_key:" <boolean> "}"

<sleep-action> ::= "{" "sleep:" <unsigned-integer> "}"

<press-action> ::= "{" "press:" <key> "}"

<repeat-action> ::= "{" "repeat:" <key> "}"

<release-action> ::= "{" "release:" <key> "}"

<null-action> ::= "null" | "[]"
```

---

## Application, Window, and Device Filters

```ebnf
<application-property> ::= "application:" <only-or-not>
<window-property> ::= "window:" <only-or-not>
<device-property> ::= "device:" <only-or-not>

<only-or-not> ::= <only-filter> | <not-filter>

<only-filter> ::= "{" "only:" <filter-value> "}"
<not-filter> ::= "{" "not:" <filter-value> "}"

<filter-value> ::= <string> | <string-list> | <regex>
```

---

## Mode Grammar

```ebnf
<mode-property> ::= "mode:" <mode-value>

<mode-value> ::= <string>
               | <string-list>
```

---

## Key and Key Press Grammar

```ebnf
<key> ::= <key-name>
        | <evdev-name>

<key-name> ::= <letter>
             | <digit>
             | <special-key>
             | <modifier-key>
             | <relative-event>
             | "ANY"

<evdev-name> ::= "KEY_" <uppercase-identifier>

<special-key> ::= "Enter" | "Space" | "Tab" | "Esc" | "Escape"
                | "Backspace" | "Delete" | "Insert" | "Home" | "End"
                | "PageUp" | "PageDown" | "Up" | "Down" | "Left" | "Right"
                | "F1" | "F2" | ... | "F12"
                | "CapsLock" | "ScrollLock" | "NumLock"
                | "PrintScreen" | "Pause" | "Menu"
                | ... (see evdev key names)

<modifier-key> ::= "Shift" | "Shift_L" | "Shift_R"
                 | "Control" | "Ctrl" | "C" | "Control_L" | "Control_R"
                 | "Alt" | "M" | "Alt_L" | "Alt_R"
                 | "Super" | "Win" | "Windows" | "Super_L" | "Super_R"

<relative-event> ::= "XRIGHTCURSOR" | "XLEFTCURSOR"
                   | "XUPCURSOR" | "XDOWNCURSOR"
                   | "XUPSCROLL" | "XDOWNSCROLL"
                   | "XLEFTSCROLL" | "XRIGHTSCROLL"
                   | ... (see relative event names)

<key-press> ::= {<modifier> "-"} <key>

<modifier> ::= "Shift" | "Shift_L" | "Shift_R"
             | "C" | "Ctrl" | "Control" | "Control_L" | "Control_R"
             | "M" | "Alt" | "Alt_L" | "Alt_R"
             | "Super" | "Win" | "Windows" | "Super_L" | "Super_R"
             | <virtual-modifier>

<virtual-modifier> ::= <key>  (* Any key declared in virtual_modifiers *)
```

---

## Application Matcher Types

```ebnf
<application-matcher> ::= <literal-match>
                        | <name-match>
                        | <regex-match>

<literal-match> ::= <string>  (* Exact match with app class, e.g., "slack.Slack" *)

<name-match> ::= <string>  (* Matches app name, ignoring class, e.g., "Slack" *)

<regex-match> ::= <regex>  (* Regex pattern, e.g., "/^Minecraft.*/" *)
```

---

## Window Matcher Types

```ebnf
<window-matcher> ::= <regex>  (* Window title regex pattern only *)
```

---

## Device Matcher Types

```ebnf
<device-matcher> ::= <string>  (* Device name, filename, path, or substring *)
                   | <string-list>  (* No regex support *)
```

---

## Basic Types

```ebnf
<string> ::= (* Any valid YAML/TOML string *)

<string-list> ::= "[" {<string>} "]"

<key-list> ::= "[" {<key>} "]"

<regex> ::= "/" <pattern> "/"  (* Enclosed in forward slashes *)

<boolean> ::= "true" | "false"

<unsigned-integer> ::= <digit> {<digit>}

<letter> ::= "a" | "b" | ... | "z" | "A" | "B" | ... | "Z"

<digit> ::= "0" | "1" | ... | "9"

<uppercase-identifier> ::= <uppercase-letter> {<uppercase-letter> | <digit> | "_"}

<uppercase-letter> ::= "A" | "B" | ... | "Z"

<any> ::= (* Any valid YAML/TOML value *)
```

---

## Semantic Rules

### 1. Key Name Resolution
- Key names are **case-insensitive**
- The `KEY_` prefix is **optional** for evdev names
- Examples: `a`, `A`, `KEY_A`, `key_a` → all resolve to the same key
- Custom aliases are supported: `Enter` = `KEY_ENTER`, `Esc` = `KEY_ESC`, etc.

### 2. Modifier Parsing
- Modifiers are separated by `-`
- Modifier names are **case-insensitive**
- Order of modifiers does **not** matter
- Examples: `C-M-a`, `Ctrl-Alt-a`, `M-C-a` → all equivalent

### 3. Application Matching
- **Literal**: Exact match with app class (e.g., `"slack.Slack"`)
- **Name**: Matches app name only, ignoring class (e.g., `"Slack"`)
- **Regex**: Pattern enclosed in `/` (e.g., `"/^Minecraft.*/"`))
- Can be a single string or a list of strings

### 4. Window Matching
- **Only supports regex patterns** (enclosed in `/`)
- Matches against window title
- Can be a single regex or a list of regexes

### 5. Device Matching
- Matches device name, filename, path, or substring
- **Does not support regex** (unlike application matching)
- Can be a single string or a list of strings

### 6. Mode System
- Default mode is `"default"`
- Modes are strings or arrays of strings
- Only one mode is active at a time
- Use `set_mode` action to switch modes
- Keymaps/modmaps can be restricted to specific modes

### 7. Virtual Modifiers
- Declare keys as modifiers using `virtual_modifiers`
- Enables using any key as a modifier in key combinations
- Example: `CapsLock-i` (if `CapsLock` is in `virtual_modifiers`)

### 8. Exact Match
- When `exact_match: true`, only the specified modifiers must be pressed
- When `exact_match: false` (default), additional modifiers are allowed
- Example with `exact_match: false`: `C-a` matches `C-a`, `C-Shift-a`, etc.
- Example with `exact_match: true`: `C-a` matches only `C-a`

### 9. Multi-Purpose Keys
- A key can behave differently when held vs. tapped
- `alone_timeout_millis` determines the threshold (default: 1000ms)
- `free_hold: true` allows the held action even after other keys
- `free_hold: false` (default) requires continuous hold without other keys

### 10. Nested Remaps (Key Sequences)
- Enables Emacs-style key chords (e.g., `C-x C-c`)
- `timeout_millis` defines how long to wait for the next key
- `timeout_key` specifies what to emit if timeout occurs
- Nesting can be arbitrarily deep

### 11. Mark and Selection
- `set_mark: true` enables selection mode (Emacs-style)
- `with_mark` adds `Shift` modifier when mark is set
- Enables keyboard-based text selection

### 12. Hook Actions (Modmap Only)
- `press`, `repeat`, `release` hooks execute actions at specific events
- `skip_key_event: true` prevents the original key event
- Hooks can contain any `KeymapAction`

### 13. Action Priority
- Modmaps are applied before keymaps
- More specific filters take precedence
- Later entries in the list have higher priority

### 14. Comments
- YAML: Use `#` for line comments
- TOML: Use `#` for line comments
- Both formats support inline and full-line comments

---

## Complete Grammar Example

```yaml
# Root level configuration
default_mode: default
virtual_modifiers:
  - CapsLock
keypress_delay_ms: 0
enable_wheel: true

# Modmap: key-to-key remapping
modmap:
  - name: Global modifiers
    remap:
      CapsLock: Esc
      Alt_L: Ctrl_L

  - name: Multi-purpose Space
    remap:
      Space:
        held: Shift_L
        alone: Space
        alone_timeout_millis: 200
        free_hold: true

# Keymap: key combination remapping
keymap:
  - name: Emacs bindings
    application:
      not: [Emacs, Terminal]
    remap:
      C-a: Home
      C-e: End
      C-f: Right
      C-b: Left
      C-n: Down
      C-p: Up
      C-k: [Shift-End, C-x]
      C-space: { set_mark: true }
      C-g: { set_mark: false }
      M-w: [C-c, { set_mark: false }]
      C-w: [C-x, { set_mark: false }]
      C-y: C-v

  - name: Key sequences
    remap:
      C-x:
        remap:
          C-c: Esc
          C-s: C-w
        timeout_millis: 1000
        timeout_key: C-x

  - name: Launch apps
    remap:
      Super-t: { launch: [xterm] }
      Super-b: { launch: [bash, -c, "firefox"] }
```

---

## Validation Rules

1. **Required Fields**:
   - At least one of `modmap` or `keymap` must be present
   - Each modmap/keymap must have a `remap` field

2. **Type Constraints**:
   - `keypress_delay_ms` must be a non-negative integer
   - `alone_timeout_millis` must be a positive integer
   - `timeout_millis` must be a positive integer
   - All keys must be valid evdev key names or custom aliases

3. **Logical Constraints**:
   - A key cannot be mapped to itself directly (infinite loop)
   - Modes must exist before being referenced
   - Virtual modifiers must be valid keys
   - Application/window/device filters cannot be empty

4. **Syntax Constraints**:
   - Regex patterns must be valid and enclosed in `/`
   - Key presses must follow the `MOD-MOD-KEY` format
   - Modifier names must be valid (see modifier list)

---

## References

- [xremap README](../README.md) - User guide and examples
- [src/config/mod.rs](../src/config/mod.rs) - Main config struct
- [src/config/keymap.rs](../src/config/keymap.rs) - Keymap implementation
- [src/config/modmap.rs](../src/config/modmap.rs) - Modmap implementation
- [example/config.yml](../example/config.yml) - Example configuration

---

## Grammar Version

- Document Version: 1.0
- xremap Compatibility: Current main branch
- Last Updated: 2025-11-06
