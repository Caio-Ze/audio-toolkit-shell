# Audio Toolkit Shell

A high-performance, native macOS multi-tab terminal application built entirely in Rust. Provides a unified interface for audio workflow tools with configurable tabs and automatic executable launching.

## 🎯 **Project Overview**

Audio Toolkit Shell is a native desktop application that replaces traditional terminal workflows with an intelligent, multi-tab interface. Each tab can be configured to automatically launch specific executables, creating a streamlined workflow for audio processing tools.

## 🚀 **Key Features**

### ✅ **Native Performance**
- Built with Rust using `eframe` and `egui` for maximum performance
- No web components or external dependencies
- Direct PTY integration for true terminal behavior

### ✅ **TOML Configuration**
- Configurable tabs via `config.toml`
- Per-tab executable auto-launch
- Customizable window size and appearance
- Success pattern detection for workflow automation

### ✅ **Interactive Terminals**
- Real PTY-backed terminal sessions using `portable-pty`
- Full interactive support for complex executables
- ANSI color code processing for clean output
- Persistent input/output handling

### ✅ **Workflow Automation**
- Auto-restart functionality (configurable per tab)
- Success pattern detection to return to main menus
- Seamless transitions between tools

## 🏗️ **Architecture**

```
Audio Toolkit Shell (Native Rust)
├── eframe/egui GUI Framework
├── portable-pty for Terminal Integration
├── TOML Configuration System
└── Multi-Tab Interface
    ├── Tab 1: Start Scripts (auto-launches executable)
    ├── Tab 2: Terminal (standard bash)
    └── Tab N: Configurable...
```

## 📋 **Requirements**

- **macOS** (native application)
- **Rust 1.77.2+**
- **Real executables** (no mock/safe scripts supported)

## 🛠️ **Installation & Setup**

### 1. Clone the Repository
```bash
git clone <repository-url>
cd audio-toolkit-shell
```

### 2. Build the Application
```bash
cd src-tauri
cargo build --release
```

### 3. Configure Tabs
Edit `src-tauri/config.toml` to configure your tabs:

```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1280
window_height = 720

[[tabs]]
title = "Start Scripts"
command = "/path/to/your/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]

[[tabs]]
title = "Terminal 2"
command = "bash"
auto_restart_on_success = false
success_patterns = []
```

### 4. Run the Application
```bash
cargo run
```

## ⚙️ **Configuration**

### Tab Configuration
Each `[[tabs]]` section in `config.toml` defines a terminal tab:

- **`title`**: Display name for the tab
- **`command`**: Executable path or shell command to run
- **`auto_restart_on_success`**: Whether to restart when success patterns are detected
- **`success_patterns`**: Text patterns that trigger auto-restart

### Application Settings
The `[app]` section configures the application:

- **`name`**: Window title
- **`window_width`**: Initial window width
- **`window_height`**: Initial window height

## 🎮 **Usage**

1. **Launch**: Run `cargo run` to start the application
2. **Navigate**: Click tabs to switch between terminals
3. **Interact**: Use the input field at the bottom to send commands
4. **Workflow**: Configured tabs will auto-launch their executables
5. **Automation**: Tools will automatically return to menus when complete (if configured)

## 🔧 **Development**

### Project Structure
```
audio-toolkit-shell/
├── src-tauri/
│   ├── src/
│   │   └── main.rs          # Main application code
│   ├── config.toml          # Tab configuration
│   └── Cargo.toml           # Rust dependencies
└── README.md
```

### Key Dependencies
- `eframe` & `egui`: Native GUI framework
- `portable-pty`: Cross-platform PTY handling
- `serde` & `toml`: Configuration parsing

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run with debug output
cargo run
```

## 🚫 **Important Constraints**

- **No Mock Scripts**: All modes use real executables (no development/testing mocks)
- **Native Only**: This is a desktop application, not a web/browser app
- **macOS Target**: Optimized for macOS (though Rust code is cross-platform)

## 📝 **Example Workflows**

### Audio Processing Workflow
1. Tab 1 auto-launches script menu
2. Select audio processing tool
3. Tool runs and processes files
4. On completion, automatically returns to menu
5. Select next tool seamlessly

### Development Workflow
1. Configure tabs for different development tools
2. Switch between build, test, and deployment tools
3. Each tab maintains its own session and state

## 🎉 **Migration from Previous Architecture**

This version represents a complete rewrite from the previous Tauri + React architecture:

### ✅ **Removed**
- Entire React/TypeScript frontend
- Tauri backend framework
- Node.js dependencies
- Web-based terminal components

### ✅ **Added**
- Native Rust GUI with eframe/egui
- Direct PTY integration
- TOML-based configuration
- Simplified, high-performance architecture

## 🤝 **Contributing**

1. Ensure you have Rust 1.77.2+ installed
2. All executables must be real (no mocks)
3. Test on macOS for full compatibility
4. Follow Rust best practices and error handling

## 📄 **License**

MIT License - see LICENSE file for details.

---

**Audio Toolkit Shell** - A native, high-performance multi-tab terminal for audio workflow automation.
