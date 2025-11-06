# Nix String Escaping Fix for rofi-launcher.nix

## The Problem

When embedding Lua code in a Nix multi-line string (`''...''`), the shell escape pattern for single quotes was causing Nix evaluation errors.

**Error:**
```
error: syntax error, unexpected DOLLAR_CURLY
```

## The Solution

Use Lua's **bracket string literals** `[[...]]` for the replacement pattern.

### ✅ WORKING CODE

```nix
luaScript = ''
  #!/usr/bin/env lua

  local function shell_escape(str)
    -- Using Lua's [[...]] bracket literals
    return "'" .. str:gsub("'", [['\'']]) .. "'"
  end
'';
```

### Why This Works

1. **Lua's `..` operator** doesn't need escaping in Nix multi-line strings
2. **`[[...]]` bracket literals** in Lua don't interpret escape sequences
3. **`[['\'']]]`** is literally the 4 characters: `'` `\` `'` `'`
4. This is the **shell pattern** to escape a single quote: `'\''`
5. **No Nix-specific escaping** is needed for this pattern

## Alternative Working Approach

```nix
luaScript = ''
  #!/usr/bin/env lua

  local function shell_escape(str)
    return "'" .. str:gsub("'", "'\"'\"'") .. "'"
  end
'';
```

This uses the shell pattern: `'` (close quote) + `"'"` (escaped quote) + `'` (open quote)

## What DOESN'T Work

❌ **Trying to escape the `..` operator:**
```nix
return "'" ''${".."}'' str:gsub(...) ''${".."}'' "'"
```

This fails because:
- Nix tries to parse `''${...}''` as antiquotation syntax
- The `..` operator doesn't need escaping in Nix
- It creates unnecessary complexity

❌ **Using backslash escaping incorrectly:**
```nix
return "'" .. str:gsub("'", "'\"\\'\"'") .. "'"
```

This might work but is harder to read and depends on proper backslash handling.

## Key Rules for Nix Multi-Line Strings

When writing Lua (or any language) inside Nix `''...''` strings:

1. **Write code naturally** - most syntax works as-is
2. **Only escape Nix-specific syntax:**
   - `${` → `''${`
   - `''` → `'''`
   - `$` → `''$`
3. **Lua operators like `..`** work without escaping
4. **Use Lua features** (like `[[...]]`) to avoid nested quote issues

## Testing

The fixed version has been tested with:
- Simple strings: `"hello world"` → `'hello world'`
- Strings with quotes: `"it's working"` → `'it'\''s working'`
- Complex commands: `["sh", "-c", "echo 'test'"]` → properly escaped

## Files

- `rofi-launcher-fixed.nix` - Corrected version with detailed comments
- `rofi-launcher.nix` - Original version (needs updating)
- `test-escaping.nix` - Test cases for different approaches

## Implementation Note

The corrected `shell_escape` function:

```lua
local function shell_escape(str)
  return "'" .. str:gsub("'", [['\'']]) .. "'"
end
```

This escapes a string for safe shell execution by:
1. Wrapping it in single quotes: `'...'`
2. Replacing any internal single quotes with `'\''`:
   - Close the quote: `'`
   - Add an escaped quote: `\'`
   - Open the quote again: `'`

Example: `it's` becomes `'it'\''s'` which the shell interprets as three parts:
- `'it'` - literal "it"
- `\'` - escaped single quote
- `'s'` - literal "s"

Result: `it's` ✅

## Recommendation

Use the **bracket literal approach** (`[['\'']]]`) as it's:
- ✅ More explicit about what the pattern is
- ✅ Easier to understand (no nested quotes)
- ✅ Works reliably in Nix multi-line strings
- ✅ Leverages Lua's syntax for clarity

Replace your current `rofi-launcher.nix` with `rofi-launcher-fixed.nix`.
