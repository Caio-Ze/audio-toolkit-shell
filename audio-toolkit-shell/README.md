# Audio Toolkit Shell

A native Rust terminal emulator application built with `eframe` and `egui` for high-performance audio tool workflows. Provides a unified, multi-tab terminal interface for managing audio processing tools efficiently.

## 🚀 **Key Features**

### ✅ **Native Performance**
- Built entirely in Rust using `eframe` and `egui` for maximum performance
- Real PTY-backed terminal sessions using `portable-pty`
- Character-by-character terminal emulation with ANSI color support
- Sub-200ms startup time with minimal memory footprint

### ✅ **TOML Configuration**
- Configurable tabs via `config.toml`
- Per-tab executable auto-launch
- Success pattern detection for workflow automation
- Customizable window size and appearance

### ✅ **Interactive Terminals**
- Full keyboard input support for complex executables
- Real-time ANSI color code processing
- Persistent PTY sessions with background thread processing
- Multi-tab interface with independent terminal sessions

## 🏗️ **Architecture**

- **Core Technology**: Native Rust with `eframe` and `egui` GUI framework
- **Terminal Backend**: `portable-pty` for cross-platform PTY management
- **Configuration**: TOML-based configuration system
- **Threading**: Background reader threads for real-time output capture

## Getting Started
### Prerequisites
- **macOS**: Native target
- **Rust 1.77.2+**

### Installation
1. **Clone the Repository**
   ```bash
   git clone <repository-url>
   cd audio-toolkit-shell
   ```

2. **Build the Application**
   ```bash
   cd src-tauri
   cargo build --release
   ```

3. **Configure Tabs**
   Edit `config.toml` to configure your terminal tabs. The file is created next to the executable on first run. You can override the location with `ATS_CONFIG_DIR`.
   ```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1458.0
window_height = 713.0
right_top_fraction = 0.617            # vertical split: top (tabs 2/3) vs bottom (tab 4)
right_top_hsplit_fraction = 0.500     # horizontal split: tab 2 vs tab 3

[[tabs]]
title = "Start Scripts"
command = "/path/to/your/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]
[tabs.dnd]
auto_cd_on_folder_drop = false
auto_run_on_folder_drop = false
```

4. **Run the Application**
   ```bash
   cargo run
   ```

## Usage
1. **Launch**: Run `cargo run` to start the application
2. **Navigate**: Click a terminal area to focus it; Shift+Tab cycles focus
3. **Interact**: Use the input field to execute commands
4. **Workflow**: Configured tabs auto-launch executables
5. **Automation**: Auto-detects completion patterns for workflow automation

## 🧭 Layout Overview (Plan v2)

- Left column (Terminal 1 + Buttons) fixed to 40% of window width.
- Buttons container uses the lower 35% of the left column height; Terminal 1 uses the upper 65%.
- Right cluster (Terminals 2/3/4) retains interactive splitters: vertical split between top (2/3) and bottom (4), and a horizontal split between 2 and 3. Defaults come from `[app]` config (`right_top_fraction`, `right_top_hsplit_fraction`).

## 🎛️ Buttons Panel Rendering

- Row-background prepass renders a single opaque background per row across full width.
- Eliminates mid-column seam and right-edge clipping; buttons render content on top (no per-cell BG).
- Toggle via `ATS_BTN_ROW_PREPASS` (default: on). Legacy per-cell background path remains for fallback.

## 🧪 Runtime Flags / Environment Variables

- `ATS_BTN_ROW_PREPASS` (default: `true`)
  - Enables row-background prepass for the buttons grid.
  - Example: `ATS_BTN_ROW_PREPASS=0 cargo run --release` (disables prepass)

- `ATS_DEBUG_OVERLAY` (default: `false`)
  - Shows debug overlay for pane bounds, splitter handles, seam guides, and focus logs. Also enables window resize logs.
  - Example: `ATS_DEBUG_OVERLAY=1 cargo run --release`

- `ATS_WINDOW_TRACE` (default: `false`)
  - Enables window resize tracing without the overlay. Logs inner size (points/pixels) and suggested `[app]` values.
  - Example: `ATS_WINDOW_TRACE=1 cargo run --release`

- `ATS_CONFIG_DIR` (optional)
  - Overrides config directory for `config.toml`.
  - Example: `ATS_CONFIG_DIR=/tmp/ats-config cargo run --release`

## 🔧 **Development**

### Project Structure
```
audio-toolkit-shell/
├── src-tauri/
│   ├── src/
│   │   └── main.rs
│   ├── target/
│   └── Cargo.toml
└── README.md
```

### Key Dependencies
- `eframe` & `egui`: GUI framework
- `portable-pty`: PTY management
- `serde` & `toml`: Configuration parsing

### Building & Running
```bash
# Development
cargo build
cargo run

# Release
cargo build --release
ATS_DEBUG_OVERLAY=1 cargo run --release
```

## Contributing
1. Ensure you have Rust 1.77.2+ installed
2. Test on macOS for full compatibility
3. Follow Rust best practices

## License
MIT License - Refer to LICENSE file for details.

## 🎯 **Project Overview**

Audio Toolkit Shell is a native desktop application that provides an intelligent, multi-tab terminal interface. Each tab can be configured to automatically launch specific executables, creating a streamlined workflow for audio processing tools.

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
On first run, the app creates `config.toml` next to the executable (or uses `ATS_CONFIG_DIR` if set). Edit that file to configure your tabs:

```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1458.0
window_height = 713.0
right_top_fraction = 0.617
right_top_hsplit_fraction = 0.500

[[tabs]]
title = "Start Scripts"
command = "/path/to/your/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]
[tabs.dnd]
auto_cd_on_folder_drop = false
auto_run_on_folder_drop = false

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
- **`window_width`**: Initial window width (points)
- **`window_height`**: Initial window height (points)
- **`right_top_fraction`**: Vertical split fraction for right cluster (top vs bottom)
- **`right_top_hsplit_fraction`**: Horizontal split for top-right (tab 2 vs 3)

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

## 🖼️ Drag-and-Drop

- All drops (file/folder/app) go to the currently focused terminal tab.
- Per-tab behavior can auto-cd and optionally auto-run on folder drops via `[tabs.dnd]`:
  - `auto_cd_on_folder_drop` (bool)
  - `auto_run_on_folder_drop` (bool)

## 📐 Setting Default Size & Splits

Use the overlay/tracing to capture your preferred window size and splits and persist them to config:

- Guide: see `SETING_DEFAULT_SIZE.md`

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

## 🔧 **Current Status**

- **Functionality**: Character-by-character terminal emulation with real PTY backing
- **Performance**: Optimized for minimal resource usage and fast startup
- **Features**: Multi-tab interface with configurable executables and auto-restart
- **Platform**: Native macOS application with cross-platform Rust codebase

## 🤝 **Contributing**

1. Ensure you have Rust 1.77.2+ installed
2. All executables must be real (no mocks)
3. Test on macOS for full compatibility
4. Follow Rust best practices and error handling

## 📄 **License**

MIT License - see LICENSE file for details.

---

**Audio Toolkit Shell** - A native, high-performance multi-tab terminal for audio workflow automation.
