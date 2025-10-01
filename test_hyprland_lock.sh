#!/bin/bash

# Test script to demonstrate the Hyprland lock integration

echo "Testing xremap Hyprland lock integration"
echo "========================================="

echo "Building xremap with Hyprland support..."
cargo build --features hypr

echo ""
echo "To test the integration:"
echo "1. Run xremap with Hyprland support:"
echo "   ./target/debug/xremap --features hypr [your-config.yaml]"
echo ""
echo "2. Lock your Hyprland session (usually Super+L or hyprlock)"
echo "3. You should see 'Hyprland screen locked - disabling key mappings'"
echo "4. Unlock your session"
echo "5. You should see 'Hyprland screen unlocked - enabling key mappings'"
echo ""
echo "When locked, xremap will pass through all key events without remapping."
echo "When unlocked, normal key remapping functionality resumes."