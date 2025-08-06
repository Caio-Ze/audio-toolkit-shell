//! # Audio Toolkit Shell
//! 
//! A terminal emulator application built with Rust and egui, featuring:
//! - Dual-pane terminal interface with tab switching
//! - ANSI color support with Catppuccin Frappé theme
//! - PTY-based terminal emulation
//! - Configurable terminal tabs via TOML configuration
//! 
//! ## Architecture
//! 
//! The application is structured into focused modules:
//! - `main.rs` - Application entry point and initialization
//! - `app.rs` - Main application logic and UI rendering
//! - `terminal.rs` - Terminal emulation and ANSI processing
//! - `theme.rs` - Catppuccin color theme definitions
//! - `config.rs` - Configuration loading and management
//! 
//! ## Usage
//! 
//! The application loads configuration from `config.toml` if present,
//! otherwise uses sensible defaults with two bash terminal tabs.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod app;
mod config;
mod terminal;
mod theme;

use app::AudioToolkitApp;
use config::load_config;

/// Application entry point
/// 
/// Initializes the application by:
/// 1. Loading configuration from `config.toml` or using defaults
/// 2. Setting up the egui native window with configured dimensions
/// 3. Creating and running the main application instance
/// 
/// # Returns
/// 
/// Returns `Ok(())` on successful execution, or an `eframe::Error` if the
/// application fails to initialize or run.
fn main() -> Result<(), eframe::Error> {
    let config = load_config();
    let app_name = config.app.name.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.app.window_width, config.app.window_height]),
        ..Default::default()
    };
    eframe::run_native(
        &app_name,
        options,
        Box::new(move |_cc| Box::new(AudioToolkitApp::new(config))),
    )
}