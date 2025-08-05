# Technical Documentation

## Architecture Overview

Audio Toolkit Shell is built as a native Rust application using the eframe/egui ecosystem for GUI and portable-pty for terminal integration. The application features a character-by-character terminal emulator with real PTY backing for authentic terminal behavior.

## Core Components

### 1. Application Structure

```rust
struct AudioToolkitApp {
    active_tab: usize,
    tabs: Vec<TerminalTab>,
    config: AppConfig,
}

struct TerminalTab {
    title: String,
    config: TabConfig,
    pty_master: Box<dyn portable_pty::MasterPty + Send>,
    pty_writer: Option<Box<dyn std::io::Write + Send>>,
    output_rx: Receiver<String>,
    terminal_emulator: TerminalEmulator,
    input: String,
    needs_restart: bool,
}

struct TerminalEmulator {
    buffer: Vec<Vec<TerminalCell>>,
    cursor_row: usize,
    cursor_col: usize,
    rows: usize,
    cols: usize,
    current_color: egui::Color32,
    bold: bool,
}
```

### 2. Configuration System

The application uses TOML for configuration with the following structure:

```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1280
window_height = 720

[[tabs]]
title = "Start Scripts"
command = "/path/to/executable"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]
```

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

This technical documentation provides the foundation for understanding, maintaining, and extending the Audio Toolkit Shell application.
