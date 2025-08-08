//! # Application Module
//! 
//! This module contains the main application logic including terminal tab management,
//! UI rendering, and application state management.
//! 
//! ## Architecture
//! 
//! The application uses a dual-pane layout with two terminal tabs displayed side by side.
//! Each tab runs its own PTY (pseudo-terminal) and can execute different commands.
//! 
//! ## Key Components
//! 
//! - **TerminalTab**: Manages individual terminal instances with PTY communication
//! - **AudioToolkitApp**: Main application state and UI rendering logic
//! 
//! ## Features
//! 
//! - Split-screen terminal interface
//! - Tab focus switching with Tab key
//! - Auto-restart functionality based on pattern matching
//! - Full keyboard input support including arrow keys
//! - ANSI color rendering with Catppuccin theme

use eframe::{egui, App, Frame};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

use crate::config::{AppConfig, AppSettings, TabConfig};
use crate::terminal::{TerminalCell, TerminalEmulator};
use crate::theme::CatppuccinTheme;

/// Represents a single terminal tab with its own PTY and state
/// 
/// Each tab manages its own pseudo-terminal, command execution, and terminal emulator.
/// Tabs can be configured to auto-restart based on success patterns.
/// 
/// The tab handles:
/// - PTY creation and management
/// - Command execution with proper environment setup
/// - Terminal output processing through the emulator
/// - Pattern-based auto-restart functionality
/// - Input handling and forwarding to the PTY
pub struct TerminalTab {
    title: String,
    config: TabConfig,
    pty_master: Box<dyn portable_pty::MasterPty + Send>,
    pty_writer: Option<Box<dyn std::io::Write + Send>>,
    output_rx: Receiver<String>,
    output: String,
    terminal_emulator: TerminalEmulator,
    input: String,
    needs_restart: bool,
    startup_time: std::time::Instant,
    pattern_matches: u32,
}

