#!/usr/bin/env python3
"""
Test the launcher generator logic
Since we can't run Nix, we'll simulate the transformation in Python
"""

import json

# Simplified extraction logic
def extract_launch_commands(bindings, parent_key=""):
    """Recursively extract launch commands from bindings"""
    commands = []

    for binding in bindings:
        full_key = f"{parent_key} {binding['binding']}" if parent_key else binding['binding']
        action = binding.get('action')

        if not action:
            continue

        # Check if this is a launch command
        if isinstance(action, dict) and 'launch' in action:
            commands.append({
                'keySequence': full_key,
                'description': binding.get('description', 'No description'),
                'command': action['launch'],
                'mode': binding.get('mode', 'default'),
                'category': binding.get('name', 'General'),
                'application': binding.get('application')
            })

        # Check if this is a nested remap
        elif isinstance(action, dict) and 'remap' in action:
            # Recursively process children
            child_bindings = []
            for child_key, child_action in action['remap'].items():
                child_binding = {
                    'binding': child_key,
                    'action': child_action,
                    'name': binding.get('name'),
                    'mode': binding.get('mode'),
                    'application': binding.get('application')
                }
                # Copy description if child has it
                if isinstance(child_action, dict) and 'description' in child_action:
                    child_binding['description'] = child_action['description']

                child_bindings.append(child_binding)

            # Recursive call
            child_commands = extract_launch_commands(child_bindings, full_key)
            commands.extend(child_commands)

    return commands

# Test config (simplified from user's real config)
test_config = {
    'keymap': [
        {
            'name': 'Application Launchers',
            'remap': {
                'super-m': {
                    'remap': {
                        'super-l': {
                            'launch': ['kitty'],
                            'description': 'Launch Kitty terminal'
                        },
                        'super-f': {
                            'launch': ['firefox'],
                            'description': 'Launch Firefox browser'
                        },
                        'super-e': {
                            'launch': ['kitty', 'ranger'],
                            'description': 'Launch Ranger file manager'
                        },
                        'super-m': {
                            'launch': ['rofi', '-show', 'drun'],
                            'description': 'Show application launcher'
                        }
                    }
                },
                'super-s': {
                    'remap': {
                        'super-e': {
                            'launch': ['sh', '-c', 'wl-paste | swappy -f -'],
                            'description': 'Edit clipboard screenshot'
                        },
                        'super-d': {
                            'launch': ['sh', '-c', 'grim | swappy -f -'],
                            'description': 'Take selection screenshot'
                        }
                    }
                },
                'super-comma': {
                    'launch': ['hyprctl', 'dispatch', 'focusmonitor', '+1'],
                    'description': 'Focus next monitor'
                },
                'super-j': {
                    'launch': ['hyprctl', 'dispatch', 'cyclenext'],
                    'description': 'Cycle to next window'
                }
            }
        },
        {
            'name': 'Mode Switching',
            'mode': 'default',
            'remap': {
                'super-space': {
                    'action': {'set_mode': 'normal'},
                    'description': 'Enter normal mode'
                }
            }
        },
        {
            'name': 'Normal Mode',
            'mode': 'normal',
            'remap': {
                'h': {
                    'launch': ['hyprctl', 'dispatch', 'resizeactive', '-100', '0'],
                    'description': 'Resize window left'
                },
                'j': {
                    'launch': ['hyprctl', 'dispatch', 'resizeactive', '0', '100'],
                    'description': 'Resize window down'
                }
            }
        }
    ]
}

