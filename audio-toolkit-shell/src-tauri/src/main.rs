// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, Frame};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

#[derive(Clone, Debug)]
struct ColoredText {
    text: String,
    color: egui::Color32,
    bold: bool,
}

#[derive(Clone, Debug)]
struct TerminalState {
    current_color: egui::Color32,
    bold: bool,
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
        Ok(content) => {
            toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Error parsing config.toml: {}", e);
                default_config()
            })
        }
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
        viewport: egui::ViewportBuilder::default().with_inner_size([config.app.window_width, config.app.window_height]),
        ..
        Default::default()
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
    output: String,  // Keep for pattern matching
    colored_output: Vec<ColoredText>,  // New: store colored text segments
    input: String,
    needs_restart: bool,
    startup_time: std::time::Instant,
    pattern_matches: u32,
    terminal_state: TerminalState,  // Track current color state
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
            colored_output: Vec::new(),
            input: String::new(),
            needs_restart: false,
            startup_time: std::time::Instant::now(),
            pattern_matches: 0,
            terminal_state: TerminalState {
                current_color: egui::Color32::WHITE,
                bold: false,
            },
        }
    }

    fn update_output(&mut self) {
        loop {
            match self.output_rx.try_recv() {
                Ok(data) => {
                    // Parse ANSI colors and get both colored segments and plain text
                    let (colored_segments, plain_text) = Self::parse_ansi_text(&data, &mut self.terminal_state);
                    
                    // Add colored segments to colored output
                    self.colored_output.extend(colored_segments);
                    
                    // Add plain text to output for pattern matching
                    self.output.push_str(&plain_text);
                    
                    // Check for success patterns if auto-restart is enabled
                    if self.config.auto_restart_on_success {
                        // Only check patterns after 5 seconds to avoid startup menu detection
                        let elapsed = self.startup_time.elapsed();
                        if elapsed.as_secs() >= 5 {
                            for pattern in &self.config.success_patterns {
                                if plain_text.contains(pattern) {
                                    self.pattern_matches += 1;
                                    println!("[PATTERN] Found '{}' in tab '{}' (match #{}/2)", pattern, self.title, self.pattern_matches);
                                    
                                    // Require 2 pattern matches to avoid false positives
                                    if self.pattern_matches >= 2 {
                                        println!("[PATTERN] Triggering restart for tab '{}'", self.title);
                                        self.needs_restart = true;
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Still in startup period - don't check patterns yet
                            if !self.config.success_patterns.is_empty() {
                                println!("[PATTERN] Startup period for '{}' - {} seconds remaining", 
                                    self.title, 5 - elapsed.as_secs());
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
        self.colored_output.clear();
        self.needs_restart = false;
        self.startup_time = std::time::Instant::now();
        self.pattern_matches = 0;
        // Reset terminal color state
        self.terminal_state = TerminalState {
            current_color: egui::Color32::WHITE,
            bold: false,
        };
        
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
                self.output.push_str(&format!("\n[ERROR] Failed to restart: {}\n", e));
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
                self.output.push_str(&format!("\n✅ Script executed successfully\n\n", ));
            },
            Err(e) => {
                eprintln!("[RESTART] Failed to spawn command for {}: {}", self.title, e);
                self.output.push_str(&format!("\n[ERROR] Failed to restart command: {}\n", e));
                return;
            }
        }

        // Set up new PTY reader with better error handling
        let mut reader = match pty_pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("[RESTART] Failed to clone PTY reader for {}: {}", self.title, e);
                self.output.push_str(&format!("\n[ERROR] Failed to setup PTY reader: {}\n", e));
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
                eprintln!("[RESTART] Failed to get PTY writer for {}: {}", self.title, e);
                None
            }
        };
        
        // Update the tab with new PTY components
        self.pty_master = pty_pair.master;
        self.pty_writer = writer;
        self.output_rx = output_rx;
        
        println!("[RESTART] Successfully restarted tab: {}", self.title);
    }
    
    fn parse_ansi_text(input: &str, terminal_state: &mut TerminalState) -> (Vec<ColoredText>, String) {
        let mut colored_segments = Vec::new();
        let mut plain_text = String::new(); // For pattern matching
        let mut chars = input.chars().peekable();
        let mut current_text = String::new();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' || ch == '\u{1b}' {
                // Save current text segment before processing escape sequence
                if !current_text.is_empty() {
                    colored_segments.push(ColoredText {
                        text: current_text.clone(),
                        color: terminal_state.current_color,
                        bold: terminal_state.bold,
                    });
                    plain_text.push_str(&current_text);
                    current_text.clear();
                }
                
                // Process ANSI escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let mut code = String::new();
                    
                    // Collect the escape sequence code
                    while let Some(next_ch) = chars.next() {
                        if next_ch.is_alphabetic() {
                            // Process the collected code
                            Self::apply_ansi_code(&code, terminal_state);
                            break;
                        } else {
                            code.push(next_ch);
                        }
                    }
                }
            } else {
                current_text.push(ch);
            }
        }
        
        // Add any remaining text
        if !current_text.is_empty() {
            colored_segments.push(ColoredText {
                text: current_text.clone(),
                color: terminal_state.current_color,
                bold: terminal_state.bold,
            });
            plain_text.push_str(&current_text);
        }
        
        (colored_segments, plain_text)
    }
    
    fn apply_ansi_code(code: &str, terminal_state: &mut TerminalState) {
        // Parse ANSI color codes
        let codes: Vec<&str> = code.split(';').collect();
        
        let mut i = 0;
        while i < codes.len() {
            match codes[i] {
                "0" => {
                    // Reset all attributes
                    terminal_state.current_color = egui::Color32::WHITE;
                    terminal_state.bold = false;
                }
                "1" => terminal_state.bold = true,
                "22" => terminal_state.bold = false,
                
                // 256-color foreground: ESC[38;5;n
                "38" if i + 2 < codes.len() && codes[i + 1] == "5" => {
                    if let Ok(color_index) = codes[i + 2].parse::<u8>() {
                        terminal_state.current_color = Self::ansi_256_to_rgb(color_index);
                    }
                    i += 2; // Skip the next two codes (5 and color_index)
                }
                
                // 256-color background: ESC[48;5;n (we'll ignore background for now)
                "48" if i + 2 < codes.len() && codes[i + 1] == "5" => {
                    i += 2; // Skip background colors for now
                }
                
                // Basic foreground colors (30-37)
                "30" => terminal_state.current_color = egui::Color32::BLACK,
                "31" => terminal_state.current_color = egui::Color32::from_rgb(205, 49, 49), // Red
                "32" => terminal_state.current_color = egui::Color32::from_rgb(13, 188, 121), // Green
                "33" => terminal_state.current_color = egui::Color32::from_rgb(229, 229, 16), // Yellow
                "34" => terminal_state.current_color = egui::Color32::from_rgb(36, 114, 200), // Blue
                "35" => terminal_state.current_color = egui::Color32::from_rgb(188, 63, 188), // Magenta
                "36" => terminal_state.current_color = egui::Color32::from_rgb(17, 168, 205), // Cyan
                "37" => terminal_state.current_color = egui::Color32::WHITE,
                "39" => terminal_state.current_color = egui::Color32::WHITE, // Default foreground
                
                // Bright foreground colors (90-97)
                "90" => terminal_state.current_color = egui::Color32::from_rgb(102, 102, 102), // Bright Black
                "91" => terminal_state.current_color = egui::Color32::from_rgb(241, 76, 76), // Bright Red
                "92" => terminal_state.current_color = egui::Color32::from_rgb(35, 209, 139), // Bright Green
                "93" => terminal_state.current_color = egui::Color32::from_rgb(245, 245, 67), // Bright Yellow
                "94" => terminal_state.current_color = egui::Color32::from_rgb(59, 142, 234), // Bright Blue
                "95" => terminal_state.current_color = egui::Color32::from_rgb(214, 112, 214), // Bright Magenta
                "96" => terminal_state.current_color = egui::Color32::from_rgb(41, 184, 219), // Bright Cyan
                "97" => terminal_state.current_color = egui::Color32::WHITE, // Bright White
                
                _ => {} // Ignore unknown codes
            }
            i += 1;
        }
    }
    
    fn ansi_256_to_rgb(color_index: u8) -> egui::Color32 {
        match color_index {
            // Standard colors (0-15)
            0 => egui::Color32::BLACK,
            1 => egui::Color32::from_rgb(128, 0, 0),     // Dark Red
            2 => egui::Color32::from_rgb(0, 128, 0),     // Dark Green
            3 => egui::Color32::from_rgb(128, 128, 0),   // Dark Yellow
            4 => egui::Color32::from_rgb(0, 0, 128),     // Dark Blue
            5 => egui::Color32::from_rgb(128, 0, 128),   // Dark Magenta
            6 => egui::Color32::from_rgb(0, 128, 128),   // Dark Cyan
            7 => egui::Color32::from_rgb(192, 192, 192), // Light Gray
            8 => egui::Color32::from_rgb(128, 128, 128), // Dark Gray
            9 => egui::Color32::from_rgb(255, 0, 0),     // Bright Red
            10 => egui::Color32::from_rgb(0, 255, 0),    // Bright Green
            11 => egui::Color32::from_rgb(255, 255, 0),  // Bright Yellow
            12 => egui::Color32::from_rgb(0, 0, 255),    // Bright Blue
            13 => egui::Color32::from_rgb(255, 0, 255),  // Bright Magenta
            14 => egui::Color32::from_rgb(0, 255, 255),  // Bright Cyan
            15 => egui::Color32::WHITE,                  // Bright White
            
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
}

struct AudioToolkitApp {
    tabs: Vec<TerminalTab>,
    active_tab: usize,
    focused_terminal: usize, // 0 = left terminal, 1 = right terminal
}

impl AudioToolkitApp {
    fn new(config: AppConfig) -> Self {
        let tabs = config.tabs.into_iter()
            .map(|tab_config| TerminalTab::new(tab_config))
            .collect();
        
        Self {
            tabs,
            active_tab: 0,
            focused_terminal: 0, // Start with left terminal focused
        }
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
                println!("Focus switched to terminal: {}", if self.focused_terminal == 0 { "Left" } else { "Right" });
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
        egui::SidePanel::left("terminal_1").resizable(true).default_width(ctx.screen_rect().width() * 0.5).show(ctx, |ui| {
            if self.tabs.len() > 0 {
                let tab = &mut self.tabs[0]; // Terminal 1
                let is_focused = self.focused_terminal == 0;
                let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                let title_color = if is_focused { egui::Color32::LIGHT_BLUE } else { egui::Color32::GRAY };
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&format!("{} 🖥️ {}", focus_indicator, tab.title))
                        .color(title_color)
                        .strong());
                    if !is_focused {
                        ui.label(egui::RichText::new("(Press Tab to focus)")
                            .color(egui::Color32::DARK_GRAY)
                            .italics());
                    }
                });
                ui.separator();

                // Terminal 1 Output Area with ANSI Color Support
                egui::ScrollArea::vertical().stick_to_bottom(true).max_height(ui.available_height() - 60.0).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    
                    // Render colored text segments
                    ui.horizontal_wrapped(|ui| {
                        for colored_text in &tab.colored_output {
                            let mut rich_text = egui::RichText::new(&colored_text.text)
                                .font(egui::FontId::monospace(12.0))
                                .color(colored_text.color);
                            
                            if colored_text.bold {
                                rich_text = rich_text.strong();
                            }
                            
                            ui.label(rich_text);
                        }
                    });
                });

                // Terminal 1 Input Area
                ui.separator();
                let input_response = ui.horizontal(|ui| {
                    ui.label("$");
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                    )
                }).inner;
                
                // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                if is_focused {
                    ui.input(|i| {
                        // Handle arrow keys for navigation
                        if i.key_pressed(egui::Key::ArrowUp) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[A"); // Up arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowDown) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[B"); // Down arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowLeft) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[D"); // Left arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowRight) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[C"); // Right arrow ANSI sequence
                            }
                        }
                        // Handle other special keys
                        if i.key_pressed(egui::Key::Escape) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b"); // Escape key
                            }
                        }
                        if i.key_pressed(egui::Key::Space) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b" "); // Space key
                            }
                        }
                    });
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
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs.len() > 1 {
                let tab = &mut self.tabs[1]; // Terminal 2
                let is_focused = self.focused_terminal == 1;
                let focus_indicator = if is_focused { "🔵" } else { "⚪" };
                let title_color = if is_focused { egui::Color32::LIGHT_BLUE } else { egui::Color32::GRAY };
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&format!("{} 🖥️ {}", focus_indicator, tab.title))
                        .color(title_color)
                        .strong());
                    if !is_focused {
                        ui.label(egui::RichText::new("(Press Tab to focus)")
                            .color(egui::Color32::DARK_GRAY)
                            .italics());
                    }
                });
                ui.separator();

                // Terminal 2 Output Area with ANSI Color Support
                egui::ScrollArea::vertical().stick_to_bottom(true).max_height(ui.available_height() - 60.0).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    
                    // Render colored text segments
                    ui.horizontal_wrapped(|ui| {
                        for colored_text in &tab.colored_output {
                            let mut rich_text = egui::RichText::new(&colored_text.text)
                                .font(egui::FontId::monospace(12.0))
                                .color(colored_text.color);
                            
                            if colored_text.bold {
                                rich_text = rich_text.strong();
                            }
                            
                            ui.label(rich_text);
                        }
                    });
                });

                // Terminal 2 Input Area
                ui.separator();
                let input_response = ui.horizontal(|ui| {
                    ui.label("$");
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                    )
                }).inner;
                
                // Handle comprehensive key input for terminal navigation (only if this terminal is focused)
                if is_focused {
                    ui.input(|i| {
                        // Handle arrow keys for navigation
                        if i.key_pressed(egui::Key::ArrowUp) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[A"); // Up arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowDown) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[B"); // Down arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowLeft) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[D"); // Left arrow ANSI sequence
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowRight) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b[C"); // Right arrow ANSI sequence
                            }
                        }
                        // Handle other special keys
                        if i.key_pressed(egui::Key::Escape) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b"\x1b"); // Escape key
                            }
                        }
                        if i.key_pressed(egui::Key::Space) {
                            if let Some(ref mut writer) = tab.pty_writer {
                                let _ = writer.write_all(b" "); // Space key
                            }
                        }
                    });
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
