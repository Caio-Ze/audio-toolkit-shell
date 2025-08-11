# Changelog

All notable changes to Audio Toolkit Shell will be documented in this file.

## [2.1.0] - 2025-08-10

### UI layout and rendering improvements
- Buttons panel seam and right-edge cutoff resolved via row-background prepass.
- Feature flag: `ATS_BTN_ROW_PREPASS` (default: true) toggles prepass; legacy per-cell background kept for fallback.
- Added diagnostics overlay flag `ATS_DEBUG_OVERLAY` (default: false) to visualize pane/splitter/seam geometry.

### Layout Plan v2
- Left column (Terminal 1 + Buttons) fixed to 40% of window width.
- Buttons container fixed to 35% of total page height within the left column; Terminal 1 uses the upper 65%.
- Right cluster retains interactive splitters between Terminals 2/3 and (2/3)/4; Terminal 4 resizable vertically.

### Focus, scrolling, and interaction fixes
- Broadened header click band for reliable click-to-focus and keyboard routing.
- Splitter handle rects moved below header band to avoid event overlap.
- Independent scrolling per terminal via stable, index-based ScrollArea ids.

### Documentation
- Updated `RESIZER_AND_LAYOUT_AUDIT.md` with implementation details, validation checklist, and results (validated at 100%/125% scale).
- Updated `README.md`, `CONFIGURATION.md`, and `TECHNICAL.md` to document flags and layout behavior.

### Code references
- Implementation in `src-tauri/src/app.rs` under the buttons panel renderer and layout logic.

## [2.0.0] - 2025-01-04

### 🎉 **MAJOR RELEASE: Native Rust Terminal Application**

Complete rewrite as a native Rust desktop application using eframe/egui for high-performance terminal workflows.

### ✅ **Added**
- **Native Rust GUI** using `eframe` and `egui` for maximum performance
- **Multi-tab terminal interface** with configurable tabs via TOML
- **Real PTY integration** using `portable-pty` for true terminal behavior
- **Interactive terminal sessions** with full keyboard input support
- **ANSI color processing** for clean, readable terminal output
- **Auto-restart functionality** with configurable success pattern detection
- **Character-by-character rendering** for accurate terminal display
- **Persistent PTY sessions** for robust multi-input handling

### 🔧 **Architecture**
- **Core**: Native Rust application with eframe/egui GUI framework
- **Terminal**: portable-pty for cross-platform PTY management
- **Configuration**: TOML-based configuration system
- **Threading**: Background threads for real-time output capture
- **Performance**: ~10-20MB memory usage, sub-200ms startup time

### 📋 **Configuration**
```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1280
window_height = 720

[[tabs]]
title = "Start Scripts"
command = "/path/to/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully"]
```

### 🎯 **Key Features**
- ✅ Multi-tab terminal interface
- ✅ Real PTY-backed terminal sessions
- ✅ TOML configuration for tabs and settings
- ✅ Interactive command execution
- ✅ ANSI color code support
- ✅ Auto-launch configured executables
- ✅ Success pattern detection for workflow automation

### 🚀 **Performance**
- **Startup**: ~100-200ms
- **Memory**: ~10-20MB base usage
- **CPU**: Minimal idle usage
- **Responsiveness**: Real-time PTY integration

### 📚 **Documentation**
- `README.md` - Project overview and setup
- `TECHNICAL.md` - Architecture and implementation details
- `CONFIGURATION.md` - Complete configuration guide
- `CHANGELOG.md` - Version history

### 🔮 **Future Enhancements**
- Catppuccin theme integration
- Enhanced text alignment
- Keyboard shortcuts
- Tab persistence
- Extended configuration options

---

## Version Numbering

- **Major (X.0.0)**: Breaking changes, architecture rewrites
- **Minor (0.X.0)**: New features, significant improvements
- **Patch (0.0.X)**: Bug fixes, minor improvements

## Support

For issues, questions, or contributions related to any version, please refer to the project repository and documentation.



