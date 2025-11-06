{ lib, pkgs }:

# Xremap Launch Menu Generator
# Creates a rofi-based launcher that shows only launch commands from xremap config
# Preserves command structure as lists for proper execution from Lua

let
  splitter = import ./config-splitter.nix { inherit lib; };

  # Filter to only launch commands
  filterLaunchCommands = bindings:
    lib.filter (binding:
      binding ? action &&
      lib.isAttrs binding.action &&
      binding.action ? launch
    ) bindings.bindings;

  # Format a binding key sequence for display
  # Handles nested remaps by tracking the parent key
  formatKeySequence = binding:
    if binding ? parentKey then
      "${binding.parentKey} ${binding.binding}"
    else
      binding.binding;

  # Generate the launcher from a split config result
  generateLauncher = { splitResult, name ? "xremap-launcher" }:
    let
      launchBindings = filterLaunchCommands splitResult.dmenuBindings;

      # Convert bindings to Lua table format
      bindingsLua = lib.concatMapStringsSep ",\n  " (b:
        let
          keySeq = formatKeySequence b;
          desc = b.description or "No description";
          cmd = b.action.launch;
          # Keep command as Lua table (list)
          cmdLua = "{${lib.concatMapStringsSep ", " (arg: ''"${lib.escape ["\\" "\""] arg}"'') cmd}}";
          mode = b.mode or "default";
          category = b.name or "General";
        in
          ''{ key = "${keySeq}", desc = "${lib.escape ["\\" "\""] desc}", cmd = ${cmdLua}, mode = "${mode}", category = "${category}" }''
      ) launchBindings;

      luaScript = pkgs.writeText "${name}.lua" ''
        #!/usr/bin/env lua

        -- Xremap Launch Menu
        -- Generated launcher for xremap keybindings

        local bindings = {
          ${bindingsLua}
        }

        -- Format a binding for rofi display
        local function format_for_rofi(binding)
          return string.format("%-25s │ %-40s │ [%s]",
            binding.key,
            binding.desc,
            binding.category)
        end

        -- Build rofi menu entries
        local menu_items = {}
        local binding_map = {}

        for i, binding in ipairs(bindings) do
          local formatted = format_for_rofi(binding)
          table.insert(menu_items, formatted)
          binding_map[formatted] = binding
        end

        -- Show rofi menu
        local rofi_cmd = string.format(
          "${pkgs.rofi}/bin/rofi -dmenu -i -p 'Launch Command' " ..
          "-theme-str 'window {width: 80%%;}' " ..
          "-theme-str 'listview {lines: 15;}' " ..
          "-format 'i' " ..  -- Return index
          "-no-custom"
        )

        -- Create input for rofi
        local menu_text = table.concat(menu_items, "\n")

        -- Open rofi with pipe
        local handle = io.popen(rofi_cmd, "w")
        handle:write(menu_text)
        handle:close()

        -- Get rofi output (selected index)
        local result_handle = io.popen(rofi_cmd .. " <<< '" .. menu_text .. "'", "r")
        local result = result_handle:read("*a")
        result_handle:close()

        if result == nil or result == "" then
          os.exit(0)
        end

        -- Parse selected index
        local selected_idx = tonumber(result:match("^%d+"))
        if selected_idx == nil then
          -- Fallback: try to match by text
          result = result:gsub("\n", "")
          for formatted, binding in pairs(binding_map) do
            if formatted == result then
              selected_idx = nil
              for i, item in ipairs(menu_items) do
                if item == formatted then
                  selected_idx = i
                  break
                end
              end
              break
            end
          end
        end

        if selected_idx == nil or selected_idx < 1 or selected_idx > #bindings then
          os.exit(1)
        end

        local selected = bindings[selected_idx]

        -- Execute the command
        -- Build command with proper argument escaping
        local cmd_parts = {}
        for _, arg in ipairs(selected.cmd) do
          -- Escape for shell
          local escaped = arg:gsub("'", "'\\'''")
          table.insert(cmd_parts, "'" .. escaped .. "'")
        end

        local full_cmd = table.concat(cmd_parts, " ")

        -- Execute
        os.execute(full_cmd)
      '';

      # Alternative version with better rofi integration
      luaScriptV2 = pkgs.writeText "${name}-v2.lua" ''
        #!/usr/bin/env lua

        -- Xremap Launch Menu (Version 2 - Better rofi integration)

        local json = require("cjson")

        local bindings = {
          ${bindingsLua}
        }

        -- Format a binding for rofi display
        local function format_for_rofi(binding)
          return string.format("%-25s │ %-40s │ [%s]",
            binding.key,
            binding.desc,
            binding.category)
        end

        -- Build menu
        local menu_items = {}
        for i, binding in ipairs(bindings) do
          table.insert(menu_items, format_for_rofi(binding))
        end

        -- Show rofi and get selection
        local menu_text = table.concat(menu_items, "\n")
        local rofi_cmd = string.format(
          "echo %s | ${pkgs.rofi}/bin/rofi -dmenu -i " ..
          "-p 'Launch Command' " ..
          "-theme-str 'window {width: 80%%;} listview {lines: 15;}'",
          "${pkgs.coreutils}/bin/printf '%s' " .. "'" .. menu_text .. "'"
        )

        local handle = io.popen(rofi_cmd, "r")
        local selected_line = handle:read("*a")
        handle:close()

        if not selected_line or selected_line == "" then
          os.exit(0)
        end

        selected_line = selected_line:gsub("\n$", "")

        -- Find matching binding
        local selected = nil
        for i, item in ipairs(menu_items) do
          if item == selected_line then
            selected = bindings[i]
            break
          end
        end

        if not selected then
          os.exit(1)
        end

        -- Execute command
        local cmd_parts = {}
        for _, arg in ipairs(selected.cmd) do
          local escaped = arg:gsub("'", "'\\'''")
          table.insert(cmd_parts, "'" .. escaped .. "'")
        end

        local full_cmd = table.concat(cmd_parts, " ")
        os.execute(full_cmd)
      '';

      # Simpler shell wrapper version
      shellScript = pkgs.writeShellScriptBin name ''
        #!/usr/bin/env bash

        # Bindings data (JSON)
        BINDINGS_JSON=$(cat <<'EOF'
        [
        ${lib.concatMapStringsSep ",\n" (b:
          let
            keySeq = formatKeySequence b;
            desc = b.description or "No description";
            cmd = builtins.toJSON b.action.launch;
            mode = b.mode or "default";
            category = b.name or "General";
          in
            ''{"key":"${keySeq}","desc":"${lib.escape ["\\" "\""] desc}","cmd":${cmd},"mode":"${mode}","category":"${category}"}''
        ) launchBindings}
        ]
        EOF
        )

        # Format for rofi
        MENU=$(echo "$BINDINGS_JSON" | ${pkgs.jq}/bin/jq -r '.[] |
          "\(.key)\t│ \(.desc)\t│ [\(.category)]"' |
          ${pkgs.coreutils}/bin/column -t -s $'\t')

        # Show rofi
        SELECTED=$(echo "$MENU" | ${pkgs.rofi}/bin/rofi \
          -dmenu -i \
          -p "Launch Command" \
          -theme-str 'window {width: 80%;}' \
          -theme-str 'listview {lines: 15;}')

        if [ -z "$SELECTED" ]; then
          exit 0
        fi

        # Extract the key from selection
        KEY=$(echo "$SELECTED" | ${pkgs.gawk}/bin/awk -F'│' '{gsub(/^[ \t]+|[ \t]+$/, "", $1); print $1}')

        # Find matching command
        CMD=$(echo "$BINDINGS_JSON" | ${pkgs.jq}/bin/jq -r \
          --arg key "$KEY" \
          '.[] | select(.key == $key) | .cmd | @sh')

        if [ -z "$CMD" ]; then
          exit 1
        fi

        # Execute (cmd is already shell-escaped by jq's @sh)
        eval "$CMD"
      '';

      # Pure Lua version (most reliable)
      pureLuaScript = pkgs.writeScriptBin name ''
        #!${pkgs.lua}/bin/lua

        -- Xremap Launch Menu (Pure Lua)

        local bindings = {
          ${bindingsLua}
        }

        -- Format for rofi
        local function format_for_rofi(binding)
          return string.format("%-25s │ %-40s │ [%s]",
            binding.key,
            binding.desc,
            binding.category)
        end

        -- Build menu
        local lines = {}
        for i, binding in ipairs(bindings) do
          lines[i] = format_for_rofi(binding)
        end
        local menu = table.concat(lines, "\n")

        -- Write menu to temp file
        local temp = os.tmpname()
        local f = io.open(temp, "w")
        f:write(menu)
        f:close()

        -- Show rofi
        local rofi_cmd = string.format(
          "${pkgs.rofi}/bin/rofi -dmenu -i " ..
          "-p 'Launch Command' " ..
          "-theme-str 'window {width: 80%%;} listview {lines: 15;}' " ..
          "< %s",
          temp
        )

        local handle = io.popen(rofi_cmd, "r")
        local selected = handle:read("*l")
        local exit_code = handle:close()

        os.remove(temp)

        if not selected or selected == "" then
          os.exit(0)
        end

        -- Find binding
        local selected_binding = nil
        for i, line in ipairs(lines) do
          if line == selected then
            selected_binding = bindings[i]
            break
          end
        end

        if not selected_binding then
          io.stderr:write("Error: Could not find selected binding\n")
          os.exit(1)
        end

        -- Execute command
        -- Use exec to properly handle arguments without shell interpretation issues
        local cmd = selected_binding.cmd

        -- Build argument list for execv-style execution
        local args = {}
        for i, arg in ipairs(cmd) do
          args[i] = arg
        end

        -- Spawn process
        local pid = os.execute(string.format(
          "exec %s",
          table.concat((function()
            local escaped = {}
            for i, arg in ipairs(cmd) do
              -- Escape for shell
              local e = arg:gsub("'", "'\"'\"'")
              escaped[i] = "'" .. e .. "'"
            end
            return escaped
          end)(), " ")
        ))

        os.exit(pid and 0 or 1)
      '';

    in {
      # The generated launcher script
      launcher = pureLuaScript;

      # Alternative versions
      shellLauncher = shellScript;

      # The bindings data as JSON
      bindingsJson = pkgs.writeText "${name}-bindings.json" (builtins.toJSON launchBindings);

      # Statistics
      stats = {
        totalLaunchCommands = builtins.length launchBindings;
        bindings = launchBindings;
      };
    };

  # Helper to process a nested remap and extract launch commands
  # This flattens nested remaps like super-m -> super-l into "super-m super-l"
  flattenNestedLaunches = bindings:
    let
      processBinding = binding:
        if binding ? action && lib.isAttrs binding.action && binding.action ? remap then
          # This is a nested remap, extract its children
          let
            parentKey = binding.binding;
            children = lib.mapAttrsToList (childKey: childAction:
              if lib.isAttrs childAction && childAction ? launch then
                binding // {
                  binding = childKey;
                  parentKey = parentKey;
                  action = { launch = childAction.launch; };
                  description = childAction.description or binding.description or null;
                }
              else if lib.isAttrs childAction && childAction ? remap then
                # Nested nested remap - recursive
                null  # TODO: handle deeper nesting if needed
              else
                null
            ) binding.action.remap;
          in
            lib.filter (x: x != null) children
        else
          [ binding ];
    in
      lib.flatten (map processBinding bindings);

  # Enhanced version that flattens nested remaps
  generateLauncherEnhanced = { splitResult, name ? "xremap-launcher" }:
    let
      # Get all bindings and flatten nested remaps
      allBindings = flattenNestedLaunches splitResult.dmenuBindings.bindings;

      # Filter to only launch commands
      launchBindings = lib.filter (binding:
        binding ? action &&
        lib.isAttrs binding.action &&
        binding.action ? launch
      ) allBindings;

      # Rest of implementation same as generateLauncher
      # ... (use same Lua script generation)
    in
      generateLauncher {
        splitResult = splitResult // {
          dmenuBindings = splitResult.dmenuBindings // {
            bindings = allBindings;
          };
        };
        inherit name;
      };

in {
  inherit generateLauncher generateLauncherEnhanced filterLaunchCommands;

  # Convenience function: split config and generate launcher in one go
  makeLauncher = config: name:
    let
      splitResult = splitter.splitConfig config;
    in
      generateLauncherEnhanced { inherit splitResult name; };
}
