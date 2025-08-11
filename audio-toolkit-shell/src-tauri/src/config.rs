//! # Configuration Module
//! 
//! This module handles application configuration including loading from TOML files,
//! providing default configurations, and defining configuration data structures.
//! 
//! ## Configuration File Format
//! 
//! The application uses a `config.toml` file located next to the executable (in the
//! same directory as the binary). On first run, if the file does not exist, a
//! ready-to-edit template will be created there automatically. You may also
//! override the directory for development/tests by setting `ATS_CONFIG_DIR` to a
//! folder path before launching the app.
//!
//! The configuration uses the following structure:
//! 
//! ```toml
//! [app]
//! name = "Audio Toolkit Shell"
//! window_width = 1280.0
//! window_height = 720.0
//! 
//! [[tabs]]
//! title = "Terminal 1"
//! command = "bash"
//! auto_restart_on_success = false
//! success_patterns = []
//! [tabs.dnd]
//! # On single folder drop, send: cd '<dir>' and press Enter
//! auto_cd_on_folder_drop = false
//! # If auto_cd_on_folder_drop is false, send: '<dir>' and press Enter
//! auto_run_on_folder_drop = false
//! 
//! [[tabs]]
//! title = "Terminal 2"
//! command = "/path/to/script"
//! auto_restart_on_success = true
//! success_patterns = ["Success", "Completed"]
//! [tabs.dnd]
//! auto_cd_on_folder_drop = true
//! auto_run_on_folder_drop = false
//! ```
//! 
//! ## Fallback Behavior
//! 
//! If the configuration file is missing or invalid, the application will use
//! sensible defaults with four bash terminal tabs arranged in a fixed layout
//! (one left, two right-top, one right-bottom).

use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

/// Main application configuration structure
/// 
/// Contains all configuration settings for the Audio Toolkit Shell including
/// application settings and tab configurations. This is the root configuration
/// object that gets deserialized from the TOML configuration file.
/// 
/// # Fields
/// 
/// * `app` - Global application settings (window size, name, etc.)
/// * `tabs` - Vector of terminal tab configurations
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub tabs: Vec<TabConfig>,
}

/// Application-level settings
/// 
/// Contains global application settings such as window dimensions and application name.
/// These settings affect the overall application behavior and appearance.
/// 
/// # Fields
/// 
/// * `name` - The application window title
/// * `window_width` - Initial window width in pixels
/// * `window_height` - Initial window height in pixels
#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub name: String,
    pub window_width: f32,
    pub window_height: f32,
    /// Minimum width (in px) for the left terminal panel. Defaults to 120.0
    #[serde(default = "default_min_left_width")]
    pub min_left_width: f32,
    /// Minimum width (in px) for the right (central) terminal panel. Defaults to 120.0
    #[serde(default = "default_min_right_width")]
    pub min_right_width: f32,
    /// If true, panels may collapse to zero width. Defaults to false for stability.
    #[serde(default)]
    pub allow_zero_collapse: bool,
    /// Fraction of the right area height taken by the top row (two terminals). Defaults to 0.6
    #[serde(default = "default_right_top_fraction")]
    pub right_top_fraction: f32,
    /// Fraction of the top row width given to the left terminal (of the two top). Defaults to 0.5
    #[serde(default = "default_right_top_hsplit_fraction")]
    pub right_top_hsplit_fraction: f32,
}

/// Drag-and-drop behavior settings
///
/// These settings control optional actions when dropping a single folder.
/// Both keys default to false when omitted in the TOML file.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct DndSettings {
    /// If true, dropping a single directory will insert `cd '<dir>'` and simulate Enter
    #[serde(default)]
    pub auto_cd_on_folder_drop: bool,
    /// If true (and auto_cd_on_folder_drop is false), insert `'<dir>'` and simulate Enter
    #[serde(default)]
    pub auto_run_on_folder_drop: bool,
}

/// Resolve the path to the configuration file.
///
/// Order of precedence:
/// 1. If `ATS_CONFIG_DIR` env var is set, use that directory.
/// 2. Otherwise, use the directory containing the current executable.
fn config_file_path() -> PathBuf {
    if let Ok(dir_override) = env::var("ATS_CONFIG_DIR") {
        return PathBuf::from(dir_override).join("config.toml");
    }
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
    dir.join("config.toml")
}

/// Default first-run configuration template written when no config exists.
///
/// The template enables only "Terminal 1" by default and includes numbered
/// examples for Tabs 2–4, commented out. Users can uncomment exactly one
/// additional `[[tabs]]` block (or keep Tab 1) and edit the command as needed.
const DEFAULT_CONFIG_TEMPLATE: &str = r#"# Audio Toolkit Shell Configuration
# First-run template
#
# This file lives next to the application binary. Edit it in place.
#
# Layout map (fixed arrangement):
#   Terminal 1: Left column (large), buttons panel below
#   Terminal 2: Right top-left
#   Terminal 3: Right top-right
#   Terminal 4: Right bottom
#
# Usage:
# - By default, only Terminal 1 is active.
# - Uncomment Terminal 2–4 sections as needed to enable more panels.
# - Keep the order to match the on-screen layout.
# - Each section below is clearly separated and numbered.