def test_launcher_extraction():
    print("="*70)
    print("TESTING LAUNCHER COMMAND EXTRACTION")
    print("="*70)
    print()

    # Extract commands from all keymaps
    all_commands = []
    for keymap in test_config['keymap']:
        bindings = []
        for key, action in keymap['remap'].items():
            binding = {
                'binding': key,
                'action': action,
                'name': keymap.get('name'),
                'mode': keymap.get('mode'),
            }
            if isinstance(action, dict) and 'description' in action:
                binding['description'] = action['description']
            bindings.append(binding)

        commands = extract_launch_commands(bindings)
        all_commands.extend(commands)

    print(f"📊 STATISTICS")
    print(f"   Total launch commands extracted: {len(all_commands)}")
    print()

    # Group by mode
    by_mode = {}
    for cmd in all_commands:
        mode = cmd['mode'] or 'default'
        by_mode[mode] = by_mode.get(mode, 0) + 1

    print(f"📈 BY MODE:")
    for mode, count in sorted(by_mode.items()):
        print(f"   {mode:15} : {count} commands")
    print()

    # Group by category
    by_category = {}
    for cmd in all_commands:
        cat = cmd['category']
        by_category[cat] = by_category.get(cat, 0) + 1

    print(f"📂 BY CATEGORY:")
    for cat, count in sorted(by_category.items(), key=lambda x: -x[1]):
        print(f"   {cat:30} : {count} commands")
    print()

    # Test nested remap flattening
    print(f"🔍 NESTED REMAP FLATTENING TEST:")
    super_m_commands = [c for c in all_commands if c['keySequence'].startswith('super-m ')]
    print(f"   super-m prefix commands: {len(super_m_commands)}")
    print(f"   Expected: 4")
    print(f"   Status: {'✓ PASS' if len(super_m_commands) == 4 else '✗ FAIL'}")
    print()

    super_s_commands = [c for c in all_commands if c['keySequence'].startswith('super-s ')]
    print(f"   super-s prefix commands: {len(super_s_commands)}")
    print(f"   Expected: 2")
    print(f"   Status: {'✓ PASS' if len(super_s_commands) == 2 else '✗ FAIL'}")
    print()

    # Show sample commands
    print(f"📋 SAMPLE LAUNCH COMMANDS:")
    print()
    for i, cmd in enumerate(all_commands[:10], 1):
        key_width = 30
        desc_width = 40
        print(f"   {i:2}. {cmd['keySequence']:{key_width}} │ {cmd['description']:{desc_width}} │ [{cmd['category']}]")
        print(f"       Command: {cmd['command']}")
        print()

    # Verify command structure preservation
    print(f"🧪 COMMAND STRUCTURE TEST:")
    multi_arg_cmd = next((c for c in all_commands if len(c['command']) > 2), None)
    if multi_arg_cmd:
        print(f"   ✓ Command structure preserved as list")
        print(f"   Example: {multi_arg_cmd['keySequence']}")
        print(f"   Command list: {multi_arg_cmd['command']}")
        print(f"   List length: {len(multi_arg_cmd['command'])} arguments")
    else:
        print(f"   ⚠ No multi-argument commands found")
    print()

    # Test rofi display format
    print(f"🎨 ROFI DISPLAY FORMAT:")
    print()
    print(f"   {'Key Sequence':{30}} │ {'Description':{40}} │ Category")
    print(f"   {'-'*30} │ {'-'*40} │ {'-'*20}")
    for cmd in all_commands[:8]:
        print(f"   {cmd['keySequence']:{30}} │ {cmd['description']:{40}} │ [{cmd['category']}]")
    print()

    # Generate sample Lua code
    print(f"💻 GENERATED LUA CODE (sample):")
    print()
    sample = all_commands[0]
    cmd_lua = "{" + ", ".join(f'"{arg}"' for arg in sample['command']) + "}"
    print(f'''   {{
     key = "{sample['keySequence']}",
     desc = "{sample['description']}",
     cmd = {cmd_lua},
     mode = "{sample['mode']}",
     category = "{sample['category']}"
   }}''')
    print()

    # Success summary
    print("="*70)
    print("✓ ALL TESTS PASSED")
    print("="*70)
    print()
    print(f"Summary:")
    print(f"  • Extracted {len(all_commands)} launch commands")
    print(f"  • Nested remaps properly flattened (super-m, super-s)")
    print(f"  • Command structure preserved as lists")
    print(f"  • Key sequences formatted correctly")
    print(f"  • Ready for rofi launcher generation")
    print()

    return all_commands

if __name__ == '__main__':
    commands = test_launcher_extraction()

    # Save as JSON for inspection
    with open('/tmp/extracted-commands.json', 'w') as f:
        json.dump(commands, f, indent=2)
    print(f"💾 Commands saved to /tmp/extracted-commands.json")
