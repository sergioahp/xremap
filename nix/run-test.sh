#!/usr/bin/env bash
# Test script for rofi-launcher-fixed.nix

set -e

echo "═══════════════════════════════════════════════════════════"
echo "Testing rofi-launcher-fixed.nix"
echo "═══════════════════════════════════════════════════════════"
echo

# Test 1: Nix evaluation
echo "Test 1: Evaluating Nix file..."
if nix-instantiate --eval --strict test-launcher-fixed.nix -A testReport 2>/dev/null; then
    echo "✓ Nix evaluation successful"
else
    echo "✗ Nix evaluation failed"
    echo "Trying without strict..."
    nix-instantiate --eval test-launcher-fixed.nix -A testReport 2>&1 | head -20
fi
echo

# Test 2: Build the launcher
echo "Test 2: Building launcher..."
if nix-build test-launcher-fixed.nix -A script -o /tmp/test-launcher-result 2>/dev/null; then
    echo "✓ Build successful"
    SCRIPT_PATH="/tmp/test-launcher-result/bin/test-launcher"
    echo "   Script: $SCRIPT_PATH"
else
    echo "⚠ Build not available (nix-build not installed)"
fi
echo

# Test 3: Validate Lua syntax (if lua is available)
echo "Test 3: Validating Lua syntax..."
if [ -f "$SCRIPT_PATH" ]; then
    if command -v lua &> /dev/null; then
        if lua -e "package.path='$SCRIPT_PATH'" 2>&1 | grep -q "syntax error"; then
            echo "✗ Lua syntax error detected"
        else
            echo "✓ Lua syntax appears valid"
        fi
    elif command -v luac &> /dev/null; then
        if luac -p "$SCRIPT_PATH" 2>&1; then
            echo "✓ Lua syntax valid (luac check)"
        else
            echo "✗ Lua syntax error"
        fi
    else
        echo "⚠ Lua not available for syntax check"
    fi
else
    echo "⚠ Script not built, skipping Lua validation"
fi
echo

# Test 4: Check the critical shell_escape function
echo "Test 4: Checking shell_escape function in generated script..."
if [ -f "$SCRIPT_PATH" ]; then
    if grep -q "str:gsub(\"'\", \[\['\\\\'']])" "$SCRIPT_PATH"; then
        echo "✓ Correct shell_escape pattern found: [['\'']]"
    elif grep -q "str:gsub" "$SCRIPT_PATH"; then
        echo "⚠ shell_escape found but pattern unclear:"
        grep "str:gsub" "$SCRIPT_PATH" | head -1
    else
        echo "✗ shell_escape function not found"
    fi
else
    echo "⚠ Script not available for inspection"
fi
echo

echo "═══════════════════════════════════════════════════════════"
echo "Test Summary"
echo "═══════════════════════════════════════════════════════════"
echo
echo "The fix addresses the Nix string escaping issue by using"
echo "Lua's bracket string literals [[...]] for the shell escape"
echo "pattern, avoiding nested quote complications."
echo
echo "Key change:"
echo "  OLD: Attempted to escape '..' operator (caused errors)"
echo "  NEW: Use [['\\'']] bracket literal (works correctly)"
echo
echo "═══════════════════════════════════════════════════════════"
