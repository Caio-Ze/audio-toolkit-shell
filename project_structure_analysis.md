# Audio Toolkit Shell - Project Structure Analysis

## CRITICAL SCOPE CLARIFICATION
**The `launchers/` folder is OUTSIDE the refactoring scope** - it contains separate projects that are only launched by the main audio-toolkit-shell project. The refactoring focuses ONLY on the `audio-toolkit-shell/` directory.

## Core Project Files (audio-toolkit-shell/) - REFACTORING SCOPE

### Main Application Files
- `audio-toolkit-shell/src-tauri/src/main.rs` - Main Rust application entry point
- `audio-toolkit-shell/src-tauri/Cargo.toml` - Rust dependencies and build configuration
- `audio-toolkit-shell/src-tauri/config.toml` - Application configuration file

### Documentation
- `audio-toolkit-shell/README.md` - Main project documentation
- `audio-toolkit-shell/CHANGELOG.md` - Version history
- `audio-toolkit-shell/CONFIGURATION.md` - Configuration documentation
- `audio-toolkit-shell/TECHNICAL.md` - Technical documentation

### Confirmed Legacy Files (To Be Removed)
- `audio-toolkit-shell/src-tauri/fix_terminal_ui.py` - Python UI fix script
- `audio-toolkit-shell/src-tauri/fix_ui_complete.py` - Python completion fix script
- `audio-toolkit-shell/src-tauri/fix_ui_rendering.py` - Python rendering fix script
- `audio-toolkit-shell/dev-mocks/mock_launcher.sh` - Development mock script
- `audio-toolkit-shell/test_wrapper.sh` - Test wrapper script (root)
- `audio-toolkit-shell/src-tauri/test_wrapper.sh` - Test wrapper script (src-tauri)

### Tauri Infrastructure (To Be Evaluated)
- `audio-toolkit-shell/src-tauri/capabilities/` - Tauri capabilities configuration
- `audio-toolkit-shell/src-tauri/gen/` - Generated Tauri files
- `audio-toolkit-shell/src-tauri/icons/` - Application icons
- `audio-toolkit-shell/src-tauri/target/` - Rust build artifacts

### System Files (To Be Cleaned)
- Multiple `.DS_Store` files throughout the project
- Build artifacts in `target/` directory

## External Tools (launchers/) - OUTSIDE REFACTORING SCOPE

**These are separate projects launched by the main application:**
- `launchers/audio-analyzer/start_scripts/PYTHON_SCRIPTS/` - Python-based audio processing scripts
- `launchers/audio-analyzer/start_scripts/RUST_SUPERFASTNORMALIZER/` - Rust audio normalizer tools
- `launchers/audio-analyzer/start_scripts/SESSSION_MONITOR_PT_SESSIONS/` - Pro Tools session monitoring

## Key Findings

1. **Current Architecture**: Native Rust application using eframe/egui with complete terminal functionality
2. **Working Implementation**: Application builds and runs successfully with multi-tab terminal interface
3. **Configuration System**: TOML-based config references executables in launchers/ directory (separate projects)
4. **Legacy Python Scripts**: Three Python fix scripts in src-tauri directory (confirmed legacy)
5. **Mock Scripts**: Development mock files that conflict with "no mock scripts" requirement
6. **System Files**: Multiple .DS_Store files need cleanup
7. **Build Artifacts**: Large target directory with compiled dependencies
8. **Tauri Infrastructure**: May contain unused files from previous architecture migration

## Baseline Verification Results
✅ **Rust Application**: Builds successfully with `cargo build`
✅ **Code Quality**: Passes `cargo check` without errors  
✅ **Architecture**: Complete eframe/egui implementation with terminal emulation
✅ **Configuration**: Valid config.toml with proper tab configurations
✅ **External Dependencies**: Correctly references separate launcher projects

## Files Requiring Analysis
1. **Python fix scripts** - verify they're not referenced in Rust code (confirmed legacy)
2. **Mock and test wrapper scripts** - confirm they're not needed (conflicts with requirements)
3. **Tauri configuration directories** - determine if still needed after migration to native Rust
4. **Documentation files** - ensure they reflect current native Rust architecture
5. **System files** - clean up .DS_Store files and update .gitignore