[app]
name = "Audio Toolkit Shell"
window_width = 1458.0
window_height = 713.0

# ===================== Terminal 1 (Left column) =====================
[[tabs]]
title = "Terminal 1"
command = "bash"                      # Change to your tool or script path
auto_restart_on_success = false
success_patterns = []
[tabs.dnd]
auto_cd_on_folder_drop = false
auto_run_on_folder_drop = false

# ===================== Terminal 2 (Right top-left) ==================
# [[tabs]]
# title = "Terminal 2"
# command = "bash"
# auto_restart_on_success = false
# success_patterns = []
# [tabs.dnd]
# auto_cd_on_folder_drop = false
# auto_run_on_folder_drop = false

# ===================== Terminal 3 (Right top-right) =================
# [[tabs]]
# title = "Terminal 3"
# command = "bash"
# auto_restart_on_success = false
# success_patterns = []
# [tabs.dnd]
# auto_cd_on_folder_drop = false
# auto_run_on_folder_drop = false

# ===================== Terminal 4 (Right bottom) ====================
# [[tabs]]
# title = "Terminal 4"
# command = "bash"
# auto_restart_on_success = false
# success_patterns = []
# [tabs.dnd]
# auto_cd_on_folder_drop = false
# auto_run_on_folder_drop = false
"#;

fn default_min_left_width() -> f32 {
    120.0
}

fn default_min_right_width() -> f32 {
    120.0
}

fn default_right_top_fraction() -> f32 {
    0.6
}

fn default_right_top_hsplit_fraction() -> f32 {
    0.5
}

/// Configuration for individual terminal tabs
/// 
/// Each tab can have its own command, title, and behavior settings.
/// Tabs support auto-restart functionality based on pattern matching.
/// 
/// # Fields
/// 
/// * `title` - Display name for the tab
/// * `command` - Command to execute (absolute path or shell command)
/// * `auto_restart_on_success` - Whether to restart when success patterns are detected
/// * `success_patterns` - Text patterns that indicate successful completion
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TabConfig {
    pub title: String,
    pub command: String,
    pub auto_restart_on_success: bool,
    pub success_patterns: Vec<String>,
    /// Per-tab drag-and-drop behavior settings
    #[serde(default)]
    pub dnd: DndSettings,
}

/// Loads configuration from config.toml file
/// 
/// Attempts to read and parse `config.toml` located next to the executable. If the
/// file doesn't exist, a first-run template is created there and then loaded.
/// If parsing fails, the application falls back to the in-memory default configuration.
/// 
/// # Returns
/// 
/// An `AppConfig` instance either loaded from file or using defaults
pub fn load_config() -> AppConfig {
    let config_path = config_file_path();
    match fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Error parsing {:?}: {}", config_path, e);
            default_config()
        }),
        Err(err) => {
            // Create first-run template only if the file is missing
            eprintln!("Config not found at {:?} ({}). Creating template...", config_path, err);
            if let Err(write_err) = fs::write(&config_path, DEFAULT_CONFIG_TEMPLATE) {
                eprintln!("Failed to create config at {:?}: {}", config_path, write_err);
                return default_config();
            }
            match fs::read_to_string(&config_path) {
                Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                    eprintln!("Error parsing freshly written config {:?}: {}", config_path, e);
                    default_config()
                }),
                Err(read_err) => {
                    eprintln!("Failed to read freshly written config {:?}: {}", config_path, read_err);
                    default_config()
                }
            }
        }
    }
}

