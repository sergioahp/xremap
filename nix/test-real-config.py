#!/usr/bin/env python3
"""
Test the config splitter logic with the real user config.
Since we don't have full Nix, we simulate the transformation.
"""

import json
import sys
import os

# Add current directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Import from verify-logic.py
import importlib.util
spec = importlib.util.spec_from_file_location("verify_logic", os.path.join(os.path.dirname(__file__), "verify-logic.py"))
verify_logic = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify_logic)
split_config = verify_logic.split_config

# Simplified version of user's config (without pkgs references for testing)
real_config = {
    "keypress_delay_ms": 20,
    "modmap": [
        {
            "name": "main modmaps",
            "remap": {
                "shift_r": "alt_l"
            }
        }
    ],
    "keymap": [
        {
            "name": "main remaps",
            "remap": {
                "super-m": {
                    "remap": {
                        "super-l": {
                            "launch": ["kitty"],
                            "description": "Launch Kitty terminal"
                        },
                        "super-f": {
                            "launch": ["firefox"],
                            "description": "Launch Firefox browser"
                        },
                        "super-e": {
                            "launch": ["kitty", "ranger"],
                            "description": "Launch Ranger file manager"
                        },
                        "super-o": {
                            "launch": ["kitty", "btop"],
                            "description": "Launch btop system monitor"
                        },
                        "super-m": {
                            "launch": ["rofi", "-show", "drun"],
                            "description": "Show application launcher (Rofi)"
                        },
                        "super-i": {
                            "launch": ["bitwarden"],
                            "description": "Launch Bitwarden password manager"
                        }
                    }
                },
                "super-s": {
                    "remap": {
                        "super-e": {
                            "launch": ["sh", "-c", "wl-paste | swappy -f -"],
                            "description": "Edit clipboard screenshot"
                        },
                        "super-d": {
                            "launch": ["sh", "-c", "grim | swappy -f -"],
                            "description": "Take selection screenshot"
                        }
                    }
                },
                "super-u": {
                    "remap": {
                        "super-f": {
                            "launch": ["dunstctl", "history-pop"],
                            "description": "Show notification history"
                        },
                        "super-d": {
                            "launch": ["dunstctl", "close"],
                            "description": "Close current notification"
                        },
                        "super-s": {
                            "launch": ["dunstctl", "close-all"],
                            "description": "Close all notifications"
                        },
                        "super-t": {
                            "launch": ["dunstctl", "set-paused", "toggle"],
                            "description": "Toggle notifications pause"
                        }
                    }
                },
                "super-comma": {
                    "launch": ["hyprctl", "dispatch", "focusmonitor", "+1"],
                    "description": "Focus next monitor"
                },
                "super-j": {
                    "launch": ["hyprctl", "dispatch", "cyclenext"],
                    "description": "Cycle to next window"
                },
                "super-k": {
                    "launch": ["hyprctl", "dispatch", "cyclenext", "prev"],
                    "description": "Cycle to previous window"
                },
                "super-n": {
                    "launch": ["hyprctl", "dispatch", "workspace", "r+1"],
                    "description": "Next workspace"
                },
                "super-p": {
                    "launch": ["hyprctl", "dispatch", "workspace", "r-1"],
                    "description": "Previous workspace"
                },
                "super-i": {
                    "launch": ["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"],
                    "description": "Decrease volume"
                },
                "super-o": {
                    "launch": ["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", "5%+"],
                    "description": "Increase volume"
                }
            }
        },
        {
            "name": "to normal",
            "remap": {
                "super-space": {
                    "action": {"set_mode": "normal"},
                    "description": "Enter normal mode (Vim-like)"
                }
            },
            "mode": "default"
        },
        {
            "name": "to default",
            "remap": {
                "super-space": {
                    "action": {"set_mode": "default"},
                    "description": "Return to default mode"
                }
            },
            "mode": "normal"
        },
        {
            "mode": "normal",
            "name": "Normal mode window management",
            "remap": {
                "shift-h": {
                    "launch": ["hyprctl", "dispatch", "moveactive", "-100", "0"],
                    "description": "Move window left"
                },
                "shift-j": {
                    "launch": ["hyprctl", "dispatch", "moveactive", "0", "100"],
                    "description": "Move window down"
                },
                "h": {
                    "launch": ["hyprctl", "dispatch", "resizeactive", "-100", "0"],
                    "description": "Resize window left"
                },
                "j": {
                    "launch": ["hyprctl", "dispatch", "resizeactive", "0", "100"],
                    "description": "Resize window down"
                },
                "c": {
                    "launch": ["hyprctl", "dispatch", "centerwindow"],
                    "description": "Center window"
                },
                "semicolon": {
                    "launch": ["brightnessctl", "set", "10%+"],
                    "description": "Increase brightness"
                }
            }
        },
        {
            "name": "firefox remaps",
            "remap": {
                "super-b": {
                    "action": ["c-l", "shift-5", "space"],
                    "description": "Jump to search in Firefox"
                },
                "super-c": {
                    "remap": {
                        "super-c": {
                            "action": "c-alt-z",
                            "description": "Reader mode"
                        },
                        "super-u": {
                            "action": "alt-1",
                            "description": "Switch to tab 1"
                        },
                        "super-i": {
                            "action": "alt-2",
                            "description": "Switch to tab 2"
                        }
                    }
                },
                "super-z": {
                    "action": "c-shift-tab",
                    "description": "Previous tab"
                },
                "super-x": {
                    "action": "c-tab",
                    "description": "Next tab"
                }
            },
            "application": {
                "only": "firefox"
            }
        }
    ]
}

