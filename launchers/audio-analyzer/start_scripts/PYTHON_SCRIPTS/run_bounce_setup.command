#!/bin/bash

# Double-click friendly launcher that ensures BounceT4 permissions and opens the menu.

set -e

# Resolve directory where this .command resides
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

INSTALLER="install_requirements_rust"
MENU="start_scripts_rust"

# Ensure installer is executable and de-quarantined
chmod +x "$INSTALLER" || true
if command -v xattr &>/dev/null; then
  for attr in com.apple.quarantine com.apple.provenance; do
    xattr -dr "$attr" "$INSTALLER" 2>/dev/null || true
  done
fi

# Run installer (will request sudo once)
"./$INSTALLER"

# After successful setup, launch the interactive menu
"./$MENU" 