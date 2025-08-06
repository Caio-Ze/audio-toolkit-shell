//! # Configuration Module
//! 
//! This module handles application configuration including loading from TOML files,
//! providing default configurations, and defining configuration data structures.

use serde::{Deserialize, Serialize};
use std::fs;

/// Main application configuration structure
/// 
/// Contains all configuration settings for the Audio Toolkit Shell including
/// application settings and tab configurations.
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub tabs: Vec<TabConfig>,
}

/// Application-level settings
/// 
/// Contains global application settings such as window dimensions and application name.
#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    pub name: String,
    pub window_width: f32,
    pub window_height: f32,
}

/// Configuration for individual terminal tabs
/// 
/// Each tab can have its own command, title, and behavior settings.
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
        ],
    }
}