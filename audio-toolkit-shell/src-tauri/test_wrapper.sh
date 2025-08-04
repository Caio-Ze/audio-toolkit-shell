#!/bin/bash

# Simple wrapper script to test execution
echo "🔧 Test Wrapper: Starting..."
echo "🔧 Test Wrapper: About to call start_scripts_rust"
echo ""

# Call your actual executable and send "20" (Exit) as input
echo "20" | /Users/caioraphael/Desktop/BOUNCET4/PYTHON_SCRIPTS/start_scripts_rust

exit_code=$?
echo ""
echo "🔧 Test Wrapper: start_scripts_rust finished with exit code: $exit_code"