// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, Frame};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

#[derive(Clone)]
struct TerminalCell {
    character: char,
    color: egui::Color32,
    bold: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            character: ' ',
            color: egui::Color32::WHITE,
            bold: false,
        }
    }
}

#[derive(Clone)]
struct TerminalEmulator {
    buffer: Vec<Vec<TerminalCell>>,
    cursor_row: usize,
    cursor_col: usize,
    rows: usize,
    cols: usize,
    current_color: egui::Color32,
    bold: bool,
}

impl TerminalEmulator {
    fn new(rows: usize, cols: usize) -> Self {
        let buffer = vec![vec![TerminalCell::default(); cols]; rows];
        Self {
            buffer,
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
            current_color: egui::Color32::WHITE,
            bold: false,
        }
    }

    fn clear_screen(&mut self) {
        for row in &mut self.buffer {
            for cell in row {
                *cell = TerminalCell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
    }

    fn write_char(&mut self, ch: char) {
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            // Determine color: use white/gray for box drawing characters, current color for text
            let char_color = if self.is_box_drawing_char(ch) {
                egui::Color32::from_rgb(128, 128, 128) // Gray for box drawing
            } else {
                self.current_color
            };
            
            // Write the character to the current position
            self.buffer[self.cursor_row][self.cursor_col] = TerminalCell {
                character: ch,
                color: char_color,
                bold: self.bold,
            };

            // Advance cursor by 1 (simple approach)
            self.cursor_col += 1;
            if self.cursor_col >= self.cols {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    // Scroll up (simple implementation)
                    self.buffer.remove(0);
                    self.buffer.push(vec![TerminalCell::default(); self.cols]);
                    self.cursor_row = self.rows - 1;
                }
            }
        }
    }
    
    fn is_box_drawing_char(&self, ch: char) -> bool {
        matches!(ch, 
            // Box drawing characters (Unicode block 2500-257F)
            '─' | '━' | '│' | '┃' | '┌' | '┍' | '┎' | '┏' | 
            '┐' | '┑' | '┒' | '┓' | '└' | '┕' | '┖' | '┗' | 
            '┘' | '┙' | '┚' | '┛' | '├' | '┝' | '┞' | '┟' | 
            '┠' | '┡' | '┢' | '┣' | '┤' | '┥' | '┦' | '┧' | 
            '┨' | '┩' | '┪' | '┫' | '┬' | '┭' | '┮' | '┯' | 
            '┰' | '┱' | '┲' | '┳' | '┴' | '┵' | '┶' | '┷' | 
            '┸' | '┹' | '┺' | '┻' | '┼' | '┽' | '┾' | '┿' | 
            '╀' | '╁' | '╂' | '╃' | '╄' | '╅' | '╆' | '╇' | 
            '╈' | '╉' | '╊' | '╋' | '╌' | '╍' | '╎' | '╏' | 
            '═' | '║' | '╒' | '╓' | '╔' | '╕' | '╖' | '╗' | 
            '╘' | '╙' | '╚' | '╛' | '╜' | '╝' | '╞' | '╟' | 
            '╠' | '╡' | '╢' | '╣' | '╤' | '╥' | '╦' | '╧' | 
            '╨' | '╩' | '╪' | '╫' | '╬' | '╭' | '╮' | '╯' | '╰'
        )
    }

