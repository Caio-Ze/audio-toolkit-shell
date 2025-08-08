//! # Configuration Module
//! 
//! This module handles application configuration including loading from TOML files,
//! providing default configurations, and defining configuration data structures.
//! 
//! ## Configuration File Format
//! 
//! The application expects a `config.toml` file in the working directory with the following structure:
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
//! 
//! [[tabs]]
//! title = "Terminal 2"
//! command = "/path/to/script"
//! auto_restart_on_success = true
//! success_patterns = ["Success", "Completed"]
//! ```
//! 
//! ## Fallback Behavior
//! 
//! If the configuration file is missing or invalid, the application will use
//! sensible defaults with four bash terminal tabs arranged in a fixed layout
//! (one left, two right-top, one right-bottom).

use serde::{Deserialize, Serialize};
use std::fs;

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
}

/// Loads configuration from config.toml file
/// 
/// Attempts to read and parse the configuration file. If the file doesn't exist
/// or cannot be parsed, falls back to the default configuration.
/// 
/// # Returns
/// 
/// An `AppConfig` instance either loaded from file or using defaults
pub fn load_config() -> AppConfig {
    let config_path = "config.toml";
    match fs::read_to_string(config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Error parsing config.toml: {}", e);
            default_config()
        }),
        Err(_) => {
            eprintln!("config.toml not found, using default configuration");
            default_config()
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
            window_width: 1280.0,
            window_height: 720.0,
            min_left_width: 120.0,
            min_right_width: 120.0,
            allow_zero_collapse: false,
            right_top_fraction: 0.6,
            right_top_hsplit_fraction: 0.5,
        },
        tabs: vec![
            TabConfig {
                title: "Terminal 1".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
            },
            TabConfig {
                title: "Terminal 2".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
            },
            TabConfig {
                title: "Terminal 3".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
            },
            TabConfig {
                title: "Terminal 4".to_string(),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
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
        assert_eq!(config.app.window_width, 1280.0);
        assert_eq!(config.app.window_height, 720.0);
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
        // Change to a temporary directory to avoid interfering with actual config
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        
        std::env::set_current_dir(&temp_dir).expect("Failed to change dir");
        
        let config = load_config();
        
        // Should fall back to default config
        assert_eq!(config.app.name, "Audio Toolkit Shell");
        assert_eq!(config.tabs.len(), 4);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");
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
        
        // Create a temporary directory and config file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, invalid_content).expect("Failed to write config");
        
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
        
        let config = load_config();
        
        // Should fall back to default config on parse error
        assert_eq!(config.app.name, "Audio Toolkit Shell");
        assert_eq!(config.tabs.len(), 4);
        
        // Cleanup
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    }

    #[test]
    fn test_tab_config_clone() {
        let tab = TabConfig {
            title: "Test".to_string(),
            command: "echo".to_string(),
            auto_restart_on_success: true,
            success_patterns: vec!["done".to_string()],
        };
        
        let cloned = tab.clone();
        assert_eq!(tab.title, cloned.title);
        assert_eq!(tab.command, cloned.command);
        assert_eq!(tab.auto_restart_on_success, cloned.auto_restart_on_success);
        assert_eq!(tab.success_patterns, cloned.success_patterns);
    }
}