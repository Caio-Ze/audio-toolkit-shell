// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, Frame};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

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
    output: String,
    input: String,
    needs_restart: bool,
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
            // Absolute path - run through bash
            println!("Executing command: bash -c {}", &config.command);
            let mut cmd = CommandBuilder::new("bash");
            cmd.arg("-c");
            cmd.arg(&config.command);
            cmd
        } else {
            // Command name - run in shell
            println!("Executing bash shell");
            CommandBuilder::new("bash")
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
            input: String::new(),
            needs_restart: false,
        }
    }

    fn update_output(&mut self) {
        loop {
            match self.output_rx.try_recv() {
                Ok(data) => {
                    // Strip ANSI escape codes for now (basic implementation)
                    let cleaned = Self::strip_ansi_codes(&data);
                    self.output.push_str(&cleaned);
                    
                    // Check for success patterns if auto-restart is enabled
                    if self.config.auto_restart_on_success {
                        for pattern in &self.config.success_patterns {
                            if cleaned.contains(pattern) {
                                self.needs_restart = true;
                                break;
                            }
                        }
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }
    
    fn restart(&mut self) {
        if !self.needs_restart {
            return;
        }
        
        // Clear output and reset state
        self.output.clear();
        self.needs_restart = false;
        
        // Create new PTY and restart the command
        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                ..Default::default()
            })
            .unwrap();

        let cmd = if self.config.command.contains('/') {
            // Absolute path - run through bash
            let mut cmd = CommandBuilder::new("bash");
            cmd.arg("-c");
            cmd.arg(&self.config.command);
            cmd
        } else {
            // Command name - run in shell
            CommandBuilder::new("bash")
        };
        match pty_pair.slave.spawn_command(cmd) {
            Ok(_child) => println!("Restart command spawned successfully"),
            Err(e) => {
                eprintln!("Failed to spawn restart command: {}", e);
                return; // Don't continue if command spawn fails
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

        let writer = pty_pair.master.take_writer().ok();
        
        // Update the tab with new PTY components
        self.pty_master = pty_pair.master;
        self.pty_writer = writer;
        self.output_rx = output_rx;
    }
    
    fn strip_ansi_codes(input: &str) -> String {
        // Basic ANSI escape sequence removal
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' || ch == '\u{1b}' {
                // Skip escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Skip until we find a letter (end of escape sequence)
                    while let Some(next_ch) = chars.next() {
                        if next_ch.is_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }
        result
    }
}

struct AudioToolkitApp {
    active_tab: usize,
    tabs: Vec<TerminalTab>,
}

impl AudioToolkitApp {
    fn new(config: AppConfig) -> Self {
        let tabs = config.tabs.into_iter()
            .map(|tab_config| TerminalTab::new(tab_config))
            .collect();
        
        Self {
            active_tab: 0,
            tabs,
        }
    }
}

impl App for AudioToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Request a repaint to ensure we check for new PTY data
        ctx.request_repaint();

        // Update output for all tabs and handle restarts
        for tab in &mut self.tabs {
            tab.update_output();
            if tab.needs_restart {
                tab.restart();
            }
        }

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(self.active_tab == i, &tab.title).clicked() {
                        self.active_tab = i;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = &mut self.tabs[self.active_tab];
            ui.heading(&tab.title);
            ui.separator();

            // Terminal Output Area
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                ui.add(
                    egui::TextEdit::multiline(&mut tab.output.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .interactive(false)
                );
            });

            // Terminal Input Area
            ui.separator();
            let input_response = ui.horizontal(|ui| {
                ui.label("$");
                ui.add(
                    egui::TextEdit::singleline(&mut tab.input)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                )
            }).inner;
            
            if input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Append newline because shells expect it
                let mut input_with_newline = tab.input.clone();
                input_with_newline.push('\n');
                
                // Send input to PTY
                if let Some(ref mut writer) = tab.pty_writer {
                    if let Err(e) = writer.write_all(input_with_newline.as_bytes()) {
                        eprintln!("Error writing to PTY: {}", e);
                    }
                } else {
                    eprintln!("No PTY writer available");
                }
                
                // Clear the input field
                tab.input.clear();
                // Refocus the input field
                input_response.request_focus();
            }
        });
    }
}