    fn handle_newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            // Scroll up
            self.buffer.remove(0);
            self.buffer.push(vec![TerminalCell::default(); self.cols]);
            self.cursor_row = self.rows - 1;
        }
    }

    fn handle_carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn process_ansi_data(&mut self, data: &str) {
        let mut chars = data.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                // ESC character
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['

                    // Parse the ANSI sequence
                    let mut sequence = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_alphabetic() || "~".contains(next_ch) {
                            let cmd = chars.next().unwrap();
                            sequence.push(cmd);
                            break;
                        } else {
                            sequence.push(chars.next().unwrap());
                        }
                    }

                    self.handle_ansi_sequence(&sequence);
                } else {
                    // Handle other escape sequences if needed
                    self.write_char(ch);
                }
            } else if ch == '\n' {
                self.handle_newline();
            } else if ch == '\r' {
                self.handle_carriage_return();
            } else if ch == '\t' {
                // Handle tab - move to next tab stop (every 8 characters)
                let next_tab = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next_tab.min(self.cols - 1);
            } else if ch.is_control() {
                // Skip other control characters for now
                continue;
            } else {
                self.write_char(ch);
            }
        }
    }

    fn handle_ansi_sequence(&mut self, sequence: &str) {
        if sequence.is_empty() {
            return;
        }

        let cmd = sequence.chars().last().unwrap();
        let params: Vec<&str> = sequence[..sequence.len() - 1].split(';').collect();

        match cmd {
            'H' | 'f' => {
                // Cursor position
                let row = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .saturating_sub(1);
                let col = params
                    .get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .saturating_sub(1);
                self.move_cursor(row, col);
            }
            'A' => {
                // Cursor up
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.cursor_row = self.cursor_row.saturating_sub(count);
            }
            'B' => {
                // Cursor down
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.cursor_row = (self.cursor_row + count).min(self.rows - 1);
            }
            'C' => {
                // Cursor forward
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.cursor_col = (self.cursor_col + count).min(self.cols - 1);
            }
            'D' => {
                // Cursor backward
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.cursor_col = self.cursor_col.saturating_sub(count);
            }
            'J' => {
                // Clear screen
                let mode = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end of screen
                        // Clear current line from cursor
                        for col in self.cursor_col..self.cols {
                            if self.cursor_row < self.rows {
                                self.buffer[self.cursor_row][col] = TerminalCell::default();
                            }
                        }
                        // Clear all lines below
                        for row in (self.cursor_row + 1)..self.rows {
                            for col in 0..self.cols {
                                self.buffer[row][col] = TerminalCell::default();
                            }
                        }
                    }
                    1 => {
                        // Clear from beginning of screen to cursor
                        // Clear all lines above
                        for row in 0..self.cursor_row {
                            for col in 0..self.cols {
                                self.buffer[row][col] = TerminalCell::default();
                            }
                        }
                        // Clear current line to cursor
                        for col in 0..=self.cursor_col {
                            if self.cursor_row < self.rows {
                                self.buffer[self.cursor_row][col] = TerminalCell::default();
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Clear line
                let mode = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                if self.cursor_row < self.rows {
                    match mode {
                        0 => {
                            // Clear from cursor to end of line
                            for col in self.cursor_col..self.cols {
                                self.buffer[self.cursor_row][col] = TerminalCell::default();
                            }
                        }
                        1 => {
                            // Clear from beginning of line to cursor
                            for col in 0..=self.cursor_col {
                                self.buffer[self.cursor_row][col] = TerminalCell::default();
                            }
                        }
                        2 => {
                            // Clear entire line
                            for col in 0..self.cols {
                                self.buffer[self.cursor_row][col] = TerminalCell::default();
                            }
                        }
                        _ => {}
                    }
                }
            }
            'm' => {
                // Set graphics mode (colors, bold, etc.)
                self.handle_graphics_mode(&params);
            }
            _ => {
                // Unknown sequence - ignore for now
            }
        }
    }

    fn handle_graphics_mode(&mut self, params: &[&str]) {
        if params.is_empty() || (params.len() == 1 && params[0].is_empty()) {
            // Reset
            self.current_color = egui::Color32::WHITE;
            self.bold = false;
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                "0" => {
                    self.current_color = egui::Color32::WHITE;
                    self.bold = false;
                }
                "1" => self.bold = true,
                "22" => self.bold = false,
                "30" => self.current_color = egui::Color32::BLACK,
                "31" => self.current_color = egui::Color32::from_rgb(205, 49, 49),
                "32" => self.current_color = egui::Color32::from_rgb(13, 188, 121),
                "33" => self.current_color = egui::Color32::from_rgb(229, 229, 16),
                "34" => self.current_color = egui::Color32::from_rgb(36, 114, 200),
                "35" => self.current_color = egui::Color32::from_rgb(188, 63, 188),
                "36" => self.current_color = egui::Color32::from_rgb(17, 168, 205),
                "37" => self.current_color = egui::Color32::WHITE,
                "90" => self.current_color = egui::Color32::from_rgb(102, 102, 102),
                "91" => self.current_color = egui::Color32::from_rgb(241, 76, 76),
                "92" => self.current_color = egui::Color32::from_rgb(35, 209, 139),
                "93" => self.current_color = egui::Color32::from_rgb(245, 245, 67),
                "94" => self.current_color = egui::Color32::from_rgb(59, 142, 234),
                "95" => self.current_color = egui::Color32::from_rgb(214, 112, 214),
                "96" => self.current_color = egui::Color32::from_rgb(41, 184, 219),
                "97" => self.current_color = egui::Color32::WHITE,
                "38" if i + 2 < params.len() && params[i + 1] == "5" => {
                    // 256-color foreground
                    if let Ok(color_index) = params[i + 2].parse::<u8>() {
                        self.current_color = ansi_256_to_rgb(color_index);
                    }
                    i += 2; // Skip the next two parameters
                }
                _ => {
                    // Unknown parameter - ignore
                }
            }
            i += 1;
        }
    }
}

