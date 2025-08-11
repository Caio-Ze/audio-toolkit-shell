//! # Application Module
//! 
//! This module contains the main application logic including terminal tab management,
//! UI rendering, and application state management.
//! 
//! ## Architecture
//! 
//! The application uses a fixed four-terminal layout: one terminal on the left,
//! two terminals on the top-right (side-by-side), and one terminal on the
//! bottom-right. Each tab runs its own PTY (pseudo-terminal) and can execute
//! different commands.
//! 
//! ## Key Components
//! 
//! - **TerminalTab**: Manages individual terminal instances with PTY communication
//! - **AudioToolkitApp**: Main application state and UI rendering logic
//! 
//! ## Features
//! 
//! - Split-screen terminal interface
//! - Focus cycling with Shift+Tab (Tab is forwarded to the terminal)
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
    focused_terminal: usize, // 0..=3 correspond to the four fixed terminals (L, RT-L, RT-R, RB)
    app_settings: AppSettings,
    // Runtime, interactive split fractions (persist defaults from config)
    right_top_frac: f32,       // top row height share in right cluster
    right_hsplit_frac: f32,    // top-left vs top-right width share in right cluster
    left_buttons_frac: f32,    // fraction of left panel height devoted to buttons container
    // Debug: overlay pane and handle rects
    debug_overlay: bool,
    // Feature flags
    dnd_auto_cd_dirs: bool,
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
        let mut tabs: Vec<TerminalTab> = tabs.into_iter().map(TerminalTab::new).collect();

        // Ensure we have exactly four terminals for the fixed layout:
        // Fill missing with default bash tabs; ignore extras beyond four.
        while tabs.len() < 4 {
            let idx = tabs.len() + 1;
            let cfg = TabConfig {
                title: format!("Terminal {}", idx),
                command: "bash".to_string(),
                auto_restart_on_success: false,
                success_patterns: vec![],
            };
            tabs.push(TerminalTab::new(cfg));
        }
        if tabs.len() > 4 {
            tabs.truncate(4);
        }

        // Initialize interactive split fractions from config defaults
        let right_top_frac = app.right_top_fraction.clamp(0.2, 0.8);
        let right_hsplit_frac = app.right_top_hsplit_fraction.clamp(0.2, 0.8);
        // Start buttons area around ~22% of left panel height; can be adjusted by user
        let left_buttons_frac = 0.22_f32;

        // Debug overlay flag: set via env var ATS_DEBUG_OVERLAY (1/true/yes)
        let debug_overlay = std::env::var("ATS_DEBUG_OVERLAY")
            .ok()
            .map(|v| {
                let v = v.trim();
                v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
            })
            .unwrap_or(false);

        // Optional: automatically cd into a directory when a single folder is dropped
        let dnd_auto_cd_dirs = std::env::var("ATS_DND_AUTO_CD_DIRS")
            .ok()
            .map(|v| {
                let v = v.trim();
                v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
            })
            .unwrap_or(false);

        Self {
            tabs,
            focused_terminal: 0, // Start with left terminal focused
            app_settings: app,
            right_top_frac,
            right_hsplit_frac,
            left_buttons_frac,
            debug_overlay,
            dnd_auto_cd_dirs,
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

    /// Global keyboard input handler: reads from ctx so input is not lost to nested widgets
    fn handle_terminal_key_input_ctx(
        ctx: &egui::Context,
        pty_writer: &mut Option<Box<dyn Write + Send>>,
    ) {
        ctx.input(|i| {
            // Handle all inputs via raw events so no widget consumption can block them
            for ev in &i.events {
                match ev {
                    egui::Event::Text(text) => {
                        // If CTRL is held and a single ASCII char is typed, map to control code.
                        // Example: Ctrl+C -> 0x03 (ETX), Ctrl+D -> 0x04 (EOT), etc.
                        // This enables terminal signals like SIGINT via PTY line discipline.
                        if i.modifiers.ctrl {
                            if text.len() == 1 {
                                let mut ch_iter = text.chars();
                                if let Some(ch) = ch_iter.next() {
                                    if ch.is_ascii() {
                                        let upper = ch.to_ascii_uppercase();
                                        let code = (upper as u8) & 0x1F; // Map A..Z to 0x01..0x1A
                                        if code != 0 { // Skip NUL for non A..Z, still harmless
                                            if let Some(ref mut writer) = pty_writer {
                                                let _ = writer.write_all(&[code]);
                                            }
                                            // Swallow regular text for Ctrl+<char>
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        // Default: forward typed text as-is
                        if let Some(ref mut writer) = pty_writer {
                            let _ = writer.write_all(text.as_bytes());
                        }
                    }
                    egui::Event::Paste(text) => {
                        if let Some(ref mut writer) = pty_writer {
                            let _ = writer.write_all(text.as_bytes());
                        }
                    }
                    egui::Event::Key { key, pressed, repeat: _, modifiers, .. } => {
                        if *pressed {
                            // Handle Ctrl-based TTY signals only (avoid hijacking Command shortcuts)
                            let mut handled_ctrl = false;
                            if modifiers.ctrl {
                                match key {
                                    // Ctrl+C -> ETX (SIGINT)
                                    egui::Key::C => {
                                        if let Some(ref mut writer) = pty_writer {
                                            let _ = writer.write_all(&[0x03]);
                                        }
                                        handled_ctrl = true;
                                    }
                                    // Ctrl+D -> EOT (EOF)
                                    egui::Key::D => {
                                        if let Some(ref mut writer) = pty_writer {
                                            let _ = writer.write_all(&[0x04]);
                                        }
                                        handled_ctrl = true;
                                    }
                                    _ => {}
                                }
                            }
                            if handled_ctrl { return; }
                            match key {
                                egui::Key::Enter => {
                                    if let Some(ref mut writer) = pty_writer {
                                        // CR is the most compatible for PTYs
                                        let _ = writer.write_all(b"\r");
                                    }
                                }
                                egui::Key::Backspace => {
                                    if let Some(ref mut writer) = pty_writer {
                                        // Send DEL (0x7F). If a target app expects BS (0x08), we can add an option later.
                                        let _ = writer.write_all(&[0x7F]);
                                    }
                                }
                                egui::Key::Tab => {
                                    if !modifiers.shift {
                                        if let Some(ref mut writer) = pty_writer {
                                            let _ = writer.write_all(b"\t");
                                        }
                                    }
                                }
                                egui::Key::Escape => {
                                    if let Some(ref mut writer) = pty_writer {
                                        let _ = writer.write_all(b"\x1b");
                                    }
                                }
                                egui::Key::ArrowUp => {
                                    if let Some(ref mut writer) = pty_writer {
                                        let _ = writer.write_all(b"\x1b[A");
                                    }
                                }
                                egui::Key::ArrowDown => {
                                    if let Some(ref mut writer) = pty_writer {
                                        let _ = writer.write_all(b"\x1b[B");
                                    }
                                }
                                egui::Key::ArrowLeft => {
                                    if let Some(ref mut writer) = pty_writer {
                                        let _ = writer.write_all(b"\x1b[D");
                                    }
                                }
                                egui::Key::ArrowRight => {
                                    if let Some(ref mut writer) = pty_writer {
                                        let _ = writer.write_all(b"\x1b[C");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
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

    /// Render a polished action button with rounded background, left accent stripe, and hover ring
    fn render_action_button(
        ui: &mut egui::Ui,
        label: &str,
        accent: egui::Color32,
        size: egui::Vec2,
        is_right_col: bool,
    ) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        // Base paint rect
        let near_right_edge = (rect.max.x + 0.5) >= ui.max_rect().right();
        let mut paint_rect = rect;
        // Seam overlap to defeat AA between columns
        if is_right_col {
            paint_rect.min.x = paint_rect.min.x - 0.5;
        } else {
            paint_rect.max.x = paint_rect.max.x + 0.5;
        }
        // Do not recess the far-right edge anymore (avoid visible cutoff)
        if near_right_edge {
            // no-op: keep full width
            paint_rect.max.x = paint_rect.max.x;
        }

        // Outer-corner rounding: zero out inner corners between columns
        let mut rounding = egui::Rounding::same(4.0);
        if is_right_col {
            // Right column: no rounding on left edge (inner seam)
            rounding.nw = 0.0;
            rounding.sw = 0.0;
        } else {
            // Left column: no rounding on right edge (inner seam)
            rounding.ne = 0.0;
            rounding.se = 0.0;
        }
        let visuals = ui.style().visuals.clone();

        // Neutral background with subtle states (more discrete)
        let bg = if resp.is_pointer_button_down_on() {
            visuals.widgets.inactive.bg_fill
        } else if resp.hovered() {
            CatppuccinTheme::FRAPPE.surface1
        } else {
            CatppuccinTheme::FRAPPE.surface0
        };

        let painter = ui.painter();
        painter.rect_filled(paint_rect, rounding, bg);

        // Left accent stripe (slimmer)
        let stripe_w = 3.0;
        // Anchor stripe to original rect to avoid visual drift when overlapping seams
        let stripe_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, paint_rect.min.y),
            egui::pos2((rect.min.x + stripe_w).min(paint_rect.max.x), paint_rect.max.y),
        );
        painter.rect_filled(
            stripe_rect,
            egui::Rounding {
                nw: rounding.nw,
                ne: 0.0,
                sw: rounding.sw,
                se: 0.0,
            },
            accent,
        );

        // Hover ring (thinner/subtle)
        if resp.hovered() {
            painter.rect_stroke(
                paint_rect,
                rounding,
                egui::Stroke { width: 0.5, color: accent.gamma_multiply(0.4) },
            );
        }

        // Label content (left-aligned, vertically centered)
        // Anchor text to original rect min x + stripe to avoid seam overlap shifting the label
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + stripe_w + 6.0, paint_rect.min.y),
            egui::pos2(paint_rect.max.x - 6.0, paint_rect.max.y),
        );
        ui.allocate_ui_at_rect(inner, |ui| {
            let text = egui::RichText::new(label).size(11.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(text).truncate(true));
            });
        });

        resp
    }

    /// Render only the content for an action button (stripe, label, hover ring) assuming the row background
    /// is already painted. This avoids per-cell background seams.
    fn render_action_button_no_bg(
        ui: &mut egui::Ui,
        label: &str,
        accent: egui::Color32,
        size: egui::Vec2,
        is_right_col: bool,
    ) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let paint_rect = rect; // no seam overlap; row provides background

        // Outer-corner rounding applies only to hover ring; keep inner corners square
        let mut rounding = egui::Rounding::same(4.0);
        if is_right_col {
            rounding.nw = 0.0; rounding.sw = 0.0;
        } else {
            rounding.ne = 0.0; rounding.se = 0.0;
        }

        let painter = ui.painter();

        // Left accent stripe
        let stripe_w = 3.0;
        let stripe_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, paint_rect.min.y),
            egui::pos2((rect.min.x + stripe_w).min(paint_rect.max.x), paint_rect.max.y),
        );
        painter.rect_filled(
            stripe_rect,
            egui::Rounding { nw: rounding.nw, ne: 0.0, sw: rounding.sw, se: 0.0 },
            accent,
        );

        // Hover ring (thin)
        if resp.hovered() {
            painter.rect_stroke(
                paint_rect,
                rounding,
                egui::Stroke { width: 0.5, color: accent.gamma_multiply(0.4) },
            );
        }

        // Label content
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + stripe_w + 6.0, paint_rect.min.y),
            egui::pos2(paint_rect.max.x - 6.0, paint_rect.max.y),
        );
        ui.allocate_ui_at_rect(inner, |ui| {
            let text = egui::RichText::new(label).size(11.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(text).truncate(true));
            });
        });

        resp
    }

    /// Split a rect vertically into (top, handle, bottom) using a fractional split with pixel minimums
    fn split_vertical(
        ui: &mut egui::Ui,
        fraction: &mut f32,
        min_top_px: f32,
        min_bottom_px: f32,
        handle_px: f32,
        id: egui::Id,
    ) -> (egui::Rect, egui::Rect, egui::Rect) {
        let rect = ui.available_rect_before_wrap();
        let total_h = rect.height().max(1.0);

        // Clamp based on pixel minimums
        let min_f = (min_top_px / total_h).clamp(0.0, 0.9);
        let max_f = 1.0 - (min_bottom_px / total_h).clamp(0.0, 0.9);
        *fraction = (*fraction).clamp(min_f, max_f);

        let split_y = rect.top() + total_h * (*fraction);
        let handle_top = split_y - handle_px * 0.5;
        let handle_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), handle_top),
            egui::pos2(rect.right(), handle_top + handle_px),
        );

        // Interaction
        let resp = ui.interact(handle_rect, id, egui::Sense::drag());
        if resp.dragged() {
            let dy = ui.input(|i| i.pointer.delta().y);
            *fraction = ((*fraction) + dy / total_h).clamp(min_f, max_f);
        }

        // Paint handle (subtle)
        let color = if resp.hovered() || resp.dragged() {
            CatppuccinTheme::FRAPPE.overlay0
        } else {
            CatppuccinTheme::FRAPPE.surface1
        };
        ui.painter()
            .rect_filled(handle_rect, 2.0, color.linear_multiply(0.35));

        let top_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), handle_top));
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), handle_top + handle_px),
            rect.max,
        );
        (top_rect, handle_rect, bottom_rect)
    }

    /// Split a rect horizontally into (left, handle, right) using a fractional split with pixel minimums
    fn split_horizontal(
        ui: &mut egui::Ui,
        fraction: &mut f32,
        min_left_px: f32,
        min_right_px: f32,
        handle_px: f32,
        id: egui::Id,
    ) -> (egui::Rect, egui::Rect, egui::Rect) {
        let rect = ui.available_rect_before_wrap();
        let total_w = rect.width().max(1.0);

        // Clamp based on pixel minimums
        let min_f = (min_left_px / total_w).clamp(0.0, 0.9);
        let max_f = 1.0 - (min_right_px / total_w).clamp(0.0, 0.9);
        *fraction = (*fraction).clamp(min_f, max_f);

        let split_x = rect.left() + total_w * (*fraction);
        let handle_left = split_x - handle_px * 0.5;
        let handle_rect = egui::Rect::from_min_max(
            egui::pos2(handle_left, rect.top()),
            egui::pos2(handle_left + handle_px, rect.bottom()),
        );

        // Interaction
        let resp = ui.interact(handle_rect, id, egui::Sense::drag());
        if resp.dragged() {
            let dx = ui.input(|i| i.pointer.delta().x);
            *fraction = ((*fraction) + dx / total_w).clamp(min_f, max_f);
        }

        // Paint handle (subtle)
        let color = if resp.hovered() || resp.dragged() {
            CatppuccinTheme::FRAPPE.overlay0
        } else {
            CatppuccinTheme::FRAPPE.surface1
        };
        ui.painter()
            .rect_filled(handle_rect, 2.0, color.linear_multiply(0.35));

        let left_rect = egui::Rect::from_min_max(rect.min, egui::pos2(handle_left, rect.bottom()));
        let right_rect = egui::Rect::from_min_max(
            egui::pos2(handle_left + handle_px, rect.top()),
            rect.max,
        );
        (left_rect, handle_rect, right_rect)
    }

    /// Shell-quote a filesystem path for POSIX shells (single-quote, escape internal quotes)
    fn shell_quote_path(path: &std::path::Path) -> String {
        let s = path.to_string_lossy();
        let mut out = String::with_capacity(s.len() + 2);
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("'\\\''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
        out
    }

    /// Shell-quote an arbitrary string for POSIX shells (single-quote, escape internal quotes)
    fn shell_quote_str(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push_str("'\\\''");
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
        out
    }

    
    
    /// Single-pass drag-and-drop handler: always route to the focused terminal.
    /// Pointer and rect hit-testing are intentionally ignored.
    fn handle_dnd_single_pass(&mut self, ctx: &egui::Context, focus_rects: &[(usize, egui::Rect)]) {
        // Silence unused parameter warning; rects are collected for potential future use
        let _ = focus_rects;

        let (hovering_files, dropped_files) = ctx.input(|i| {
            (!i.raw.hovered_files.is_empty(), i.raw.dropped_files.clone())
        });

        if hovering_files {
            // Ensure smooth hover visuals while dragging
            ctx.request_repaint();
        }

        if dropped_files.is_empty() {
            return;
        }

        let target = self.focused_terminal;
        if let Some(tab) = self.tabs.get_mut(target) {
            use std::io::Write;
            // If exactly one directory dropped and feature flag set, emit a cd command
            let single_dir = if dropped_files.len() == 1 {
                if let Some(path) = dropped_files[0].path.as_ref() {
                    std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
                } else { false }
            } else { false };

            let text_to_send = if single_dir && self.dnd_auto_cd_dirs {
                if let Some(path) = dropped_files[0].path.as_ref() {
                    format!("cd {} ", Self::shell_quote_path(path))
                } else {
                    format!("cd {} ", Self::shell_quote_str(&dropped_files[0].name))
                }
            } else {
                let mut parts: Vec<String> = Vec::with_capacity(dropped_files.len());
                for f in &dropped_files {
                    if let Some(path) = f.path.as_ref() {
                        parts.push(Self::shell_quote_path(path));
                    } else {
                        parts.push(Self::shell_quote_str(&f.name));
                    }
                }
                let mut s = parts.join(" ");
                s.push(' ');
                s
            };

            if let Some(ref mut writer) = tab.pty_writer {
                let _ = writer.write_all(text_to_send.as_bytes());
            }

            // Keep focus on the focused terminal (explicitly set for clarity)
            self.focused_terminal = target;
        }
    }
    
    /// Compute the header band height used by terminal panels.
    /// This must be kept consistent across `render_terminal_panel` and any
    /// splitter geometry that needs to avoid overlapping the header.
    fn header_band_height(ui: &egui::Ui) -> f32 {
        let styles = &ui.style().text_styles;
        let body = styles
            .get(&egui::TextStyle::Body)
            .map(|f| f.size)
            .unwrap_or(14.0);
        let base = styles
            .get(&egui::TextStyle::Button)
            .map(|f| f.size)
            .unwrap_or(body);
        let title_size = base * 1.3;
        title_size + 10.0
    }

    /// Renders a single terminal panel (header, output). Returns true if the header was clicked.
    /// `tab_index` is used to assign stable, unique widget ids (e.g., ScrollArea state).
    fn render_terminal_panel(
        ui: &mut egui::Ui,
        tab: &mut TerminalTab,
        is_focused: bool,
        tab_index: usize,
        debug_overlay: bool,
    ) -> bool {
        let mut clicked = false;
        let focus_indicator = if is_focused { "🔵" } else { "⚪" };
        let title_color = if is_focused {
            CatppuccinTheme::FRAPPE.blue
        } else {
            CatppuccinTheme::FRAPPE.subtext0
        };

        let frame_inner = egui::Frame::default()
            .fill(ui.style().visuals.panel_fill)
            .stroke(egui::Stroke { width: 0.0, color: egui::Color32::TRANSPARENT })
            .inner_margin(egui::Margin::same(0.0))
            .outer_margin(egui::Margin::same(0.0))
            .show(ui, |ui| {
                // Header: wide clickable band (not just the label)
                let header_h = Self::header_band_height(ui);

                // Allocate a full-width header rect that is clickable anywhere
                let available_w = ui.available_width();
                let (header_rect, header_resp) =
                    ui.allocate_exact_size(egui::vec2(available_w, header_h), egui::Sense::click());
                if debug_overlay {
                    ui.painter().rect_filled(
                        header_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0, 128, 255, 24),
                    );
                }
                ui.allocate_ui_at_rect(header_rect, |ui| {
                    ui.horizontal(|ui| {
                        let title = egui::RichText::new(format!("{} 🖥️ {}", focus_indicator, tab.title()))
                            .color(title_color)
                            .strong()
                            .size((header_h - 10.0).max(1.0));
                        ui.add(egui::Label::new(title).truncate(true));
                        if !is_focused {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("(Click to focus)")
                                        .color(CatppuccinTheme::FRAPPE.overlay0)
                                        .italics(),
                                )
                                .truncate(true),
                            );
                        }
                    });
                });
                if header_resp.clicked() {
                    if debug_overlay { eprintln!("[FOCUS] header clicked idx={} rect={:?}", tab_index, header_rect); }
                    clicked = true;
                }

                // Output: occupy full width; sticky to bottom; assign unique id per terminal to avoid shared scrolling
                egui::ScrollArea::vertical()
                    .id_source(egui::Id::new(("terminal_output_scroll_idx", tab_index)))
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        // Ensure inner content takes the full available width (flush to the right edge)
                        ui.set_width(ui.available_width());
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                        Self::render_terminal_buffer(ui, &tab.terminal_emulator().buffer);
                    });
            });
        // Draw a more visible focus border around the entire panel.
        // When a drag is in progress, add a soft outer glow to make the drop target obvious.
        if is_focused {
            let rect = frame_inner.response.rect;
            let p = ui.painter();
            // Subtle glow only while dragging files over the app
            let hovering_dnd = ui.input(|i| !i.raw.hovered_files.is_empty());
            if hovering_dnd {
                p.rect_stroke(
                    rect,
                    egui::Rounding::same(3.0),
                    egui::Stroke { width: 4.0, color: CatppuccinTheme::FRAPPE.blue.linear_multiply(0.35) },
                );
            }
            // Crisp border for the focused panel
            p.rect_stroke(
                rect,
                egui::Rounding::same(3.0),
                egui::Stroke { width: 2.0, color: CatppuccinTheme::FRAPPE.blue },
            );
        }
        clicked
    }

    /// Handles terminal keyboard input including arrow keys and special keys (panel-local)
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
            // Forward text input and paste directly to PTY
            for ev in &i.events {
                match ev {
                    egui::Event::Text(text) => {
                        if let Some(ref mut writer) = pty_writer {
                            let _ = writer.write_all(text.as_bytes());
                        }
                    }
                    egui::Event::Paste(text) => {
                        if let Some(ref mut writer) = pty_writer {
                            let _ = writer.write_all(text.as_bytes());
                        }
                    }
                    _ => {}
                }
            }

            // Special keys
            if i.key_pressed(egui::Key::Enter) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\n");
                }
            }
            if i.key_pressed(egui::Key::Backspace) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(&[0x7F]); // DEL
                }
            }
            // Forward Tab to PTY; Shift+Tab is reserved for focus cycling globally
            if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\t");
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b");
                }
            }

            // Arrow keys -> CSI sequences
            if i.key_pressed(egui::Key::ArrowUp) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[A");
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[B");
                }
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[D");
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if let Some(ref mut writer) = pty_writer {
                    let _ = writer.write_all(b"\x1b[C");
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
        // Make splitters easier to grab so users can resize panels even when narrow
        style.interaction.resize_grab_radius_side = 12.0;
        
        // Apply the themed style to the egui context
        ctx.set_style(style);
        
        // Keep repainting to poll PTY data and handle input
        ctx.request_repaint();

        // Global keyboard shortcut: cycle focus across terminals (Shift+Tab)
        if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::Tab)) {
            if !self.tabs.is_empty() {
                self.focused_terminal = (self.focused_terminal + 1) % self.tabs.len().max(1);
            }
        }

        // Update output for all tabs and handle restarts
        for tab in &mut self.tabs {
            tab.update_output();
            if tab.needs_restart {
                tab.restart();
            }
        }

        // Collect DnD focus rects for this frame
        let mut dnd_focus_rects: Vec<(usize, egui::Rect)> = Vec::new();

        // Fixed layout + interactive right cluster (Plan v2):
        // - Left column fixed to 40% of window width; non-resizable.
        // - Buttons box occupies the lower 30% of the left column (scrolls internally).
        // - Right cluster vertical split (top vs bottom) is interactive; defaults come from config
        //   (`right_top_fraction`, e.g., 0.6 for ~60% top / ~40% bottom). Terminal 4 is not fixed.
        let screen_w = ctx.screen_rect().width();
        let left_w = (screen_w * 0.40).max(1.0);
        let left_panel = egui::SidePanel::left("terminal_1")
            .resizable(false)
            .default_width(left_w)
            .min_width(left_w)
            .width_range(left_w..=left_w)
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    // Reserve a small right gutter so content never covers the resize handle
                    .inner_margin(egui::Margin { left: 0.0, right: 12.0, top: 0.0, bottom: 0.0 })
                    .outer_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                // Allow the left panel content to shrink to zero width
                ui.set_min_width(0.0);

                // Fixed vertical split inside left column: 70% Terminal 1 (top), 30% Buttons (bottom)
                let rect = ui.available_rect_before_wrap();
                let total_h = rect.height().max(1.0);
                let split_y = rect.top() + total_h * 0.70;
                let t1_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), split_y));
                let btn_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), split_y), rect.max);

                // Top: Terminal 1
                let mut t1_ui = ui.child_ui(t1_rect, egui::Layout::top_down(egui::Align::Min));
                if self.tabs.len() >= 1 {
                    let tab = &mut self.tabs[0];
                    let is_focused = self.focused_terminal == 0;
                    let clicked = {
                        let mut clicked_local = false;
                        t1_ui.push_id(("pane_scope", 0usize), |ui| {
                            clicked_local = Self::render_terminal_panel(ui, tab, is_focused, 0, self.debug_overlay);
                        });
                        clicked_local
                    };
                    if clicked {
                        self.focused_terminal = 0;
                    }

                    // Panel-wide click-to-focus for Terminal 1 (pane minus header band)
                    let t1_focus_rect = egui::Rect::from_min_max(
                        egui::pos2(t1_rect.left(), t1_rect.top() + Self::header_band_height(ui)),
                        t1_rect.max,
                    );
                    if self.debug_overlay {
                        ui.painter().rect_filled(
                            t1_focus_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(0, 128, 255, 16),
                        );
                    }
                    let t1_resp = ui.interact(
                        t1_focus_rect,
                        egui::Id::new(("pane_focus_zone", 0usize)),
                        egui::Sense::click(),
                    );
                    if t1_resp.clicked() {
                        if self.debug_overlay { eprintln!("[FOCUS] pane click -> T1 left (idx 0)"); }
                        self.focused_terminal = 0;
                    }
                    // Record DnD drop rect (full panel including header) for single-pass routing
                    dnd_focus_rects.push((0, t1_rect));
                }

                // Bottom: Buttons container with its own scroll area
                let mut btn_ui = ui.child_ui(btn_rect, egui::Layout::top_down(egui::Align::Min));
                btn_ui.label(
                    egui::RichText::new("🛠️ Actions")
                        .color(CatppuccinTheme::FRAPPE.overlay0)
                        .size(11.0),
                );
                btn_ui.add_space(4.0);
                // Compute remaining height for the grid and derive button height to fill it exactly
                let avail_h = btn_ui.available_height().max(1.0);
                let rows: usize = 4; // currently 8 buttons in 2 columns -> 4 rows
                let v_spacing = 8.0;
                let button_height = ((avail_h - v_spacing * (rows.saturating_sub(1) as f32)) / rows as f32)
                    .clamp(24.0, 54.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(&mut btn_ui, |ui| {
                        // Remove default item spacing so our Grid spacing controls layout precisely
                        {
                            let spacing_mut = &mut ui.style_mut().spacing;
                            spacing_mut.item_spacing.x = 0.0;
                            spacing_mut.item_spacing.y = 0.0;
                        }
                        // Calculate responsive button size to fill the container width (two columns)
                        let h_spacing = 0.0; // no gap between columns
                        let spacing = v_spacing; // vertical spacing only
                        let total_w = ui.available_width();
                        ui.set_width(total_w);
                        // Split width across two columns; use rounding to balance and assign exact remainder to right
                        let col_w_left = ((total_w - h_spacing) * 0.5).round().max(1.0);
                        let col_w_right = (total_w - h_spacing - col_w_left).max(1.0);
                        let button_size_left = egui::vec2(col_w_left, button_height);
                        let button_size_right = egui::vec2(col_w_right, button_height);

                        let row_prepass = std::env::var("ATS_BTN_ROW_PREPASS")
                            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                            .unwrap_or(true);

                        if row_prepass {
                            // Paint one row-wide background per row, then render two half-width buttons without BG
                            let rows_meta = [
                                (("🔄 Restart All", CatppuccinTheme::FRAPPE.blue),   ("📁 File Manager", CatppuccinTheme::FRAPPE.lavender)),
                                (("⚙️ Settings",    CatppuccinTheme::FRAPPE.sapphire),("🔧 Tools",        CatppuccinTheme::FRAPPE.peach)),
                                (("📊 Analytics",   CatppuccinTheme::FRAPPE.green),  ("🔖 Bookmarks",   CatppuccinTheme::FRAPPE.pink)),
                                (("📜 Scripts",     CatppuccinTheme::FRAPPE.mauve),  ("💡 Help",        CatppuccinTheme::FRAPPE.yellow)),
                            ];

                            for (i, ((l_label, l_acc), (r_label, r_acc))) in rows_meta.iter().enumerate() {
                                // Row rect and background with outer rounding on extremes
                                let (row_rect, _) = ui.allocate_exact_size(egui::vec2(total_w, button_height), egui::Sense::hover());
                                let mut rounding = egui::Rounding::ZERO;
                                if i == 0 { rounding.nw = 4.0; rounding.ne = 4.0; }
                                if i + 1 == rows_meta.len() { rounding.sw = 4.0; rounding.se = 4.0; }
                                ui.painter().rect_filled(row_rect, rounding, CatppuccinTheme::FRAPPE.surface0);

                                // Split row into left/right rects
                                let left_rect = egui::Rect::from_min_size(row_rect.min, egui::vec2(col_w_left, button_height));
                                let right_min = egui::pos2(row_rect.min.x + col_w_left, row_rect.min.y);
                                let right_rect = egui::Rect::from_min_size(right_min, egui::vec2(col_w_right, button_height));

                                // Left button
                                let mut left_clicked = false;
                                ui.allocate_ui_at_rect(left_rect, |ui| {
                                    if Self::render_action_button_no_bg(ui, l_label, *l_acc, button_size_left, false)
                                        .on_hover_text(match i { 0 => "Restart all terminals", 1 => "Open settings (coming soon)", 2 => "Performance analytics (coming soon)", _ => "Script management (coming soon)" })
                                        .clicked() {
                                        left_clicked = true;
                                    }
                                });
                                if left_clicked {
                                    if i == 0 { for tab in &mut self.tabs { tab.needs_restart = true; } }
                                }

                                // Right button
                                let mut right_clicked = false;
                                ui.allocate_ui_at_rect(right_rect, |ui| {
                                    if Self::render_action_button_no_bg(ui, r_label, *r_acc, button_size_right, true)
                                        .on_hover_text(match i { 0 => "Open file manager (coming soon)", 1 => "Developer tools (coming soon)", 2 => "Command bookmarks (coming soon)", _ => "Help & documentation (coming soon)" })
                                        .clicked() {
                                        right_clicked = true;
                                    }
                                });
                                if right_clicked { /* placeholders */ }

                                if i + 1 < rows_meta.len() { ui.add_space(spacing); }
                            }
                        } else {
                            // Legacy Grid path (per-cell backgrounds)
                            egui::Grid::new("action_buttons")
                                .num_columns(2)
                                .spacing([h_spacing, spacing])
                                .show(ui, |ui| {
                                    if Self::render_action_button(ui, "🔄 Restart All", CatppuccinTheme::FRAPPE.blue, button_size_left, false)
                                        .on_hover_text("Restart all terminals").clicked() {
                                        for tab in &mut self.tabs { tab.needs_restart = true; }
                                    }
                                    if Self::render_action_button(ui, "📁 File Manager", CatppuccinTheme::FRAPPE.lavender, button_size_right, true)
                                        .on_hover_text("Open file manager (coming soon)").clicked() {
                                    }
                                    ui.end_row();
                                    if Self::render_action_button(ui, "⚙️ Settings", CatppuccinTheme::FRAPPE.sapphire, button_size_left, false)
                                        .on_hover_text("Open settings (coming soon)").clicked() {}
                                    if Self::render_action_button(ui, "🔧 Tools", CatppuccinTheme::FRAPPE.peach, button_size_right, true)
                                        .on_hover_text("Developer tools (coming soon)").clicked() {}
                                    ui.end_row();
                                    if Self::render_action_button(ui, "📊 Analytics", CatppuccinTheme::FRAPPE.green, button_size_left, false)
                                        .on_hover_text("Performance analytics (coming soon)").clicked() {}
                                    if Self::render_action_button(ui, "🔖 Bookmarks", CatppuccinTheme::FRAPPE.pink, button_size_right, true)
                                        .on_hover_text("Command bookmarks (coming soon)").clicked() {}
                                    ui.end_row();
                                    if Self::render_action_button(ui, "📜 Scripts", CatppuccinTheme::FRAPPE.mauve, button_size_left, false)
                                        .on_hover_text("Script management (coming soon)").clicked() {}
                                    if Self::render_action_button(ui, "💡 Help", CatppuccinTheme::FRAPPE.yellow, button_size_right, true)
                                        .on_hover_text("Help & documentation (coming soon)").clicked() {}
                                    ui.end_row();
                                });
                        }
                    });
            });

        // Right cluster in a CentralPanel: top row (two terminals), bottom row (one terminal)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(0.0))
                    .outer_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                // Interactive splits with visible dividers; handles registered AFTER content to ensure precedence
                let rect = ui.available_rect_before_wrap();
                let total_h = rect.height().max(1.0);
                const HANDLE_THICK: f32 = 10.0;
                const MIN_TOP: f32 = 140.0;
                const MIN_BOTTOM: f32 = 140.0;
                const MIN_LEFT: f32 = 160.0;
                const MIN_RIGHT: f32 = 160.0;

                // Vertical split (top vs bottom)
                let min_vf = (MIN_TOP / total_h).clamp(0.0, 0.9);
                let max_vf = 1.0 - (MIN_BOTTOM / total_h).clamp(0.0, 0.9);
                self.right_top_frac = self.right_top_frac.clamp(min_vf, max_vf);
                if !self.right_top_frac.is_finite() || self.right_top_frac <= 0.0 { self.right_top_frac = 0.65; }
                let vsplit_y = rect.top() + total_h * self.right_top_frac;
                let v_handle_top = vsplit_y - HANDLE_THICK * 0.5;
                let v_handle_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), v_handle_top),
                    egui::pos2(rect.right(), v_handle_top + HANDLE_THICK),
                );
                let top_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), v_handle_top));
                let bottom_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), v_handle_top + HANDLE_THICK), rect.max);

                // Horizontal split (left vs right) within top_rect
                let total_w = top_rect.width().max(1.0);
                let min_hf = (MIN_LEFT / total_w).clamp(0.0, 0.9);
                let max_hf = 1.0 - (MIN_RIGHT / total_w).clamp(0.0, 0.9);
                self.right_hsplit_frac = self.right_hsplit_frac.clamp(min_hf, max_hf);
                if !self.right_hsplit_frac.is_finite() || self.right_hsplit_frac <= 0.0 { self.right_hsplit_frac = 0.5; }
                let hsplit_x = top_rect.left() + total_w * self.right_hsplit_frac;
                let h_handle_left = hsplit_x - HANDLE_THICK * 0.5;
                // Compute header band height to avoid covering header clicks with the handle.
                let header_h_band = Self::header_band_height(ui);
                let h_top = (top_rect.top() + header_h_band).min(top_rect.bottom());
                let h_handle_rect = egui::Rect::from_min_max(
                    egui::pos2(h_handle_left, h_top),
                    egui::pos2(h_handle_left + HANDLE_THICK, top_rect.bottom()),
                );
                let left_rect = egui::Rect::from_min_max(top_rect.min, egui::pos2(h_handle_left, top_rect.bottom()));
                let right_rect = egui::Rect::from_min_max(egui::pos2(h_handle_left + HANDLE_THICK, top_rect.top()), top_rect.max);
                if self.debug_overlay {
                    let p = ui.painter();
                    p.rect_filled(left_rect, 0.0, egui::Color32::from_rgba_unmultiplied(0, 0, 255, 24));
                    p.rect_filled(right_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 0, 255, 24));
                    p.rect_filled(h_handle_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 0, 48));
                }

                if self.debug_overlay {
                    let p = ui.painter();
                    p.rect_filled(top_rect, 0.0, egui::Color32::from_rgba_unmultiplied(0, 128, 0, 20));
                    p.rect_filled(bottom_rect, 0.0, egui::Color32::from_rgba_unmultiplied(128, 0, 0, 20));
                    p.rect_filled(v_handle_rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 0, 48));
                }

                // Top-left terminal (tab 1 index)
                let mut top_left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::Min));
                if self.tabs.len() >= 2 {
                    let tab = &mut self.tabs[1];
                    let is_focused = self.focused_terminal == 1;
                    let clicked = {
                        let mut clicked_local = false;
                        top_left_ui.push_id(("pane_scope", 1usize), |ui| {
                            clicked_local = Self::render_terminal_panel(ui, tab, is_focused, 1, self.debug_overlay);
                        });
                        clicked_local
                    };
                    if clicked {
                        if self.debug_overlay { eprintln!("[FOCUS] Focus -> T2 top-left (idx 1)"); }
                        self.focused_terminal = 1;
                    }
                }

                // Top-right terminal (tab 2 index)
                let mut top_right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::Min));
                if self.tabs.len() >= 3 {
                    let tab = &mut self.tabs[2];
                    let is_focused = self.focused_terminal == 2;
                    let clicked = {
                        let mut clicked_local = false;
                        top_right_ui.push_id(("pane_scope", 2usize), |ui| {
                            clicked_local = Self::render_terminal_panel(ui, tab, is_focused, 2, self.debug_overlay);
                        });
                        clicked_local
                    };
                    if clicked {
                        if self.debug_overlay { eprintln!("[FOCUS] Focus -> T3 top-right (idx 2)"); }
                        self.focused_terminal = 2;
                    }
                }

                // Bottom terminal (tab 3 index)
                let mut bottom_ui = ui.child_ui(bottom_rect, egui::Layout::top_down(egui::Align::Min));
                if self.tabs.len() >= 4 {
                    let tab = &mut self.tabs[3];
                    let is_focused = self.focused_terminal == 3;
                    let clicked = {
                        let mut clicked_local = false;
                        bottom_ui.push_id(("pane_scope", 3usize), |ui| {
                            clicked_local = Self::render_terminal_panel(ui, tab, is_focused, 3, self.debug_overlay);
                        });
                        clicked_local
                    };
                    if clicked {
                        if self.debug_overlay { eprintln!("[FOCUS] Focus -> T4 bottom (idx 3)"); }
                        self.focused_terminal = 3;
                    }
                }

                // Panel-wide click-to-focus zones for right cluster (registered BEFORE handle interactions)
                if self.tabs.len() >= 2 {
                    let left_focus_rect = egui::Rect::from_min_max(
                        egui::pos2(left_rect.left(), left_rect.top() + Self::header_band_height(ui)),
                        left_rect.max,
                    );
                    // Record DnD drop rect (full panel including header) for single-pass routing
                    dnd_focus_rects.push((1, left_rect));
                    let resp = ui.interact(
                        left_focus_rect,
                        egui::Id::new(("pane_focus_zone", 1usize)),
                        egui::Sense::click(),
                    );
                    if resp.clicked() {
                        if self.debug_overlay { eprintln!("[FOCUS] pane click -> T2 top-left (idx 1)"); }
                        self.focused_terminal = 1;
                    }
                }
                if self.tabs.len() >= 3 {
                    let right_focus_rect = egui::Rect::from_min_max(
                        egui::pos2(right_rect.left(), right_rect.top() + Self::header_band_height(ui)),
                        right_rect.max,
                    );
                    let resp = ui.interact(
                        right_focus_rect,
                        egui::Id::new(("pane_focus_zone", 2usize)),
                        egui::Sense::click(),
                    );
                    if resp.clicked() {
                        if self.debug_overlay { eprintln!("[FOCUS] pane click -> T3 top-right (idx 2)"); }
                        self.focused_terminal = 2;
                    }
                    // Record DnD drop rect (full panel including header) for single-pass routing
                    dnd_focus_rects.push((2, right_rect));
                }
                if self.tabs.len() >= 4 {
                    let bottom_focus_rect = egui::Rect::from_min_max(
                        egui::pos2(bottom_rect.left(), bottom_rect.top() + Self::header_band_height(ui)),
                        bottom_rect.max,
                    );
                    // Record DnD drop rect (full panel including header) for single-pass routing
                    dnd_focus_rects.push((3, bottom_rect));
                    let resp = ui.interact(
                        bottom_focus_rect,
                        egui::Id::new(("pane_focus_zone", 3usize)),
                        egui::Sense::click(),
                    );
                    if resp.clicked() {
                        if self.debug_overlay { eprintln!("[FOCUS] pane click -> T4 bottom (idx 3)"); }
                        self.focused_terminal = 3;
                    }
                }

                // Register interactions AFTER content so handles are on top
                let v_resp = ui.interact(v_handle_rect, egui::Id::new("right_vsplit"), egui::Sense::drag());
                if v_resp.dragged() {
                    let dy = ui.input(|i| i.pointer.delta().y);
                    let new_f = (self.right_top_frac + dy / total_h).clamp(min_vf, max_vf);
                    self.right_top_frac = new_f;
                    if self.debug_overlay { eprintln!("[SPLIT] Vertical drag dy={:.1} new_top_frac={:.3}", dy, self.right_top_frac); }
                }
                let h_resp = ui.interact(h_handle_rect, egui::Id::new("right_hsplit"), egui::Sense::drag());
                if h_resp.dragged() {
                    let dx = ui.input(|i| i.pointer.delta().x);
                    let new_f = (self.right_hsplit_frac + dx / total_w).clamp(min_hf, max_hf);
                    self.right_hsplit_frac = new_f;
                    if self.debug_overlay { eprintln!("[SPLIT] Horizontal drag dx={:.1} new_hsplit_frac={:.3}", dx, self.right_hsplit_frac); }
                }

                // Paint visible dividers/handles
                let handle_color_v = if v_resp.hovered() || v_resp.dragged() {
                    CatppuccinTheme::FRAPPE.overlay0
                } else {
                    CatppuccinTheme::FRAPPE.surface1
                };
                let handle_color_h = if h_resp.hovered() || h_resp.dragged() {
                    CatppuccinTheme::FRAPPE.overlay0
                } else {
                    CatppuccinTheme::FRAPPE.surface1
                };
                let painter = ui.painter();
                painter.rect_filled(v_handle_rect, 2.0, handle_color_v.linear_multiply(0.35));
                painter.rect_filled(h_handle_rect, 2.0, handle_color_h.linear_multiply(0.35));
                // Optional center strokes for crispness
                painter.hline(
                    v_handle_rect.x_range(),
                    v_handle_rect.center().y,
                    egui::Stroke { width: 1.0, color: handle_color_v },
                );
                painter.vline(
                    h_handle_rect.center().x,
                    h_handle_rect.y_range(),
                    egui::Stroke { width: 1.0, color: handle_color_h },
                );
            });

        // Elegant divider between left panel and right cluster
        // Single-pass DnD routing based on the collected focus rects
        self.handle_dnd_single_pass(ctx, &dnd_focus_rects);

        let x = left_panel.response.rect.right().round();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("divider_left_right"),
        ));
        painter.vline(
            x,
            ctx.screen_rect().y_range(),
            egui::Stroke { width: 1.0, color: CatppuccinTheme::FRAPPE.surface1 },
        );
        
        // Forward keyboard input to the currently focused terminal's PTY
        if let Some(tab) = self.tabs.get_mut(self.focused_terminal) {
            Self::handle_terminal_key_input_ctx(ctx, &mut tab.pty_writer);
        }
    }
}