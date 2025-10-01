#!/bin/bash

# Test script to demonstrate the disable/enable functionality

echo "Testing xremap disable/enable functionality"
echo "=========================================="

echo "To disable mappings, run:"
echo "echo 'disable' | nc -U /tmp/xremap_control.sock"
echo ""

echo "To enable mappings, run:"
echo "echo 'enable' | nc -U /tmp/xremap_control.sock"
echo ""

echo "Or using socat:"
echo "echo 'disable' | socat - UNIX-CONNECT:/tmp/xremap_control.sock"
echo "echo 'enable' | socat - UNIX-CONNECT:/tmp/xremap_control.sock"
echo ""

echo "The socket is created at: /tmp/xremap_control.sock"
echo "Commands: 'disable' to pause mappings, 'enable' to resume mappings"