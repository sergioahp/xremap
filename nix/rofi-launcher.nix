{ lib, pkgs }:

# Xremap Rofi Launcher Generator
#
# Creates a rofi-based launcher from xremap config that:
# - Shows only launch commands (excludes mode switching, set_mark, etc.)
# - Displays full key sequences (e.g., "super-m super-l")
# - Preserves command structure as lists for proper execution
# - Uses Lua for reliable command execution

let
  splitter = import ./config-splitter.nix { inherit lib; };

  # Recursively extract launch commands from bindings, handling nested remaps
  # Returns a list of { keySequence, description, command, metadata }
  extractLaunchCommands = bindings: parentKey: let
    processBinding = binding:
      let
        fullKey = if parentKey != "" then "${parentKey} ${binding.binding}" else binding.binding;
        action = binding.action or null;
      in
        if action == null then
          []
        else if lib.isAttrs action && action ? launch then
          # This is a launch command
          [{
            keySequence = fullKey;
            description = binding.description or "No description";
            command = action.launch;
            mode = binding.mode or "default";
            category = binding.name or "General";
            application = binding.application or null;
          }]
        else if lib.isAttrs action && action ? remap then
          # This is a nested remap, recursively process children
          let
            children = lib.mapAttrsToList (childKey: childAction:
              processBinding (binding // {
                binding = childKey;
                action = childAction;
              })
            ) action.remap;
          in
            lib.flatten children
        else
          # Not a launch command (e.g., set_mode, set_mark, key press)
          [];
  in
    lib.flatten (map (b: processBinding b) bindings);

  # Generate rofi launcher script
  makeLauncher = { config, name ? "xremap-launcher" }:
    let
      # Split config
      splitResult = splitter.splitConfig config;

      # Extract all launch commands with their full key sequences
      launchCommands = extractLaunchCommands splitResult.dmenuBindings.bindings "";

      # Generate Lua table entries for each command
      commandsLua = lib.concatMapStringsSep ",\n  " (cmd:
        let
          # Escape strings for Lua
          escapeL = str: lib.replaceStrings
            [''\'' "\\" "\n" "\r" "\t"]
            ["\\\'" "\\\\" "\\n" "\\r" "\\t"]
            str;

          # Convert command list to Lua table
          cmdTable = "{" + (lib.concatMapStringsSep ", " (arg:
            ''"${escapeL arg}"''
          ) cmd.command) + "}";

          # Metadata
          appFilter = if cmd.application != null then
            if cmd.application ? only then
              '' app_only = "${escapeL (builtins.toJSON cmd.application.only)}", ''
            else if cmd.application ? not then
              '' app_not = "${escapeL (builtins.toJSON cmd.application.not)}", ''
            else ""
          else "";
        in
          ''  {'' + "\n" +
          ''    key = "${escapeL cmd.keySequence}",'' + "\n" +
          ''    desc = "${escapeL cmd.description}",'' + "\n" +
          ''    cmd = ${cmdTable},'' + "\n" +
          ''    mode = "${escapeL cmd.mode}",'' + "\n" +
          ''    category = "${escapeL cmd.category}",'' + "\n" +
          ''    ${appFilter}'' +
          ''  }''
      ) launchCommands;

      # Generate the Lua script
      luaScript = ''
        #!/usr/bin/env lua
        -- Xremap Launch Menu
        -- Auto-generated from xremap configuration
        -- Preserves command structure for proper execution

        local commands = {
        ${commandsLua}
        }

        -- Escape string for shell execution
        local function shell_escape(str)
          -- Single quote escaping: ' becomes '\''
          return "'" .. str:gsub("'", "'\"'\"'") .. "'"
        end

        -- Format command for rofi display
        local function format_for_rofi(cmd)
          return string.format("%-30s │ %-50s │ [%s]",
            cmd.key,
            cmd.desc,
            cmd.category)
        end

        -- Build menu entries
        local menu_lines = {}
        for i, cmd in ipairs(commands) do
          menu_lines[i] = format_for_rofi(cmd)
        end

        -- Write menu to temporary file
        local temp_file = os.tmpname()
        local f = io.open(temp_file, "w")
        if not f then
          io.stderr:write("Error: Could not create temporary file\n")
          os.exit(1)
        end
        f:write(table.concat(menu_lines, "\n"))
        f:close()

        -- Show rofi menu
        local rofi_cmd = string.format(
          "${pkgs.rofi}/bin/rofi -dmenu -i " ..
          "-p 'Launch' " ..
          "-theme-str 'window {width: 90%%; height: 60%%;}' " ..
          "-theme-str 'listview {lines: 20;}' " ..
          "-mesg 'Press Enter to launch, Esc to cancel' " ..
          "< %s",
          temp_file
        )

        local handle = io.popen(rofi_cmd, "r")
        if not handle then
          os.remove(temp_file)
          io.stderr:write("Error: Could not open rofi\n")
          os.exit(1)
        end

        local selected = handle:read("*l")
        handle:close()
        os.remove(temp_file)

        -- Exit if nothing selected
        if not selected or selected == "" then
          os.exit(0)
        end

        -- Find the selected command
        local selected_cmd = nil
        for i, line in ipairs(menu_lines) do
          if line == selected then
            selected_cmd = commands[i]
            break
          end
        end

        if not selected_cmd then
          io.stderr:write("Error: Could not find selected command\n")
          os.exit(1)
        end

        -- Build command string with proper escaping
        local cmd_parts = {}
        for i, arg in ipairs(selected_cmd.cmd) do
          cmd_parts[i] = shell_escape(arg)
        end
        local full_cmd = table.concat(cmd_parts, " ")

        -- Show what we're executing (optional, comment out for silent execution)
        -- io.stderr:write("Executing: " .. full_cmd .. "\n")

        -- Execute the command
        local success = os.execute(full_cmd)

        -- Exit with appropriate code
        if success then
          os.exit(0)
        else
          os.exit(1)
        end
      '';

    in {
      # The main launcher script
      script = pkgs.writeScriptBin name luaScript;

      # Export commands as JSON for other tools
      commandsJson = pkgs.writeText "${name}-commands.json"
        (builtins.toJSON launchCommands);

      # Export commands as Lua module
      commandsLua = pkgs.writeText "${name}-commands.lua" ''
        -- Xremap launch commands (Lua module)
        return {
        ${commandsLua}
        }
      '';

      # Statistics
      stats = {
        totalCommands = builtins.length launchCommands;
        byMode = lib.groupBy (c: c.mode) launchCommands;
        byCategory = lib.groupBy (c: c.category) launchCommands;
      };

      # The raw launch commands list (for inspection)
      commands = launchCommands;
    };

in {
  inherit makeLauncher;

  # Example usage:
  # let
  #   launcher = import ./rofi-launcher.nix { inherit lib pkgs; };
  #   myLauncher = launcher.makeLauncher {
  #     config = myXremapConfig;
  #     name = "my-xremap-launcher";
  #   };
  # in {
  #   home.packages = [ myLauncher.script ];
  # }
}
