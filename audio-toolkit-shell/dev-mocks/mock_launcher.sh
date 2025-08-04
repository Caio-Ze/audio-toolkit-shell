#!/bin/bash

# Mock launcher script for safe development testing
# This script simulates a launcher without doing any real work

LAUNCHER_NAME="${1:-Mock Launcher}"

echo "🚀 Starting $LAUNCHER_NAME..."
echo "📋 Available options:"
echo "  1) Mock Script 1 - Safe test script"
echo "  2) Mock Script 2 - Another safe test"
echo "  3) Mock Script 3 - Harmless demo"
echo "  4) Exit"
echo ""

while true; do
    echo -n "Select option (1-4): "
    read -r choice
    
    case $choice in
        1)
            echo "✅ Running Mock Script 1..."
            echo "   This is a safe test that does nothing harmful"
            echo "   Processing... (fake work)"
            sleep 2
            echo "   ✅ Mock Script 1 completed successfully!"
            ;;
        2)
            echo "✅ Running Mock Script 2..."
            echo "   Another harmless test script"
            echo "   Simulating work... (no real actions)"
            sleep 1
            echo "   ✅ Mock Script 2 finished!"
            ;;
        3)
            echo "✅ Running Mock Script 3..."
            echo "   Demo script - completely safe"
            echo "   Fake processing..."
            sleep 1.5
            echo "   ✅ Mock Script 3 done!"
            ;;
        4|exit|quit)
            echo "👋 Exiting $LAUNCHER_NAME"
            exit 0
            ;;
        *)
            echo "❌ Invalid option. Please choose 1-4."
            ;;
    esac
    echo ""
done