/// Provides default application configuration
/// 
/// Creates a default configuration with standard settings and two bash terminal tabs.
/// This is used when no configuration file exists or when the configuration file
/// cannot be parsed.
/// 
/// # Returns
/// 
/// An `AppConfig` instance with default settings
pub fn default_config() -> AppConfig {
    AppConfig {
        app: AppSettings {
            name: "Audio Toolkit Shell".to_string(),
            window_width: 1458.0,
            window_height: 713.0,
            min_left_width: 120.0,
            min_right_width: 120.0,
            allow_zero_collapse: false,
            right_top_fraction: 0.617,
            right_top_hsplit_fraction: 0.5,
        },
        tabs: vec![
            TabConfig {
                title: "Terminal 1".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
                dnd: DndSettings::default(),
            },
            TabConfig {
                title: "Terminal 2".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
                dnd: DndSettings::default(),
            },
            TabConfig {
                title: "Terminal 3".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
                dnd: DndSettings::default(),
            },
            TabConfig {
                title: "Terminal 4".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
                dnd: DndSettings::default(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;


    #[test]
    fn test_default_config() {
        let config = default_config();
        
        assert_eq!(config.app.name, "Audio Toolkit Shell");
        assert_eq!(config.app.window_width, 1458.0);
        assert_eq!(config.app.window_height, 713.0);
        assert_eq!(config.tabs.len(), 4);
        assert_eq!(config.tabs[0].title, "Terminal 1");
        assert_eq!(config.tabs[1].title, "Terminal 2");
        assert_eq!(config.tabs[2].title, "Terminal 3");
        assert_eq!(config.tabs[3].title, "Terminal 4");
        for tab in &config.tabs {
            assert_eq!(tab.command, "bash");
            assert!(!tab.auto_restart_on_success);
            assert!(tab.success_patterns.is_empty());
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = default_config();
        let toml_string = toml::to_string(&config).expect("Failed to serialize config");
        
        // Should contain expected sections
        assert!(toml_string.contains("[app]"));
        assert!(toml_string.contains("[[tabs]]"));
        assert!(toml_string.contains("name = \"Audio Toolkit Shell\""));
        assert!(toml_string.contains("command = \"bash\""));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_content = r#"
[app]
name = "Test App"
window_width = 800.0
window_height = 600.0

[[tabs]]
title = "Test Tab"
command = "echo"
auto_restart_on_success = true
success_patterns = ["done", "complete"]
"#;

        let config: AppConfig = toml::from_str(toml_content).expect("Failed to parse TOML");
        
        assert_eq!(config.app.name, "Test App");
        assert_eq!(config.app.window_width, 800.0);
        assert_eq!(config.app.window_height, 600.0);
        assert_eq!(config.tabs.len(), 1);
        assert_eq!(config.tabs[0].title, "Test Tab");
        assert_eq!(config.tabs[0].command, "echo");
        assert!(config.tabs[0].auto_restart_on_success);
        assert_eq!(config.tabs[0].success_patterns, vec!["done", "complete"]);
    }

    #[test]
    fn test_load_config_with_missing_file() {
        // Use a temporary directory via ATS_CONFIG_DIR so we don't touch the real binary dir
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let original = std::env::var("ATS_CONFIG_DIR").ok();
        std::env::set_var("ATS_CONFIG_DIR", temp_dir.path());

        // No config exists yet: load_config should create a template and then load it
        let config = load_config();

        assert_eq!(config.app.name, "Audio Toolkit Shell");
        // Template enables only one tab by default (others are commented)
        assert_eq!(config.tabs.len(), 1);

        // Cleanup env var
        if let Some(val) = original { std::env::set_var("ATS_CONFIG_DIR", val); } else { std::env::remove_var("ATS_CONFIG_DIR"); }
    }

    #[test]
    fn test_config_parsing_directly() {
        // Test TOML parsing directly without file system operations
        let config_content = r#"
[app]
name = "Custom Shell"
window_width = 1920.0
window_height = 1080.0

[[tabs]]
title = "Custom Tab"
command = "zsh"
auto_restart_on_success = false
success_patterns = []
"#;
        
        let config: AppConfig = toml::from_str(config_content).expect("Failed to parse TOML");
        
        assert_eq!(config.app.name, "Custom Shell");
        assert_eq!(config.app.window_width, 1920.0);
        assert_eq!(config.app.window_height, 1080.0);
        assert_eq!(config.tabs.len(), 1);
        assert_eq!(config.tabs[0].title, "Custom Tab");
        assert_eq!(config.tabs[0].command, "zsh");
        assert!(!config.tabs[0].auto_restart_on_success);
        assert!(config.tabs[0].success_patterns.is_empty());
    }

    #[test]
    fn test_load_config_with_invalid_toml() {
        let invalid_content = "invalid toml content [[[";

        // Create a temporary directory and config file, and point ATS_CONFIG_DIR to it
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, invalid_content).expect("Failed to write config");

        let original = std::env::var("ATS_CONFIG_DIR").ok();
        std::env::set_var("ATS_CONFIG_DIR", temp_dir.path());

        let config = load_config();

        // Should fall back to default config on parse error
        assert_eq!(config.app.name, "Audio Toolkit Shell");
        assert_eq!(config.tabs.len(), 4);

        // Cleanup env var
        if let Some(val) = original { std::env::set_var("ATS_CONFIG_DIR", val); } else { std::env::remove_var("ATS_CONFIG_DIR"); }
    }

    #[test]
    fn test_tab_config_clone() {
        let tab = TabConfig {
            title: "Test".to_string(),
            command: "echo".to_string(),
            auto_restart_on_success: true,
            success_patterns: vec!["done".to_string()],
            dnd: DndSettings::default(),
        };
        
        let cloned = tab.clone();
        assert_eq!(tab.title, cloned.title);
        assert_eq!(tab.command, cloned.command);
        assert_eq!(tab.auto_restart_on_success, cloned.auto_restart_on_success);
        assert_eq!(tab.success_patterns, cloned.success_patterns);
        assert_eq!(tab.dnd, cloned.dnd);
    }
}