impl TerminalTab {
    /// Creates a new terminal tab with the given configuration
    /// 
    /// Sets up a PTY, spawns the configured command, and initializes the terminal emulator.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The tab configuration including command and title
    pub fn new(config: TabConfig) -> Self {
        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                ..Default::default()
            })
            .unwrap();

        let cmd = if config.command.contains('/') {
            // Absolute path - run directly to preserve TTY environment
            println!("Executing command: {}", &config.command);
            let mut cmd = CommandBuilder::new(&config.command);
            // Set comprehensive environment variables for proper terminal behavior
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("SHELL", "/bin/zsh");
            cmd.env("COLUMNS", "80");
            cmd.env("LINES", "24");
            // Force color output
            cmd.env("FORCE_COLOR", "1");
            cmd.env("CLICOLOR", "1");
            cmd.env("CLICOLOR_FORCE", "1");
            cmd.env("NO_COLOR", ""); // Ensure NO_COLOR is not set
                                     // Terminal capabilities
            cmd.env("TERM_PROGRAM", "AudioToolkitShell");
            cmd.env("TERM_PROGRAM_VERSION", "1.0");
            // Force interactive/TTY mode
            cmd.env("PS1", "$ ");
            cmd.env("INTERACTIVE", "1");
            cmd.env("ISATTY", "1");
            // Rust/CLI specific
            cmd.env("RUST_LOG_STYLE", "always");
            cmd.env("CLI_COLOR", "1");
            cmd
        } else {
            // Command name - run in shell
            println!("Executing bash shell");
            let mut cmd = CommandBuilder::new("bash");
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("SHELL", "/bin/zsh");
            cmd.env("COLUMNS", "80");
            cmd.env("LINES", "24");
            cmd.env("FORCE_COLOR", "1");
            cmd.env("CLICOLOR", "1");
            cmd.env("CLICOLOR_FORCE", "1");
            cmd.env("PS1", "$ ");
            cmd.env("INTERACTIVE", "1");
            cmd
        };

        match pty_pair.slave.spawn_command(cmd) {
            Ok(_child) => println!("Command spawned successfully for tab: {}", config.title),
            Err(e) => {
                eprintln!("Failed to spawn command for tab {}: {}", config.title, e);
                // Fall back to bash if the command fails
                let fallback_cmd = CommandBuilder::new("bash");
                let _fallback_child = pty_pair.slave.spawn_command(fallback_cmd).unwrap();
            }
        }

        let mut reader = pty_pair.master.try_clone_reader().unwrap();

        let (output_tx, output_rx) = channel();

        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(len) if len > 0 => {
                        if let Ok(str) = std::str::from_utf8(&buf[..len]) {
                            if output_tx.send(str.to_string()).is_err() {
                                break;
                            }
                        }
                    }
                    _ => break,
                }
            }
        });

        // Get the writer once and store it
        let writer = pty_pair.master.take_writer().ok();

        Self {
            title: config.title.clone(),
            config,
            pty_master: pty_pair.master,
            pty_writer: writer,
            output_rx,
            output: String::new(),
            terminal_emulator: TerminalEmulator::new(24, 80),
            input: String::new(),
            needs_restart: false,
            startup_time: std::time::Instant::now(),
            pattern_matches: 0,
        }
    }

    /// Updates the terminal output by reading from the PTY
    /// 
    /// Processes new data through the terminal emulator and checks for success patterns
    /// if auto-restart is enabled.
    pub fn update_output(&mut self) {
        loop {
            match self.output_rx.try_recv() {
                Ok(data) => {
                    // Process data through terminal emulator
                    self.terminal_emulator.process_ansi_data(&data);

                    // Strip ANSI codes for pattern matching
                    let plain_text = Self::strip_ansi_codes(&data);
                    self.output.push_str(&plain_text);

                    // Check for success patterns if auto-restart is enabled
                    if self.config.auto_restart_on_success {
                        // Only check patterns after 5 seconds to avoid startup menu detection
                        let elapsed = self.startup_time.elapsed();
                        if elapsed.as_secs() >= 5 {
                            for pattern in &self.config.success_patterns {
                                if plain_text.contains(pattern) {
                                    self.pattern_matches += 1;
                                    println!(
                                        "[PATTERN] Found '{}' in tab '{}' (match #{}/2)",
                                        pattern, self.title, self.pattern_matches
                                    );

                                    // Require 2 pattern matches to avoid false positives
                                    if self.pattern_matches >= 2 {
                                        println!(
                                            "[PATTERN] Triggering restart for tab '{}'",
                                            self.title
                                        );
                                        self.needs_restart = true;
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Still in startup period - don't check patterns yet
                            if !self.config.success_patterns.is_empty() {
                                println!(
                                    "[PATTERN] Startup period for '{}' - {} seconds remaining",
                                    self.title,
                                    5 - elapsed.as_secs()
                                );
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Restarts the terminal tab if needed
    /// 
    /// Creates a new PTY, spawns the command again, and resets the terminal state.
    pub fn restart(&mut self) {
        if !self.needs_restart {
            return;
        }

        println!("[RESTART] Restarting tab: {}", self.title);

        // Clear output and reset restart flag
        self.output.clear();
        self.terminal_emulator.clear_screen();
        self.needs_restart = false;
        self.startup_time = std::time::Instant::now();
        self.pattern_matches = 0;

        // Add a small delay to allow previous PTY resources to clean up
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Create new PTY with better error handling
        let pty_system = NativePtySystem::default();
        let pty_pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            ..Default::default()
        }) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("[RESTART] Failed to create PTY for {}: {}", self.title, e);
                // Add restart message to output so user knows what happened
                self.output
                    .push_str(&format!("\n[ERROR] Failed to restart: {}\n", e));
                return;
            }
        };

        let cmd = if self.config.command.contains('/') {
            // Absolute path - run directly to preserve TTY environment
            let mut cmd = CommandBuilder::new(&self.config.command);
            // Set comprehensive environment variables for proper terminal behavior
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("SHELL", "/bin/zsh");
            cmd.env("COLUMNS", "80");
            cmd.env("LINES", "24");
            // Force color output
            cmd.env("FORCE_COLOR", "1");
            cmd.env("CLICOLOR", "1");
            cmd.env("CLICOLOR_FORCE", "1");
            cmd.env("NO_COLOR", ""); // Ensure NO_COLOR is not set
                                     // Terminal capabilities
            cmd.env("TERM_PROGRAM", "AudioToolkitShell");
            cmd.env("TERM_PROGRAM_VERSION", "1.0");
            // Force interactive/TTY mode
            cmd.env("PS1", "$ ");
            cmd.env("INTERACTIVE", "1");
            cmd.env("ISATTY", "1");
            // Rust/CLI specific
            cmd.env("RUST_LOG_STYLE", "always");
            cmd.env("CLI_COLOR", "1");
            cmd
        } else {
            // Command name - run in shell
            let mut cmd = CommandBuilder::new("bash");
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("SHELL", "/bin/zsh");
            cmd.env("COLUMNS", "80");
            cmd.env("LINES", "24");
            cmd.env("FORCE_COLOR", "1");
            cmd.env("CLICOLOR", "1");
            cmd.env("CLICOLOR_FORCE", "1");
            // Force interactive mode
            cmd.env("PS1", "$ ");
            cmd.env("INTERACTIVE", "1");
            cmd
        };

        match pty_pair.slave.spawn_command(cmd) {
            Ok(_child) => {
                println!("[RESTART] Command spawned successfully for: {}", self.title);
                // Add success completion message to output
                self.output
                    .push_str("\n✅ Script executed successfully\n\n");
            }
            Err(e) => {
                eprintln!(
                    "[RESTART] Failed to spawn command for {}: {}",
                    self.title, e
                );
                self.output
                    .push_str(&format!("\n[ERROR] Failed to restart command: {}\n", e));
                return;
            }
        }

        // Set up new PTY reader with better error handling
        let mut reader = match pty_pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!(
                    "[RESTART] Failed to clone PTY reader for {}: {}",
                    self.title, e
                );
                self.output
                    .push_str(&format!("\n[ERROR] Failed to setup PTY reader: {}\n", e));
                return;
            }
        };

        let (output_tx, output_rx) = channel();
        let tab_title = self.title.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            println!("[RESTART] PTY reader thread started for: {}", tab_title);

            loop {
                match reader.read(&mut buf) {
                    Ok(len) => {
                        if len > 0 {
                            if let Ok(str) = std::str::from_utf8(&buf[..len]) {
                                if output_tx.send(str.to_string()).is_err() {
                                    println!("[RESTART] Output channel closed for: {}", tab_title);
                                    break;
                                }
                            }
                        } else {
                            // EOF - wait a bit before retrying
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }
                    Err(e) => {
                        eprintln!("[RESTART] PTY read error for {}: {}", tab_title, e);
                        break;
                    }
                }
            }
            println!("[RESTART] PTY reader thread ended for: {}", tab_title);
        });

        // Get new PTY writer with error handling
        let writer = match pty_pair.master.take_writer() {
            Ok(writer) => Some(writer),
            Err(e) => {
                eprintln!(
                    "[RESTART] Failed to get PTY writer for {}: {}",
                    self.title, e
                );
                None
            }
        };

        // Update the tab with new PTY components
        self.pty_master = pty_pair.master;
        self.pty_writer = writer;
        self.output_rx = output_rx;

        println!("[RESTART] Successfully restarted tab: {}", self.title);
    }

    /// Gets the tab title
    /// 
    /// # Returns
    /// 
    /// A string slice containing the tab's display title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Gets a reference to the terminal emulator
    /// 
    /// Provides read-only access to the terminal emulator for rendering
    /// the terminal buffer and accessing terminal state.
    /// 
    /// # Returns
    /// 
    /// A reference to the terminal emulator instance
    pub fn terminal_emulator(&self) -> &TerminalEmulator {
        &self.terminal_emulator
    }

    /// Gets a mutable reference to the input string
    /// 
    /// Used by the UI to allow editing of the current input line.
    /// 
    /// # Returns
    /// 
    /// A mutable reference to the input string
    pub fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    /// Gets the current input string
    /// 
    /// # Returns
    /// 
    /// A string slice containing the current input text
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Clears the input string
    /// 
    /// Typically called after sending input to the PTY to reset
    /// the input field for the next command.
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    /// Strips ANSI escape codes from text for pattern matching
    /// 
    /// # Arguments
    /// 
    /// * `input` - The input string containing ANSI codes
    /// 
    /// # Returns
    /// 
    /// A string with ANSI codes removed
    fn strip_ansi_codes(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                // ESC character
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['

                    // Skip the ANSI sequence
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        if next_ch.is_ascii_alphabetic() || "~".contains(next_ch) {
                            break;
                        }
                    }
                } else {
                    // Not an ANSI sequence, keep the character
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

/// Main application struct managing terminal tabs and UI state
/// 
/// Handles the overall application state, terminal focus management, and UI rendering.
/// 
/// The application implements the `eframe::App` trait to provide the main update loop
/// and rendering logic. It manages multiple terminal tabs in a split-screen layout
/// and handles global keyboard shortcuts for tab switching.
pub struct AudioToolkitApp {
    tabs: Vec<TerminalTab>,
    focused_terminal: usize, // 0 = left terminal, 1 = right terminal
    app_settings: AppSettings,
}

impl AudioToolkitApp {
    /// Creates a new application instance with the given configuration
    /// 
    /// Initializes the application by creating terminal tabs based on the provided
    /// configuration. Each tab gets its own PTY and starts executing its configured command.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The application configuration containing tab settings
    /// 
    /// # Returns
    /// 
    /// A new `AudioToolkitApp` instance ready for use with eframe
    pub fn new(config: AppConfig) -> Self {
        let AppConfig { app, tabs } = config;
        let tabs = tabs.into_iter().map(TerminalTab::new).collect();

        Self {
            tabs,
            focused_terminal: 0, // Start with left terminal focused
            app_settings: app,
        }
    }

    // ... (rest of the code remains the same)
    /// 
    /// * `row` - The row of terminal cells to render
    /// * `ui` - The egui UI context
    fn render_row(row: &[TerminalCell], ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.spacing_mut().item_spacing.y = 0.0;

            // Strict layout mode: render wide glyphs as a fixed two-cell spacer to preserve alignment
            let mut i = 0usize;
            while i < row.len() {
                let cell = &row[i];

                // If this is a wide-glyph lead cell, the next cell will be the placeholder '\0'
                if i + 1 < row.len() && row[i + 1].character == '\0' && cell.character != '\0' {
                    // Render exactly two monospace spaces to preserve grid alignment, skip the placeholder cell
                    let spacer2 = egui::RichText::new("  ")
                        .font(egui::FontId::monospace(12.0));
                    ui.add(egui::Label::new(spacer2).wrap(false).selectable(false));
                    i += 2;
                    continue;
                }

                // Preserve column width for stray placeholder by rendering a single-space spacer
                if cell.character == '\0' {
                    let spacer = egui::RichText::new(" ")
                        .font(egui::FontId::monospace(12.0));
                    ui.add(egui::Label::new(spacer).wrap(false).selectable(false));
                    i += 1;
                    continue;
                }

                // Normal single-width glyph
                let mut rich_text = egui::RichText::new(cell.character.to_string())
                    .font(egui::FontId::monospace(12.0))
                    .color(cell.color);
                
                if cell.bold {
                    rich_text = rich_text.strong();
                }
                
                ui.add(egui::Label::new(rich_text).wrap(false).selectable(false));
                i += 1;
            }
        });
    }

    /// Renders the complete terminal buffer
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The egui UI context
    /// * `buffer` - The terminal buffer to render
    fn render_terminal_buffer(ui: &mut egui::Ui, buffer: &[Vec<TerminalCell>]) {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.spacing_mut().item_spacing.x = 0.0;
        
        for row in buffer {
            Self::render_row(row, ui);
        }
    }

    /// Handles terminal keyboard input including arrow keys and special keys
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The egui UI context
    /// * `pty_writer` - The PTY writer for sending input
    fn handle_terminal_key_input(
        ui: &mut egui::Ui,
        pty_writer: &mut Option<Box<dyn Write + Send>>,
    ) {
        ui.input(|i| {
            // Handle arrow keys for navigation
            if i.key_pressed(egui::Key::ArrowUp) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[A"); // Up arrow ANSI sequence
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[B"); // Down arrow ANSI sequence
                }
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[D"); // Left arrow ANSI sequence
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[C"); // Right arrow ANSI sequence
                }
            }
            // Handle other special keys
            if i.key_pressed(egui::Key::Escape) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b"); // Escape key
                }
            }
            if i.key_pressed(egui::Key::Space) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b" "); // Space key
                }
            }
        });
    }
}