// Helper function for 256-color ANSI to RGB conversion
fn ansi_256_to_rgb(color_index: u8) -> egui::Color32 {
    match color_index {
        // Standard colors (0-15)
        0 => egui::Color32::BLACK,
        1 => egui::Color32::from_rgb(128, 0, 0), // Dark Red
        2 => egui::Color32::from_rgb(0, 128, 0), // Dark Green
        3 => egui::Color32::from_rgb(128, 128, 0), // Dark Yellow
        4 => egui::Color32::from_rgb(0, 0, 128), // Dark Blue
        5 => egui::Color32::from_rgb(128, 0, 128), // Dark Magenta
        6 => egui::Color32::from_rgb(0, 128, 128), // Dark Cyan
        7 => egui::Color32::from_rgb(192, 192, 192), // Light Gray
        8 => egui::Color32::from_rgb(128, 128, 128), // Dark Gray
        9 => egui::Color32::from_rgb(255, 0, 0), // Bright Red
        10 => egui::Color32::from_rgb(0, 255, 0), // Bright Green
        11 => egui::Color32::from_rgb(255, 255, 0), // Bright Yellow
        12 => egui::Color32::from_rgb(0, 0, 255), // Bright Blue
        13 => egui::Color32::from_rgb(255, 0, 255), // Bright Magenta
        14 => egui::Color32::from_rgb(0, 255, 255), // Bright Cyan
        15 => egui::Color32::WHITE,              // Bright White

        // 216 color cube (16-231)
        16..=231 => {
            let index = color_index - 16;
            let r = (index / 36) * 51;
            let g = ((index % 36) / 6) * 51;
            let b = (index % 6) * 51;
            egui::Color32::from_rgb(r, g, b)
        }

        // Grayscale colors (232-255)
        232..=255 => {
            let gray = 8 + (color_index - 232) * 10;
            egui::Color32::from_rgb(gray, gray, gray)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AppConfig {
    app: AppSettings,
    tabs: Vec<TabConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppSettings {
    name: String,
    window_width: f32,
    window_height: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TabConfig {
    title: String,
    command: String,
    auto_restart_on_success: bool,
    success_patterns: Vec<String>,
}

fn load_config() -> AppConfig {
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

fn default_config() -> AppConfig {
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

struct TerminalTab {
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
    fn new(config: TabConfig) -> Self {
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

    fn update_output(&mut self) {
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

    fn restart(&mut self) {
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

struct AudioToolkitApp {
    tabs: Vec<TerminalTab>,
    focused_terminal: usize, // 0 = left terminal, 1 = right terminal
}

impl AudioToolkitApp {
    fn new(config: AppConfig) -> Self {
        let tabs = config.tabs.into_iter().map(TerminalTab::new).collect();

        Self {
            tabs,
            focused_terminal: 0, // Start with left terminal focused
        }
    }

    fn render_row(row: &[TerminalCell], ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.spacing_mut().item_spacing.y = 0.0;
            for cell in row {
                let mut rich_text = egui::RichText::new(cell.character.to_string())
                    .font(egui::FontId::monospace(12.0))
                    .color(cell.color);
                
                if cell.bold {
                    rich_text = rich_text.strong();
                }
                
                ui.add(egui::Label::new(rich_text).wrap(false).selectable(false));
            }
        });
    }

    fn render_terminal_buffer(ui: &mut egui::Ui, buffer: &[Vec<TerminalCell>]) {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.spacing_mut().item_spacing.x = 0.0;
        
        for row in buffer {
            Self::render_row(row, ui);
        }
    }

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

        // Split-Screen Layout: Terminal 1 (Left) and Terminal 2 (Right)
        egui::SidePanel::left("terminal_1")
            .resizable(true)
            .default_width(ctx.screen_rect().width() * 0.5)
            .show(ctx, |ui| {
                if !self.tabs.is_empty() {
                    let tab = &mut self.tabs[0]; // Terminal 1
                    let is_focused = self.focused_terminal == 0;
                    let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                    let title_color = if is_focused {
                        egui::Color32::LIGHT_BLUE
                    } else {
                        egui::Color32::GRAY
                    };

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} 🖥️ {}", focus_indicator, tab.title))
                                .color(title_color)
                                .strong(),
                        );
                        if !is_focused {
                            ui.label(
                                egui::RichText::new("(Press Tab to focus)")
                                    .color(egui::Color32::DARK_GRAY)
                                    .italics(),
                            );
                        }
                    });
                    ui.separator();

                    // Terminal 1 Output Area with ANSI Color Support
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                            // Render terminal emulator buffer
                            Self::render_terminal_buffer(ui, &tab.terminal_emulator.buffer);
                        });

                    // Terminal 1 Input Area
                    ui.separator();
                    let input_response = ui
                        .horizontal(|ui| {
                            ui.label("$");
                            ui.add(
                                egui::TextEdit::singleline(&mut tab.input)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            )
                        })
                        .inner;

                    // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                    if is_focused {
                        Self::handle_terminal_key_input(ui, &mut tab.pty_writer);
                    }

                    // Handle text input and Enter key
                    if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let mut input_with_newline = tab.input.clone();
                        input_with_newline.push('\n');

                        if let Some(ref mut writer) = tab.pty_writer {
                            if let Err(e) = writer.write_all(input_with_newline.as_bytes()) {
                                eprintln!("Error writing to PTY: {}", e);
                            }
                        } else {
                            eprintln!("No PTY writer available");
                        }

                        tab.input.clear();
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs.len() > 1 {
                let tab = &mut self.tabs[1]; // Terminal 2
                let is_focused = self.focused_terminal == 1;
                let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                let title_color = if is_focused {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::GRAY
                };

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} 🖥️ {}", focus_indicator, tab.title))
                            .color(title_color)
                            .strong(),
                    );
                    if !is_focused {
                        ui.label(
                            egui::RichText::new("(Press Tab to focus)")
                                .color(egui::Color32::DARK_GRAY)
                                .italics(),
                        );
                    }
                });
                ui.separator();

                // Terminal 2 Output Area with ANSI Color Support
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(ui.available_height() - 60.0)
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                        // Render terminal emulator buffer
                        Self::render_terminal_buffer(ui, &tab.terminal_emulator.buffer);
                    });

                // Terminal 2 Input Area
                ui.separator();
                let input_response = ui
                    .horizontal(|ui| {
                        ui.label("$");
                        ui.add(
                            egui::TextEdit::singleline(&mut tab.input)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        )
                    })
                    .inner;

                // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                if is_focused {
                    Self::handle_terminal_key_input(ui, &mut tab.pty_writer);
                }

                // Handle text input and Enter key
                if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut input_with_newline = tab.input.clone();
                    input_with_newline.push('\n');

                    if let Some(ref mut writer) = tab.pty_writer {
                        if let Err(e) = writer.write_all(input_with_newline.as_bytes()) {
                            eprintln!("Error writing to PTY: {}", e);
                        }
                    } else {
                        eprintln!("No PTY writer available");
                    }

                    tab.input.clear();
                    input_response.request_focus();
                }
            }
        });
    }
}

// ... (rest of the code remains the same)
