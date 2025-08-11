# Technical Documentation

## Architecture Overview

Audio Toolkit Shell is built as a native Rust application using the eframe/egui ecosystem for GUI and portable-pty for terminal integration. The application features a character-by-character terminal emulator with real PTY backing for authentic terminal behavior.

## Core Components

### 1. High-level structure

- **UI shell**: `src-tauri/src/app.rs`
  - Layout, splitters, focus handling, buttons panel rendering.
  - Single-pass DnD routing to the focused terminal.
- **Configuration**: `src-tauri/src/config.rs`
  - TOML parsing, defaults, and first-run template creation.
  - Per-tab settings including `[tabs.dnd]`.
- **Terminal tabs**: PTY-backed sessions (portable-pty) with background reader threads.
- **Terminal emulator**: Character-by-character rendering and ANSI handling.

### 2. Configuration System

The application uses TOML for configuration with the following structure:

```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1458.0
window_height = 713.0
# Initial right cluster splits (interactive at runtime)
right_top_fraction = 0.617            # vertical split: top (tabs 2/3) vs bottom (tab 4)
right_top_hsplit_fraction = 0.500     # horizontal split: tab 2 vs tab 3
min_left_width = 120.0
min_right_width = 120.0
allow_zero_collapse = false

[[tabs]]
title = "Start Scripts"
command = "/path/to/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]
[tabs.dnd]
auto_cd_on_folder_drop = false        # if true, drop of a folder will auto-insert cd '<dir>' and Enter
auto_run_on_folder_drop = false       # if true (and auto_cd is false), insert '<dir>' and press Enter
```

Config file location:
- Created next to the executable on first run; override with `ATS_CONFIG_DIR`.
- Example: `ATS_CONFIG_DIR=/tmp/ats-config cargo run --release`.

### 3. PTY Integration

Each terminal tab uses `portable-pty` for true terminal behavior:

- **PTY Creation**: Each tab creates its own PTY pair (master/slave)
- **Command Execution**: Commands are executed via `bash -c` for absolute paths
- **Output Capture**: Background threads read PTY output via channels
- **Input Handling**: Persistent PTY writer for user input

### 4. Terminal Emulation

The application implements a character-based terminal emulator:

```rust
struct TerminalCell {
    character: char,
    color: egui::Color32,
    bold: bool,
}
```

- **Character-by-character rendering** for accurate display
- **ANSI escape sequence processing** for colors and formatting
- **Cursor management** with proper positioning
- **Buffer management** with configurable dimensions

### 5. Threading Model

```
Main Thread (GUI)
├── Tab 1 Reader Thread → Channel → Terminal Emulator → UI Update
├── Tab 2 Reader Thread → Channel → Terminal Emulator → UI Update
└── Tab N Reader Thread → Channel → Terminal Emulator → UI Update
```

## Implementation Details

### Drag-and-drop model (current)

- Routing: all drops (file/folder/app) are sent to the currently focused terminal tab, regardless of drop position.
- Per-tab behavior: folder-drop actions are controlled under `[tabs.dnd]`:
  - `auto_cd_on_folder_drop`: `cd '<dir>'` then Enter.
  - `auto_run_on_folder_drop`: insert `'<dir>'` then Enter (no `cd`).
  - Precedence: `auto_cd_on_folder_drop` > `auto_run_on_folder_drop` > default (quoted path + trailing space, no Enter).
- Visuals: focused terminal shows a crisp 2px blue border; while dragging over the app, a subtle 4px glow is added to the focused panel.
- Tracing: enable `ATS_DND_TRACE=1` to log DnD events for debugging.

### PTY Command Execution

```rust
let cmd = if config.command.contains('/') {
    // Absolute path - run through bash
    let mut cmd = CommandBuilder::new("bash");
    cmd.arg("-c");
    cmd.arg(&config.command);
    cmd
} else {
    // Command name - run in shell
    CommandBuilder::new("bash")
};
```

### Output Processing

1. **Raw PTY Output**: Captured in background threads
2. **ANSI Processing**: Escape sequences parsed for colors and formatting
3. **Terminal Emulation**: Characters rendered to terminal buffer with proper positioning
4. **Pattern Detection**: Success patterns monitored for auto-restart
5. **UI Update**: Terminal buffer rendered to GUI with character-level precision

### Terminal Rendering Semantics

