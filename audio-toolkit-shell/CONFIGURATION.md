# Configuration Guide

## Overview

Audio Toolkit Shell uses a TOML configuration file (`config.toml`) to define the application behavior, window settings, and terminal tabs. This guide explains how to configure the application for your specific workflow.

## Configuration File Location

The configuration file must be located at:
```
src-tauri/config.toml
```

This file defines the application behavior, window settings, and terminal tab configurations.

## Basic Structure

```toml
[app]
name = "Your App Name"
window_width = 1280
window_height = 720

[[tabs]]
title = "Tab Name"
command = "/path/to/executable"
auto_restart_on_success = true
success_patterns = ["pattern1", "pattern2"]
```

## Application Settings

### `[app]` Section

| Setting | Type | Description | Default |
|---------|------|-------------|---------|
| `name` | String | Window title displayed in title bar | "Audio Toolkit Shell" |
| `window_width` | Number | Initial window width in pixels | 1280 |
| `window_height` | Number | Initial window height in pixels | 720 |

**Example:**
```toml
[app]
name = "My Audio Tools"
window_width = 1920
window_height = 1080
```

## Tab Configuration

### `[[tabs]]` Sections

Each `[[tabs]]` section defines one terminal tab. You can have as many tabs as needed.

| Setting | Type | Description | Required |
|---------|------|-------------|----------|
| `title` | String | Tab display name | Yes |
| `command` | String | Command or executable path | Yes |
| `auto_restart_on_success` | Boolean | Whether to restart on success patterns | Yes |
| `success_patterns` | Array of Strings | Text patterns that trigger restart | Yes |

### Command Types

#### 1. Absolute Path Executables
For custom executables with full paths:
```toml
[[tabs]]
title = "My Script"
command = "/Users/username/scripts/my_script"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "Done"]
```

#### 2. System Commands
For standard shell commands:
```toml
[[tabs]]
title = "System Monitor"
command = "htop"
auto_restart_on_success = false
success_patterns = []
```

#### 3. Shell Scripts
For shell scripts:
```toml
[[tabs]]
title = "Build Script"
command = "/path/to/build.sh"
auto_restart_on_success = true
success_patterns = ["Build complete", "SUCCESS"]
```

#### 4. Standard Shell
For a regular bash terminal:
```toml
[[tabs]]
title = "Terminal"
command = "bash"
auto_restart_on_success = false
success_patterns = []
```

### Auto-Restart Configuration

#### When to Use Auto-Restart
- **Enable** (`true`) for: Menu-driven tools, batch processors, workflow scripts
- **Disable** (`false`) for: Interactive shells, long-running processes, development tools

#### Success Patterns
Patterns that trigger auto-restart when found in terminal output:

```toml
success_patterns = [
    "Completed successfully",
    "Process finished",
    "MAIN MENU",
    "Ready for next command"
]
```

**Pattern Matching Rules:**
- Case-sensitive exact substring matching
- Patterns are checked against cleaned output (ANSI codes removed)
- First matching pattern triggers restart
- Empty array `[]` disables pattern detection

## Example Configurations

### Audio Processing Workflow
```toml
[app]
name = "Audio Toolkit Shell"
window_width = 1280
window_height = 720

[[tabs]]
title = "Start Scripts"
command = "/Users/username/audio-tools/start_scripts_rust"
auto_restart_on_success = true
success_patterns = ["Completed successfully", "SCRIPT MENU"]

[[tabs]]
title = "Audio Normalizer"
command = "/Users/username/audio-tools/audio_normalizer"
auto_restart_on_success = true
success_patterns = ["Normalization complete", "Ready"]

[[tabs]]
title = "Session Monitor"
command = "/Users/username/audio-tools/session_monitor"
auto_restart_on_success = false
success_patterns = []

[[tabs]]
title = "Terminal"
command = "bash"
auto_restart_on_success = false
success_patterns = []
```