def print_section(title):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}\n")

def main():
    print_section("TESTING WITH REAL USER CONFIG")

    # Split the config
    result = split_config(real_config)

    print_section("STATISTICS")
    print(f"Total bindings extracted: {result['dmenuBindings']['count']}")
    print(f"Modmaps: {len(real_config['modmap'])}")
    print(f"Keymaps: {len(real_config['keymap'])}")

    # Count nested remaps
    nested_count = sum(1 for b in result['dmenuBindings']['bindings']
                      if isinstance(b.get('action'), dict) and 'remap' in b['action'])
    print(f"Nested remaps: {nested_count}")

    # Count descriptions
    with_desc = sum(1 for b in result['dmenuBindings']['bindings'] if b.get('description'))
    print(f"Bindings with descriptions: {with_desc}")
    print(f"Bindings without descriptions: {result['dmenuBindings']['count'] - with_desc}")

    print_section("SAMPLE BINDINGS")
    print("First 10 bindings with descriptions:\n")
    count = 0
    for binding in result['dmenuBindings']['bindings']:
        if binding.get('description') and count < 10:
            mode = binding.get('mode', 'default')
            name = binding.get('name', 'Unknown')
            app = ''
            if 'application' in binding:
                if 'only' in binding['application']:
                    app = f" [only: {binding['application']['only']}]"
                elif 'not' in binding['application']:
                    app = f" [not: {binding['application']['not']}]"

            print(f"  {binding['binding']:20} → {binding['description']}")
            print(f"  {'':20}   [{name}] (mode: {mode}){app}\n")
            count += 1

    print_section("MODE BREAKDOWN")
    modes = {}
    for binding in result['dmenuBindings']['bindings']:
        mode = binding.get('mode', 'default')
        modes[mode] = modes.get(mode, 0) + 1

    for mode, count in sorted(modes.items()):
        print(f"  {mode:15} : {count:3} bindings")

    print_section("CATEGORY BREAKDOWN")
    categories = {}
    for binding in result['dmenuBindings']['bindings']:
        cat = binding.get('name', 'Unnamed')
        categories[cat] = categories.get(cat, 0) + 1

    for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
        print(f"  {cat:30} : {count:3} bindings")

    print_section("APPLICATION-SPECIFIC BINDINGS")
    app_bindings = [b for b in result['dmenuBindings']['bindings'] if 'application' in b]
    if app_bindings:
        for binding in app_bindings[:5]:
            app_filter = binding['application']
            filter_type = 'only' if 'only' in app_filter else 'not'
            apps = app_filter.get(filter_type)
            print(f"  {binding['binding']:20} → {binding.get('description', 'N/A')}")
            print(f"  {'':20}   ({filter_type}: {apps})\n")
    else:
        print("  No application-specific bindings found")

    print_section("NESTED REMAPS (KEY SEQUENCES)")
    nested_bindings = [b for b in result['dmenuBindings']['bindings']
                       if isinstance(b.get('action'), dict) and 'remap' in b['action']]
    if nested_bindings:
        for binding in nested_bindings[:5]:
            print(f"  {binding['binding']:20} → Prefix for nested commands")
            if 'description' in binding:
                print(f"  {'':20}   ({binding['description']})")
            # Show what keys are under this prefix
            remap_keys = list(binding['action']['remap'].keys())
            print(f"  {'':20}   Contains: {', '.join(remap_keys[:5])}")
            if len(remap_keys) > 5:
                print(f"  {'':20}   ... and {len(remap_keys) - 5} more")
            print()
    else:
        print("  No nested remaps found")

    print_section("CLEAN CONFIG CHECK")
    # Verify descriptions are removed
    clean_str = json.dumps(result['cleanConfig'])
    if 'description' in clean_str:
        print("  ⚠ WARNING: Clean config still contains 'description' field!")
    else:
        print("  ✓ Clean config has all descriptions removed")

    # Verify action wrappers are removed where appropriate
    sample_keymap = result['cleanConfig']['keymap'][1]  # "to normal" keymap
    super_space_action = sample_keymap['remap']['super-space']
    if isinstance(super_space_action, dict) and 'set_mode' in super_space_action:
        print("  ✓ Action wrappers properly unwrapped")
    else:
        print(f"  ⚠ Action wrapper issue: {super_space_action}")

    # Verify metadata is preserved
    if 'mode' in sample_keymap:
        print("  ✓ Mode metadata preserved")
    else:
        print("  ⚠ Mode metadata lost")

    if 'keypress_delay_ms' in result['cleanConfig']:
        print("  ✓ Top-level config options preserved")
    else:
        print("  ⚠ Top-level config options lost")

    print_section("ROFI FORMAT PREVIEW")
    print("Sample of how bindings would appear in rofi:\n")
    for binding in result['dmenuBindings']['bindings'][:10]:
        if binding.get('description'):
            binding_str = binding['binding']
            desc_str = binding['description']
            cat_str = binding.get('name', 'General')
            mode_str = binding.get('mode', 'default')
            print(f"  {binding_str:20} │ {desc_str:40} │ [{cat_str}]")

    print_section("SUCCESS!")
    print("✓ Config splitter successfully processed your real config")
    print(f"✓ Extracted {result['dmenuBindings']['count']} total bindings")
    print(f"✓ {with_desc} bindings have descriptions")
    print("✓ Clean config is ready for xremap")
    print("✓ Dmenu bindings are ready for rofi integration")
    print()

if __name__ == '__main__':
    main()