- DEC autowrap: Implemented with a `wrap_pending` flag. After reaching the last column, wrapping occurs on the next printable character (matches DEC autowrap semantics).
- Clearing on cursor moves/CR: After `\r` (carriage return) or cursor movement via CSI `H/f` (CUP), `G` (CHA), or `d` (VPA), the emulator marks `cursor_recently_positioned`. The first non-whitespace printable that follows will clear to end-of-line to prevent contamination.
- Border-preserving EOL clear: EOL clearing preserves the last column if it contains a border glyph (box-drawing U+2500..U+257F or ASCII `|`), preventing accidental erasure of the right frame line.
- CSI support: `H/f` (CUP), `G` (CHA), `d` (VPA) for positioning, and `X` (ECH) for clearing N cells from the cursor are implemented and validated by tests.
- Wide glyphs/emojis: Character width uses `unicode-width` with explicit emoji ranges forced to width=2. Wide chars are represented as a lead cell plus a placeholder in the buffer. The UI renderer draws wide glyphs as fixed two-cell spacers to preserve alignment and avoid right border breakage. A container-based emoji renderer (clip/scale inside two cells) is planned as an optional mode.

### Auto-Restart Logic

```rust
// Check for success patterns if auto-restart is enabled
if self.config.auto_restart_on_success {
    for pattern in &self.config.success_patterns {
        if cleaned.contains(pattern) {
            self.needs_restart = true;
            break;
        }
    }
}
```

## Key Design Decisions

### 1. Native vs Web
- **Chosen**: Native Rust with eframe/egui
- **Rationale**: Maximum performance, no web overhead, true terminal integration
- **Trade-off**: Platform-specific but much faster

### 2. PTY Integration
- **Chosen**: `portable-pty` with background reader threads
- **Rationale**: True terminal behavior, cross-platform compatibility
- **Trade-off**: More complex than simple process spawning but much more capable

### 3. Configuration Format
- **Chosen**: TOML
- **Rationale**: Human-readable, Rust ecosystem support, structured
- **Trade-off**: Requires parsing but very flexible

### 4. Threading Strategy
- **Chosen**: One reader thread per PTY
- **Rationale**: Isolation, real-time output, non-blocking UI
- **Trade-off**: More threads but better responsiveness

## Performance Characteristics

### Memory Usage
- **Base Application**: ~10-20MB
- **Per Tab**: ~1-2MB additional
- **PTY Buffers**: 8KB per reader thread

### CPU Usage
- **Idle**: Minimal (GUI refresh only)
- **Active Terminal**: Low (PTY I/O + text rendering)
- **Multiple Tabs**: Linear scaling per active tab

### Startup Time
- **Cold Start**: ~100-200ms
- **Configuration Load**: ~1-5ms
- **PTY Setup**: ~10-50ms per tab

## Error Handling

### PTY Errors
- Resource exhaustion handled gracefully
- Fallback to bash shell if command fails
- Proper cleanup on tab destruction

### Configuration Errors
- Invalid TOML falls back to defaults
- Missing executables logged but don't crash app
- Malformed patterns ignored

### UI Errors
- Channel disconnections handled
- Thread panics isolated per tab
- GUI state always recoverable

## Development Guidelines

### Adding New Features
1. Update `TabConfig` struct if needed
2. Modify TOML schema documentation
3. Add configuration validation
4. Test with real executables

### Performance Optimization
1. Minimize string allocations in hot paths
2. Use efficient text rendering in egui
3. Batch PTY output updates when possible
4. Profile memory usage with multiple tabs

### Testing Strategy
1. **Unit Tests**: Configuration parsing, ANSI stripping
2. **Integration Tests**: PTY behavior with real executables

## Environment variables

- `ATS_DEBUG_OVERLAY=1`: Shows overlay (pane bounds, splitters, focus logs) and enables window resize logs.
- `ATS_WINDOW_TRACE=1`: Window resize tracing without overlay; prints inner size (points/pixels) and suggested `[app]` defaults.
- `ATS_CONFIG_DIR=/path`: Override directory for `config.toml`.
- `ATS_DND_TRACE=1`: Drag-and-drop tracing logs.

## Window size and split tuning

- Use the overlay or `ATS_WINDOW_TRACE=1` to capture size/scale changes and suggested config values.
- Update your `config.toml` or code defaults accordingly.
- See `SETING_DEFAULT_SIZE.md` for the complete step-by-step guide.
3. **Manual Testing**: Full workflows on target platform
4. **Performance Tests**: Memory/CPU usage under load

## Troubleshooting

### Common Issues

1. **PTY Resource Exhaustion**
   - Symptom: "Resource temporarily unavailable"
   - Solution: Proper PTY cleanup, limit concurrent tabs