### Development Workflow
```toml
[app]
name = "Dev Environment"
window_width = 1600
window_height = 900

[[tabs]]
title = "Build"
command = "/project/scripts/build.sh"
auto_restart_on_success = true
success_patterns = ["Build successful", "✓ Done"]

[[tabs]]
title = "Test Runner"
command = "/project/scripts/test.sh"
auto_restart_on_success = true
success_patterns = ["All tests passed", "✓ Tests complete"]

[[tabs]]
title = "Dev Server"
command = "/project/scripts/dev-server.sh"
auto_restart_on_success = false
success_patterns = []

[[tabs]]
title = "Shell"
command = "bash"
auto_restart_on_success = false
success_patterns = []
```

### System Administration
```toml
[app]
name = "System Tools"
window_width = 1440
window_height = 800

[[tabs]]
title = "System Monitor"
command = "htop"
auto_restart_on_success = false
success_patterns = []

[[tabs]]
title = "Log Viewer"
command = "tail -f /var/log/system.log"
auto_restart_on_success = false
success_patterns = []

[[tabs]]
title = "Network Tools"
command = "/usr/local/bin/network-diagnostics"
auto_restart_on_success = true
success_patterns = ["Diagnostics complete"]

[[tabs]]
title = "Admin Shell"
command = "sudo -i"
auto_restart_on_success = false
success_patterns = []
```

## Best Practices

### 1. Executable Paths
- **Always use absolute paths** for custom executables
- **Verify permissions** - executables must be executable (`chmod +x`)
- **Test manually first** - ensure your executable works in a regular terminal

### 2. Success Patterns
- **Be specific** - avoid common words that might appear in normal output
- **Test patterns** - run your executable manually and note the exact completion messages
- **Use multiple patterns** - different completion scenarios might have different messages

### 3. Tab Organization
- **Logical grouping** - related tools in adjacent tabs
- **Descriptive names** - clear, concise tab titles
- **Consistent ordering** - most-used tabs first

### 4. Window Sizing
- **Consider your screen** - don't exceed your display resolution
- **Account for content** - larger windows for complex terminal output
- **Test different sizes** - ensure usability at your chosen dimensions

## Troubleshooting

### Common Issues

#### 1. Tab Shows No Output
**Problem**: Terminal tab is empty or shows no content
**Solutions**:
- Verify executable path is correct and absolute
- Check file permissions (`ls -la /path/to/executable`)
- Test executable manually in terminal
- Check for typos in `command` field

#### 2. Auto-Restart Not Working
**Problem**: Tab doesn't restart after completion
**Solutions**:
- Verify `auto_restart_on_success = true`
- Check success patterns match actual output
- Test patterns are case-sensitive and exact
- Ensure patterns appear in cleaned output (no ANSI codes)

#### 3. Configuration Not Loading
**Problem**: Changes to config.toml not reflected in app
**Solutions**:
- Restart the application completely
- Check TOML syntax with a validator
- Verify file is saved in correct location (`src-tauri/config.toml`)
- Check for syntax errors in TOML format

#### 4. Window Size Issues
**Problem**: Window too large/small or doesn't fit screen
**Solutions**:
- Adjust `window_width` and `window_height` values
- Consider your display resolution
- Test with different values
- Use reasonable defaults (1280x720 works for most screens)

### Validation

To validate your configuration:

1. **TOML Syntax**: Use an online TOML validator
2. **File Paths**: Test executables manually in terminal
3. **Patterns**: Run executables and note exact completion messages
4. **Permissions**: Ensure all executables have execute permissions

### Example Validation Commands
```bash
# Check executable exists and is executable
ls -la /path/to/your/executable

# Test executable manually
/path/to/your/executable

# Validate TOML syntax
# (Use online validator or TOML parser)
```

## Advanced Configuration

### Custom Success Patterns
For complex workflows, you might need sophisticated patterns:

```toml
success_patterns = [
    "Process completed with exit code 0",
    "All files processed successfully",
    "Ready for next batch",
    "Return to main menu"
]
```

### Multiple Executable Versions
For development vs production executables:

```toml
# Development version
[[tabs]]
title = "Tool [DEV]"
command = "/dev/tools/my_tool_dev"
auto_restart_on_success = true
success_patterns = ["DEV: Complete"]

# Production version
[[tabs]]
title = "Tool [PROD]"
command = "/prod/tools/my_tool"
auto_restart_on_success = true
success_patterns = ["PROD: Complete"]
```

This configuration guide should help you customize Audio Toolkit Shell for your specific workflow needs.
