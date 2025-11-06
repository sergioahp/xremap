#!/bin/bash
# Example rofi integration script for xremap keybindings
# Usage: ./show-keybindings.sh

BINDINGS_FILE="${BINDINGS_FILE:-$HOME/.config/xremap/bindings.yml}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# If bindings file doesn't exist, try to generate it from example
if [ ! -f "$BINDINGS_FILE" ]; then
    echo "Bindings file not found at $BINDINGS_FILE"
    echo "Generating from example config..."

    EXAMPLE_CONFIG="$SCRIPT_DIR/config_with_descriptions.yml"

    if [ ! -f "$EXAMPLE_CONFIG" ]; then
        echo "Error: Example config not found at $EXAMPLE_CONFIG"
        exit 1
    fi

    # Generate bindings file in /tmp for demo
    BINDINGS_FILE="/tmp/xremap_bindings.yml"

    if command -v xremap-config-splitter &> /dev/null; then
        xremap-config-splitter "$EXAMPLE_CONFIG" \
            -c /tmp/xremap_clean.yml \
            -d "$BINDINGS_FILE"
    elif [ -f "$SCRIPT_DIR/../target/debug/xremap-config-splitter" ]; then
        "$SCRIPT_DIR/../target/debug/xremap-config-splitter" "$EXAMPLE_CONFIG" \
            -c /tmp/xremap_clean.yml \
            -d "$BINDINGS_FILE"
    else
        echo "Error: xremap-config-splitter not found"
        echo "Please build it first: cargo build --bin xremap-config-splitter"
        exit 1
    fi
fi

# Check if yq is available
if ! command -v yq &> /dev/null; then
    echo "Error: yq is required but not installed"
    echo "Install with: sudo snap install yq"
    echo "Or: brew install yq"
    exit 1
fi

# Check if rofi is available
if ! command -v rofi &> /dev/null; then
    echo "Error: rofi is required but not installed"
    echo "Install with: sudo apt install rofi"
    echo "Or: brew install rofi"
    echo ""
    echo "Falling back to terminal output..."

    # Terminal fallback
    echo "=== XREMAP KEYBINDINGS ==="
    yq -r '.bindings[] |
        "\(.binding)\t\(.description // "No description")\t[\(.name // "General")] \(.mode // "")"' \
        "$BINDINGS_FILE" | \
        column -t -s $'\t' | \
        less
    exit 0
fi

# Main rofi interface
SELECTION=$(yq -r '.bindings[] |
    "\(.binding)\t│ \(.description // "No description")\t│ [\(.name // "General")]"' \
    "$BINDINGS_FILE" | \
    column -t -s $'\t' | \
    rofi -dmenu -i \
        -p "Search keybindings" \
        -mesg "Browse xremap keybindings by description or binding" \
        -theme-str 'window {width: 80%;}' \
        -theme-str 'listview {lines: 20;}' \
        -theme-str 'element-text {horizontal-align: 0;}' \
        -no-custom)

if [ -n "$SELECTION" ]; then
    # Extract binding from selection
    BINDING=$(echo "$SELECTION" | awk '{print $1}')

    # Show detailed info
    yq -r ".bindings[] | select(.binding == \"$BINDING\") |
        \"Binding: \(.binding)
Description: \(.description // \"No description\")
Action: \(.action | tostring)
Mode: \(.mode // \"default\")
Category: \(.name // \"General\")
Application: \(.application // \"All\" | tostring)\"" \
        "$BINDINGS_FILE" | \
        rofi -dmenu -p "Binding Details" -mesg "Press Esc to close" -no-custom > /dev/null
fi
