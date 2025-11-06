{ lib ? import <nixpkgs/lib> }:

let
  splitter = import ./config-splitter.nix { inherit lib; };

  # Test utilities
  assertEqual = name: expected: actual:
    if expected == actual then
      { success = true; inherit name; }
    else
      { success = false; inherit name expected actual; };

  assertNotNull = name: value:
    if value != null then
      { success = true; inherit name; }
    else
      { success = false; inherit name; message = "Value is null"; };

  # Test 1: Simple keymap without descriptions
  test1 = let
    config = {
      keymap = [
        {
          name = "Basic";
          remap = {
            "C-a" = "Home";
            "C-e" = "End";
          };
        }
      ];
    };
    result = splitter.splitConfig config;
  in {
    name = "Simple keymap without descriptions";
    tests = [
      (assertEqual "Has cleanConfig" true (result ? cleanConfig))
      (assertEqual "Has dmenuBindings" true (result ? dmenuBindings))
      (assertEqual "Binding count" 2 result.dmenuBindings.count)
      (assertEqual "Clean config has keymap" true (result.cleanConfig ? keymap))
      (assertEqual "First binding is C-a" "C-a" (builtins.elemAt result.dmenuBindings.bindings 0).binding)
    ];
  };

  # Test 2: Keymap with descriptions
  test2 = let
    config = {
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
    result = splitter.splitConfig config;
    firstBinding = builtins.elemAt result.dmenuBindings.bindings 0;
  in {
    name = "Keymap with descriptions";
    tests = [
      (assertEqual "Binding count" 2 result.dmenuBindings.count)
      (assertEqual "First binding has description" "Open terminal" firstBinding.description)
      (assertEqual "First binding has name" "Launch" firstBinding.name)
      (assertEqual "First binding has mode" "default" firstBinding.mode)
      (assertEqual "Clean config has no description" false
        (builtins.hasAttr "description"
          (builtins.elemAt result.cleanConfig.keymap 0).remap."Super-t"))
    ];
  };

  # Test 3: Modmap
  test3 = let
    config = {
      modmap = [
        {
          name = "Global";
          remap = {
            "CapsLock" = "Esc";
            "Alt_L" = "Ctrl_L";
          };
        }
      ];
    };
    result = splitter.splitConfig config;
  in {
    name = "Modmap processing";
    tests = [
      (assertEqual "Binding count" 2 result.dmenuBindings.count)
      (assertEqual "Has modmap" true (result.cleanConfig ? modmap))
      (assertEqual "CapsLock remapped" "Esc"
        (builtins.elemAt result.cleanConfig.modmap 0).remap.CapsLock)
    ];
  };

  # Test 4: Application filters
  test4 = let
    config = {
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
    };
    result = splitter.splitConfig config;
    binding = builtins.elemAt result.dmenuBindings.bindings 0;
  in {
    name = "Application filters";
    tests = [
      (assertEqual "Binding has application filter" true (binding ? application))
      (assertEqual "Filter has 'not'" true (binding.application ? not))
      (assertEqual "Description preserved" "Beginning of line" binding.description)
      (assertEqual "Clean config has application" true
        ((builtins.elemAt result.cleanConfig.keymap 0) ? application))
    ];
  };

  # Test 5: Mixed modmap and keymap
  test5 = let
    config = {
      default_mode = "default";
      modmap = [
        {
          name = "Global";
          remap = { "CapsLock" = "Esc"; };
        }
      ];
      keymap = [
        {
          name = "Launch";
          remap = {
            "Super-t" = {
              launch = ["xterm"];
              description = "Terminal";
            };
          };
        }
      ];
    };
    result = splitter.splitConfig config;
  in {
    name = "Mixed modmap and keymap";
    tests = [
      (assertEqual "Total bindings" 2 result.dmenuBindings.count)
      (assertEqual "Has modmap" true (result.cleanConfig ? modmap))
      (assertEqual "Has keymap" true (result.cleanConfig ? keymap))
      (assertEqual "Has default_mode" "default" result.cleanConfig.default_mode)
    ];
  };

  # Test 6: Complex action types
  test6 = let
    config = {
      keymap = [
        {
          name = "Advanced";
          remap = {
            "C-space" = {
              action.set_mark = true;
              description = "Set mark";
            };
            "C-x" = {
              action.remap = {
                "C-c" = "Esc";
                "C-s" = "C-w";
              };
              description = "C-x prefix";
            };
          };
        }
      ];
    };
    result = splitter.splitConfig config;
  in {
    name = "Complex action types";
    tests = [
      (assertEqual "Binding count" 2 result.dmenuBindings.count)
      (assertEqual "Set mark action preserved" true
        ((builtins.elemAt result.cleanConfig.keymap 0).remap."C-space".set_mark))
      (assertEqual "Nested remap preserved" true
        ((builtins.elemAt result.cleanConfig.keymap 0).remap."C-x" ? remap))
    ];
  };

  # Test 7: Empty config
  test7 = let
    config = {};
    result = splitter.splitConfig config;
  in {
    name = "Empty config";
    tests = [
      (assertEqual "Binding count" 0 result.dmenuBindings.count)
      (assertEqual "Clean config empty" {} result.cleanConfig)
    ];
  };

  # Test 8: Multiple modes
  test8 = let
    config = {
      default_mode = "insert";
      keymap = [
        {
          name = "Insert mode";
          mode = "insert";
          remap = {
            "Esc" = {
              action.set_mode = "normal";
              description = "Enter normal mode";
            };
          };
        }
        {
          name = "Normal mode";
          mode = "normal";
          remap = {
            "i" = {
              action.set_mode = "insert";
              description = "Enter insert mode";
            };
            "h" = {
              action = "Left";
              description = "Move left";
            };
          };
        }
      ];
    };
    result = splitter.splitConfig config;
    normalBindings = splitter.filterByMode "normal" result.dmenuBindings;
  in {
    name = "Multiple modes";
    tests = [
      (assertEqual "Total bindings" 3 result.dmenuBindings.count)
      (assertEqual "Normal mode bindings" 2 (lib.length normalBindings))
      (assertEqual "Default mode preserved" "insert" result.cleanConfig.default_mode)
    ];
  };

  # Test 9: Utility functions
  test9 = let
    config = {
      keymap = [
        {
          name = "Launch";
          remap = {
            "Super-t" = {
              launch = ["xterm"];
              description = "Terminal";
            };
          };
        }
        {
          name = "Edit";
          remap = {
            "C-a" = {
              action = "Home";
              description = "Home key";
            };
          };
        }
      ];
    };
    result = splitter.splitConfig config;
    grouped = splitter.groupByName result.dmenuBindings;
  in {
    name = "Utility functions";
    tests = [
      (assertEqual "Has Launch group" true (grouped ? "Launch"))
      (assertEqual "Has Edit group" true (grouped ? "Edit"))
      (assertEqual "Launch group size" 1 (lib.length grouped.Launch))
    ];
  };

  # Test 10: Application filter - filterByApplication
  test10 = let
    config = {
      keymap = [
        {
          name = "Chrome only";
          application.only = ["Google-chrome"];
          remap = {
            "C-a" = {
              action = "Home";
              description = "Chrome binding";
            };
          };
        }
        {
          name = "Not Emacs";
          application.not = ["Emacs"];
          remap = {
            "C-e" = {
              action = "End";
              description = "End key";
            };
          };
        }
        {
          name = "Global";
          remap = {
            "C-g" = {
              action = "Esc";
              description = "Cancel";
            };
          };
        }
      ];
    };
    result = splitter.splitConfig config;
    chromeBindings = splitter.filterByApplication "Google-chrome" result.dmenuBindings;
    emacsBindings = splitter.filterByApplication "Emacs" result.dmenuBindings;
  in {
    name = "Application filtering utility";
    tests = [
      (assertEqual "Total bindings" 3 result.dmenuBindings.count)
      (assertEqual "Chrome bindings" 2 (lib.length chromeBindings))
      (assertEqual "Emacs bindings (should exclude 'Not Emacs')" 2 (lib.length emacsBindings))
    ];
  };

  # Run all tests
  allTests = [
    test1
    test2
    test3
    test4
    test5
    test6
    test7
    test8
    test9
    test10
  ];

  # Execute tests and collect results
  runTests = map (testSuite:
    testSuite // {
      results = map (test: test) testSuite.tests;
      passed = lib.all (t: t.success) testSuite.tests;
      failed = lib.filter (t: !t.success) testSuite.tests;
    }
  ) allTests;

  # Summary
  summary = {
    total = lib.length allTests;
    passed = lib.length (lib.filter (t: t.passed) runTests);
    failed = lib.length (lib.filter (t: !t.passed) runTests);
    details = runTests;
  };

in {
  inherit summary runTests allTests;

  # Pretty print results
  report = let
    testResults = lib.concatMapStringsSep "\n" (suite:
      let
        status = if suite.passed then "✓ PASS" else "✗ FAIL";
        failedTests = if suite.passed then "" else
          "\n  Failed assertions:\n" +
          lib.concatMapStringsSep "\n" (t: "    - ${t.name}") suite.failed;
      in
        "  ${status}: ${suite.name}${failedTests}"
    ) runTests;
  in ''

    ═══════════════════════════════════════════════
    XREMAP NIX CONFIG SPLITTER - TEST RESULTS
    ═══════════════════════════════════════════════

    ${testResults}

    ───────────────────────────────────────────────
    Total: ${toString summary.total}
    Passed: ${toString summary.passed}
    Failed: ${toString summary.failed}
    ═══════════════════════════════════════════════
  '';
}