impl App for AudioToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Apply Catppuccin Frappé theme to egui context
        let mut style = (*ctx.style()).clone();
        
        // Set window and panel backgrounds to Catppuccin base color
        style.visuals.window_fill = CatppuccinTheme::FRAPPE.base;
        style.visuals.panel_fill = CatppuccinTheme::FRAPPE.base;
        
        // Set text color to Catppuccin text color
        style.visuals.override_text_color = Some(CatppuccinTheme::FRAPPE.text);
        
        // Apply the themed style to the egui context
        ctx.set_style(style);
        
        // Request a repaint to ensure we check for new PTY data
        ctx.request_repaint();

        // Handle global keyboard shortcuts for terminal focus switching
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Tab) {
                // Switch focus between terminals (0 = left, 1 = right)
                self.focused_terminal = if self.focused_terminal == 0 { 1 } else { 0 };
                println!(
                    "Focus switched to terminal: {}",
                    if self.focused_terminal == 0 {
                        "Left"
                    } else {
                        "Right"
                    }
                );
            }
        });

        // Update output for all tabs and handle restarts
        for tab in &mut self.tabs {
            tab.update_output();
            if tab.needs_restart {
                tab.restart();
            }
        }

        // Compute panel minimum widths from configuration
        let (min_left, min_right) = if self.app_settings.allow_zero_collapse {
            (0.0_f32, 0.0_f32)
        } else {
            (
                self.app_settings.min_left_width.max(0.0),
                self.app_settings.min_right_width.max(0.0),
            )
        };

        // Split-Screen Layout: Terminal 1 (Left) and Terminal 2 (Right)
        egui::SidePanel::left("terminal_1")
            .resizable(true)
            .default_width(ctx.screen_rect().width() * 0.5)
            .min_width(min_left)
            .width_range(min_left..=f32::INFINITY)
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(0.0))
                    .outer_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                // Allow the left panel content to shrink to zero width
                ui.set_min_width(0.0);
                if !self.tabs.is_empty() {
                    let tab = &mut self.tabs[0]; // Terminal 1
                    let is_focused = self.focused_terminal == 0;
                    let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                    let title_color = if is_focused {
                        CatppuccinTheme::FRAPPE.blue
                    } else {
                        CatppuccinTheme::FRAPPE.subtext0
                    };

                    // Header: wrap in a horizontal scroll so it doesn't impose a minimum width
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!(
                                        "{} 🖥️ {}",
                                        focus_indicator,
                                        tab.title()
                                    ))
                                    .color(title_color)
                                    .strong(),
                                )
                                .truncate(true),
                            );
                            if !is_focused {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("(Press Tab to focus)")
                                            .color(CatppuccinTheme::FRAPPE.overlay0)
                                            .italics(),
                                    )
                                    .truncate(true),
                                );
                            }
                        });
                    });
                    ui.separator();

                    // Terminal 1 Output Area with ANSI Color Support
                    egui::ScrollArea::both()
                        .stick_to_bottom(true)
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                            // Render terminal emulator buffer
                            Self::render_terminal_buffer(ui, &tab.terminal_emulator().buffer);
                        });

                    // Terminal 1 Input Area
                    ui.separator();
                    // Input row: wrap in horizontal scroll and allow it to shrink fully
                    let input_response = egui::ScrollArea::horizontal()
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("$");
                                ui.add(
                                    egui::TextEdit::singleline(tab.input_mut())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(ui.available_width()),
                                )
                            })
                            .inner
                        })
                        .inner;

                    // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                    if is_focused {
                        Self::handle_terminal_key_input(ui, &mut tab.pty_writer);
                    }

                    // Handle text input and Enter key
                    if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let mut input_with_newline = tab.input().to_string();
                        input_with_newline.push('\n');

                        if let Some(ref mut writer) = tab.pty_writer {
                            if let Err(e) = writer.write_all(input_with_newline.as_bytes()) {
                                eprintln!("Error writing to PTY: {}", e);
                            }
                        } else {
                            eprintln!("No PTY writer available");
                        }

                        tab.clear_input();
                        input_response.request_focus();
                    }
                }
            });

        // Central panel for Terminal 2 (single divider between left panel and central panel)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Keep a simple lower bound to avoid layout issues and overlap
            ui.set_min_width(min_right);
            if self.tabs.len() > 1 {
                let tab = &mut self.tabs[1]; // Terminal 2
                let is_focused = self.focused_terminal == 1;
                let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                let title_color = if is_focused {
                    CatppuccinTheme::FRAPPE.blue
                } else {
                    CatppuccinTheme::FRAPPE.subtext0
                };

                // Header: wrap in a horizontal scroll so it doesn't impose a minimum width
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{} 🖥️ {}",
                                    focus_indicator,
                                    tab.title()
                                ))
                                .color(title_color)
                                .strong(),
                            )
                            .truncate(true),
                        );
                        if !is_focused {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("(Press Tab to focus)")
                                        .color(CatppuccinTheme::FRAPPE.overlay0)
                                        .italics(),
                                )
                                .truncate(true),
                            );
                        }
                    });
                });
                ui.separator();

                // Terminal 2 Output Area with ANSI Color Support
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .max_height(ui.available_height() - 60.0)
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                        // Render terminal emulator buffer
                        Self::render_terminal_buffer(ui, &tab.terminal_emulator().buffer);
                    });

                // Terminal 2 Input Area
                ui.separator();
                // Input row: wrap in horizontal scroll and allow it to shrink fully
                let input_response = egui::ScrollArea::horizontal()
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("$");
                            ui.add(
                                egui::TextEdit::singleline(tab.input_mut())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(ui.available_width()),
                            )
                        })
                        .inner
                    })
                    .inner;

                // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                if is_focused {
                    Self::handle_terminal_key_input(ui, &mut tab.pty_writer);
                }

                // Handle text input and Enter key
                if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut input_with_newline = tab.input().to_string();
                    input_with_newline.push('\n');

                    if let Some(ref mut writer) = tab.pty_writer {
                        if let Err(e) = writer.write_all(input_with_newline.as_bytes()) {
                            eprintln!("Error writing to PTY: {}", e);
                        }
                    } else {
                        eprintln!("No PTY writer available");
                    }

                    tab.clear_input();
                    input_response.request_focus();
                }
            }
        });
    }
}