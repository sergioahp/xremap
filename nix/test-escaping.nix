# Testing Nix String Escaping for Lua Code

let
  pkgs = import <nixpkgs> {};

  # Test 1: Using Lua's [[ ]] string literals (RECOMMENDED)
  test1 = pkgs.writeScriptBin "test1" ''
    #!/usr/bin/env lua

    local function shell_escape(str)
      -- Using Lua's [[ ]] literals avoids nested quote issues
      return "'" .. str:gsub("'", [['\'']]) .. "'"
    end

    print(shell_escape("hello world"))
    print(shell_escape("it's working"))
  '';

  # Test 2: Using escaped backslash in Nix multi-line string
  test2 = pkgs.writeScriptBin "test2" ''
    #!/usr/bin/env lua

    local function shell_escape(str)
      -- Backslash is escaped in Nix multi-line strings
      return "'" .. str:gsub("'", "'\\''") .. "'"
    end

    print(shell_escape("hello world"))
    print(shell_escape("it's working"))
  '';

  # Test 3: Using Nix's antiquotation for dots
  test3 = pkgs.writeScriptBin "test3" ''
    #!/usr/bin/env lua

    local function shell_escape(str)
      -- Using ''${".."} to escape dots
      return "'" ''${".."}'' str:gsub("'", [['\'']]) ''${".."}'' "'"
    end

    print(shell_escape("hello world"))
    print(shell_escape("it's working"))
  '';

  # Test 4: Simple approach with proper Lua escaping
  test4 = pkgs.writeScriptBin "test4" ''
    #!/usr/bin/env lua

    local function shell_escape(str)
      -- Most straightforward approach
      return "'" .. str:gsub("'", "'\"'\"'") .. "'"
    end

    print(shell_escape("hello world"))
    print(shell_escape("it's working"))
  '';

in {
  inherit test1 test2 test3 test4;
}