2. **Executable Not Found**
   - Symptom: Empty terminal output
   - Solution: Verify absolute paths, check permissions

3. **ANSI Display Issues**
   - Symptom: Escape codes in output
   - Solution: Improve ANSI stripping regex

4. **Input Not Working**
   - Symptom: Commands not reaching executable
   - Solution: Check PTY writer lifecycle

### Debug Mode
Run with debug output:
```bash
RUST_LOG=debug cargo run
```

### Performance Profiling
```bash
cargo build --release
cargo run --release
```

## Future Enhancements

### Planned Features
1. **Theme Support**: Configurable colors and fonts
2. **Tab Persistence**: Save/restore tab states
3. **Hotkeys**: Keyboard shortcuts for tab switching
4. **Search**: Find in terminal output
5. **Export**: Save terminal sessions

### Architecture Improvements
1. **Plugin System**: Dynamic tab types
2. **IPC**: Inter-tab communication
3. **Scripting**: Lua/JavaScript automation
4. **Remote**: SSH/network terminals

## Dependencies

### Core Dependencies
- `eframe = "0.27.2"` - GUI framework
- `egui = "0.27.2"` - Immediate mode GUI
- `portable-pty = "0.9.0"` - PTY integration
- `serde = "1.0"` - Serialization
- `toml = "0.8"` - Configuration parsing

### Development Dependencies
- `tempfile = "3.8"` - Testing utilities

## Build Configuration

### Release Build
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Development Build
```toml
[profile.dev]
opt-level = 0
debug = true
```

## UI Layout & Buttons Rendering

- __Layout (Plan v2)__
  - Left column (Terminal 1 + Buttons) is fixed at 40% of window width.
  - Buttons container occupies the lower 35% of the left column height; Terminal 1 uses the upper 65%.
  - Right cluster (Terminals 2/3/4) uses interactive splitters: vertical split between top (2/3) and bottom (4), and a horizontal split between 2 and 3. Defaults sourced from `[app]` config: `right_top_fraction` and `right_top_hsplit_fraction`.

- __Buttons panel rendering__
  - Row-background prepass paints a single opaque background per row across the full width, eliminating mid-column seams and right-edge clipping.
  - Buttons render only content (labels, accents, hover/pressed) atop the row background; per-cell backgrounds are disabled in this mode.
  - Implementation in `src-tauri/src/app.rs` around the buttons renderer.

## Drag-and-Drop Model

- __Routing__
  - All file/folder/app drops are routed to the currently focused terminal tab, regardless of where the drop lands in the UI.
  - The focused terminal shows a subtle blue focus border; during drag-hover it glows.

- __Per-tab behavior__ (`[tabs.dnd]`)
  - `auto_cd_on_folder_drop` (bool): when dropping a single folder, inserts `cd '<dir>'` and presses Enter.
  - `auto_run_on_folder_drop` (bool): after `cd`, simulates one extra Enter to run the new prompt.

## Environment Variables

- __ATS_DEBUG_OVERLAY__ (default: false)
  - Enables debug overlay (pane bounds, splitter handles, seam guides, focus logs) and also prints window resize logs.
  - Example: `ATS_DEBUG_OVERLAY=1 cargo run --release`

- __ATS_WINDOW_TRACE__ (default: false)
  - Enables window resize tracing without the overlay. Logs inner size in points/pixels and suggested `[app]` values.
  - Example: `ATS_WINDOW_TRACE=1 cargo run --release`

- __ATS_CONFIG_DIR__ (optional)
  - Overrides the directory where `config.toml` is read/written. Useful for testing alternative configs.
  - Example: `ATS_CONFIG_DIR=/tmp/ats-config cargo run --release`

See `SETING_DEFAULT_SIZE.md` for a step-by-step workflow to pick and persist your preferred default window size and panel split fractions from the debug logs.

## Feature Flags

- __ATS_BTN_ROW_PREPASS__ (default: true)
  - Enables the row-background prepass for the buttons grid. Set to `0`/`false` to fall back to legacy per-cell backgrounds.
  - Example: `ATS_BTN_ROW_PREPASS=0 cargo run --release`

- __ATS_DEBUG_OVERLAY__ (default: false)
  - Enables debug overlay (pane bounds, splitter handles, seam guides, focus logs).
  - Example: `ATS_DEBUG_OVERLAY=1 cargo run --release`

This technical documentation provides the foundation for understanding, maintaining, and extending the Audio Toolkit Shell application.
