#!/usr/bin/env python3
"""
Simple verification script for Nix config splitter logic.
This doesn't run actual Nix code but validates the transformation logic.
"""

import json
from typing import Any, Dict, List, Optional

def extract_description(action: Any) -> tuple[bool, Optional[str], Any]:
    """Extract description from an action."""
    if isinstance(action, dict) and 'description' in action:
        desc = action['description']
        clean = {k: v for k, v in action.items() if k != 'description'}
        # If only 'action' remains, unwrap it
        if len(clean) == 1 and 'action' in clean:
            return True, desc, clean['action']
        return True, desc, clean
    return False, None, action

def process_remap_entry(key: str, action: Any) -> Dict[str, Any]:
    """Process a single remap entry."""
    has_desc, desc, clean_action = extract_description(action)
    return {
        'binding': key,
        'cleanAction': clean_action,
        'description': desc
    }

def process_remap(remap: Dict[str, Any]) -> tuple[Dict[str, Any], List[Dict[str, Any]]]:
    """Process a remap attribute set."""
    entries = [process_remap_entry(k, v) for k, v in remap.items()]
    clean_remap = {e['binding']: e['cleanAction'] for e in entries}
    bindings = [{
        'binding': e['binding'],
        'description': e['description'],
        'action': e['cleanAction']
    } for e in entries]
    return clean_remap, bindings

def process_keymap_item(item: Dict[str, Any]) -> tuple[Dict[str, Any], List[Dict[str, Any]]]:
    """Process a keymap or modmap item."""
    clean_remap, bindings = process_remap(item.get('remap', {}))

    # Extract metadata
    metadata = {
        'name': item.get('name'),
        'mode': item.get('mode'),
        'application': item.get('application'),
        'window': item.get('window'),
        'device': item.get('device'),
        'exact_match': item.get('exact_match')
    }

    # Filter out None values
    metadata = {k: v for k, v in metadata.items() if v is not None}

    # Add metadata to bindings
    enriched_bindings = [
        {**binding, **metadata}
        for binding in bindings
    ]

    # Create clean item
    clean_item = {k: v for k, v in item.items() if k != 'remap'}
    clean_item['remap'] = clean_remap

    return clean_item, enriched_bindings

def split_config(config: Dict[str, Any]) -> Dict[str, Any]:
    """Main function: split config into clean and dmenu."""
    # Process modmap
    modmap_items = config.get('modmap', [])
    modmap_results = [process_keymap_item(item) for item in modmap_items]
    clean_modmap = [r[0] for r in modmap_results]
    modmap_bindings = [b for r in modmap_results for b in r[1]]

    # Process keymap
    keymap_items = config.get('keymap', [])
    keymap_results = [process_keymap_item(item) for item in keymap_items]
    clean_keymap = [r[0] for r in keymap_results]
    keymap_bindings = [b for r in keymap_results for b in r[1]]

    # Combine
    all_bindings = modmap_bindings + keymap_bindings

    # Create clean config
    clean_config = {k: v for k, v in config.items() if k not in ['modmap', 'keymap']}
    if modmap_items:
        clean_config['modmap'] = clean_modmap
    if keymap_items:
        clean_config['keymap'] = clean_keymap

    return {
        'cleanConfig': clean_config,
        'dmenuBindings': {
            'bindings': all_bindings,
            'count': len(all_bindings)
        }
    }

# Test cases
def test_simple_keymap():
    print("\n=== Test 1: Simple keymap ===")
    config = {
        'keymap': [{
            'name': 'Basic',
            'remap': {
                'C-a': 'Home',
                'C-e': 'End'
            }
        }]
    }
    result = split_config(config)
    assert result['dmenuBindings']['count'] == 2
    assert result['cleanConfig']['keymap'][0]['remap']['C-a'] == 'Home'
    print("✓ PASS")

def test_with_descriptions():
    print("\n=== Test 2: With descriptions ===")
    config = {
        'keymap': [{
            'name': 'Launch',
            'mode': 'default',
            'remap': {
                'Super-t': {
                    'launch': ['xterm'],
                    'description': 'Open terminal'
                }
            }
        }]
    }
    result = split_config(config)
    binding = result['dmenuBindings']['bindings'][0]
    assert binding['description'] == 'Open terminal'
    assert binding['name'] == 'Launch'
    assert binding['mode'] == 'default'
    # Check clean config has no description
    clean_remap = result['cleanConfig']['keymap'][0]['remap']['Super-t']
    assert 'description' not in clean_remap
    assert clean_remap['launch'] == ['xterm']
    print("✓ PASS")

def test_application_filter():
    print("\n=== Test 3: Application filter ===")
    config = {
        'keymap': [{
            'name': 'Emacs',
            'application': {'not': ['Emacs', 'Terminal']},
            'remap': {
                'C-a': {
                    'action': 'Home',
                    'description': 'Beginning of line'
                }
            }
        }]
    }
    result = split_config(config)
    binding = result['dmenuBindings']['bindings'][0]
    assert 'application' in binding
    assert binding['application']['not'] == ['Emacs', 'Terminal']
    assert result['cleanConfig']['keymap'][0]['application']['not'] == ['Emacs', 'Terminal']
    print("✓ PASS")

def test_modmap():
    print("\n=== Test 4: Modmap ===")
    config = {
        'modmap': [{
            'name': 'Global',
            'remap': {
                'CapsLock': 'Esc',
                'Alt_L': 'Ctrl_L'
            }
        }]
    }
    result = split_config(config)
    assert result['dmenuBindings']['count'] == 2
    assert 'modmap' in result['cleanConfig']
    assert result['cleanConfig']['modmap'][0]['remap']['CapsLock'] == 'Esc'
    print("✓ PASS")

def test_mixed():
    print("\n=== Test 5: Mixed modmap and keymap ===")
    config = {
        'default_mode': 'default',
        'modmap': [{
            'name': 'Global',
            'remap': {'CapsLock': 'Esc'}
        }],
        'keymap': [{
            'name': 'Launch',
            'remap': {
                'Super-t': {
                    'launch': ['xterm'],
                    'description': 'Terminal'
                }
            }
        }]
    }
    result = split_config(config)
    assert result['dmenuBindings']['count'] == 2
    assert 'modmap' in result['cleanConfig']
    assert 'keymap' in result['cleanConfig']
    assert result['cleanConfig']['default_mode'] == 'default'
    print("✓ PASS")

def test_complex_actions():
    print("\n=== Test 6: Complex actions ===")
    config = {
        'keymap': [{
            'name': 'Advanced',
            'remap': {
                'C-space': {
                    'action': {'set_mark': True},
                    'description': 'Set mark'
                },
                'C-x': {
                    'action': {
                        'remap': {
                            'C-c': 'Esc',
                            'C-s': 'C-w'
                        }
                    },
                    'description': 'C-x prefix'
                }
            }
        }]
    }
    result = split_config(config)
    assert result['dmenuBindings']['count'] == 2, f"Expected 2 bindings, got {result['dmenuBindings']['count']}"
    # Check nested remap is preserved
    clean_cx = result['cleanConfig']['keymap'][0]['remap']['C-x']
    assert isinstance(clean_cx, dict), f"C-x should be dict, got {type(clean_cx)}"
    # The 'action' wrapper is removed, so nested remap should be at top level
    assert 'remap' in clean_cx, f"C-x should have 'remap', got keys: {clean_cx.keys()}"
    assert clean_cx['remap']['C-c'] == 'Esc', f"Expected 'Esc', got {clean_cx['remap']['C-c']}"
    print("✓ PASS")

def test_empty_config():
    print("\n=== Test 7: Empty config ===")
    config = {}
    result = split_config(config)
    assert result['dmenuBindings']['count'] == 0
    assert result['cleanConfig'] == {}
    print("✓ PASS")

if __name__ == '__main__':
    print("\n" + "="*50)
    print("NIX CONFIG SPLITTER - LOGIC VERIFICATION")
    print("="*50)

    tests = [
        test_simple_keymap,
        test_with_descriptions,
        test_application_filter,
        test_modmap,
        test_mixed,
        test_complex_actions,
        test_empty_config
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            print(f"✗ FAIL: {e}")
            failed += 1
        except Exception as e:
            print(f"✗ ERROR: {e}")
            failed += 1

    print("\n" + "="*50)
    print(f"Results: {passed} passed, {failed} failed")
    print("="*50 + "\n")

    if failed == 0:
        print("🎉 All tests passed!")
        exit(0)
    else:
        exit(1)
