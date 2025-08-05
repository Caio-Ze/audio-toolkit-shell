// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, Frame};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use unicode_width::UnicodeWidthChar;

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
        // Defensive programming: ensure buffer dimensions are valid
        if self.rows == 0 || self.cols == 0 {
            return;
        }
        
        // Clamp cursor position to valid bounds
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
        
        // Additional validation to ensure cursor position is within buffer bounds
        self.validate_cursor_position();
    }
    
    fn validate_cursor_position(&mut self) {
        // Ensure cursor position is always valid
        if self.cursor_row >= self.rows {
            self.cursor_row = if self.rows > 0 { self.rows - 1 } else { 0 };
        }
        if self.cursor_col >= self.cols {
            self.cursor_col = if self.cols > 0 { self.cols - 1 } else { 0 };
        }
        
        // Ensure buffer has the expected dimensions
        if self.buffer.len() != self.rows {
            // Reconstruct buffer if dimensions are inconsistent
            self.buffer = vec![vec![TerminalCell::default(); self.cols]; self.rows];
            self.cursor_row = 0;
            self.cursor_col = 0;
        }
        
        // Ensure each row has the correct width
        for row in &mut self.buffer {
            if row.len() != self.cols {
                row.resize(self.cols, TerminalCell::default());
            }
        }
    }

    fn get_char_width(ch: char) -> usize {
        // Special handling for emojis that might not be correctly detected by unicode-width
        if Self::is_emoji_char(ch) {
            return 2; // Force all emojis to width 2 for consistent alignment
        }
        
        // Handle unicode-width returning None gracefully
        match ch.width() {
            Some(width) => {
                // Clamp width to reasonable bounds (0-2 for terminal display)
                width.min(2)
            }
            None => {
                // For control characters and other special cases, default to 1
                // This ensures we never have zero-width characters that could break layout
                1
            }
        }
    }
    
    fn is_emoji_char(ch: char) -> bool {
        let code_point = ch as u32;
        
        match code_point {
            // Existing emoji ranges (working correctly)
            0x1F600..=0x1F64F => true, // Emoticons
            0x1F300..=0x1F5FF => true, // Miscellaneous Symbols and Pictographs
            0x1F680..=0x1F6FF => true, // Transport and Map Symbols
            0x1F910..=0x1F96B => true, // Additional Emoticons
            0x1F900..=0x1F9FF => true, // Supplemental Symbols and Pictographs
            
            // ADD THESE RANGES - This is what's missing!
            0x2700..=0x27BF => true, // Dingbats (✂️✅✈️✉️✊ etc.)
            0x2600..=0x26FF => true, // Miscellaneous Symbols (☀️⭐✨⚡❄️ etc.)
            
            // Additional ranges that may need width-2 treatment
            0x2B50..=0x2B55 => true, // Additional symbols (⭐⭑⭒⭓⭔⭕)
            0x1F100..=0x1F1FF => true, // Enclosed Alphanumeric Supplement
            0x1F200..=0x1F2FF => true, // Enclosed Ideographic Supplement
            
            _ => false,
        }
    }

    fn write_char(&mut self, ch: char) {
        // Validate cursor position before any operations
        self.validate_cursor_position();
        
        // Handle malformed input gracefully
        if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
            // Skip most control characters to maintain buffer integrity
            return;
        }
        
        // Calculate character width with error handling
        let width = Self::get_char_width(ch);
        
        // Additional bounds checking for width
        if width == 0 || width > 2 {
            // Invalid width, treat as normal character
            return;
        }
        
        // Check if character would exceed line boundary
        if self.cursor_col + width > self.cols {
            // Character doesn't fit on current line, wrap to next line
            self.handle_newline();
        }
        
        // Validate cursor position after potential line wrap
        self.validate_cursor_position();
        
        // Bounds checking for all buffer operations
        if self.cursor_row >= self.rows || self.cursor_col >= self.cols {
            return; // Cannot write beyond buffer bounds
        }
        
        // Additional safety check for wide characters
        if width == 2 && self.cursor_col + 1 >= self.cols {
            // Wide character would exceed bounds, wrap to next line
            self.handle_newline();
            self.validate_cursor_position();
            
            // Check again after wrapping
            if self.cursor_row >= self.rows || self.cursor_col + 1 >= self.cols {
                return;
            }
        }
        
        // Now safe to write the character
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            // Determine color: use white/gray for box drawing characters, current color for text
            let char_color = if self.is_box_drawing_char(ch) {
                egui::Color32::from_rgb(128, 128, 128) // Gray for box drawing
            } else {
                self.current_color
            };
            
            // Write the character to the current position with bounds checking
            if let Some(row) = self.buffer.get_mut(self.cursor_row) {
                if let Some(cell) = row.get_mut(self.cursor_col) {
                    *cell = TerminalCell {
                        character: ch,
                        color: char_color,
                        bold: self.bold,
                    };
                }
            }

            // Create placeholder cell for wide characters (width == 2) with bounds checking
            if width == 2 && self.cursor_col + 1 < self.cols {
                if let Some(row) = self.buffer.get_mut(self.cursor_row) {
                    if let Some(cell) = row.get_mut(self.cursor_col + 1) {
                        *cell = TerminalCell {
                            character: '\0',
                            color: egui::Color32::TRANSPARENT,
                            bold: false,
                        };
                    }
                }
            }

            // Advance cursor by character width with overflow protection
            let new_col = self.cursor_col.saturating_add(width);
            self.cursor_col = new_col.min(self.cols);
            
            // Handle cursor wrapping with width-aware logic
            if self.cursor_col >= self.cols {
                self.handle_newline();
            }
        }
        
        // Final validation to ensure cursor remains in valid state
        self.validate_cursor_position();
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
        
        // Bounds checking before incrementing row
        if self.cursor_row < usize::MAX {
            self.cursor_row += 1;
        }
        
        // Handle scrolling with error protection
        if self.cursor_row >= self.rows && self.rows > 0 {
            // Scroll up with bounds checking
            if !self.buffer.is_empty() {
                self.buffer.remove(0);
                self.buffer.push(vec![TerminalCell::default(); self.cols]);
                self.cursor_row = self.rows - 1;
            } else {
                // Buffer is empty, reset to safe state
                self.buffer = vec![vec![TerminalCell::default(); self.cols]; self.rows];
                self.cursor_row = 0;
            }
        }
        
        // Validate cursor position after newline
        self.validate_cursor_position();
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

        // Safe extraction of command character
        let cmd = match sequence.chars().last() {
            Some(c) => c,
            None => return, // Empty sequence, nothing to do
        };
        
        // Safe parameter parsing with bounds checking
        if sequence.len() == 0 {
            return;
        }
        
        let param_str = if sequence.len() > 1 {
            &sequence[..sequence.len() - 1]
        } else {
            ""
        };
        
        let params: Vec<&str> = param_str.split(';').collect();

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
                // Cursor up with bounds checking
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .min(1000); // Limit to reasonable values to prevent overflow
                self.cursor_row = self.cursor_row.saturating_sub(count);
                self.validate_cursor_position();
            }
            'B' => {
                // Cursor down with bounds checking
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .min(1000); // Limit to reasonable values
                if self.rows > 0 {
                    self.cursor_row = (self.cursor_row.saturating_add(count)).min(self.rows - 1);
                }
                self.validate_cursor_position();
            }
            'C' => {
                // Cursor forward with bounds checking
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .min(1000); // Limit to reasonable values
                if self.cols > 0 {
                    self.cursor_col = (self.cursor_col.saturating_add(count)).min(self.cols - 1);
                }
                self.validate_cursor_position();
            }
            'D' => {
                // Cursor backward with bounds checking
                let count = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
                    .min(1000); // Limit to reasonable values
                self.cursor_col = self.cursor_col.saturating_sub(count);
                self.validate_cursor_position();
            }
            'J' => {
                // Clear screen with bounds checking
                let mode = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                
                // Validate cursor position before clearing operations
                self.validate_cursor_position();
                
                match mode {
                    0 => {
                        // Clear from cursor to end of screen with bounds checking
                        if self.cursor_row < self.rows {
                            // Clear current line from cursor with bounds checking
                            if let Some(row) = self.buffer.get_mut(self.cursor_row) {
                                for col in self.cursor_col..self.cols.min(row.len()) {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                            
                            // Clear all lines below with bounds checking
                            for row_idx in (self.cursor_row + 1)..self.rows.min(self.buffer.len()) {
                                if let Some(row) = self.buffer.get_mut(row_idx) {
                                    for col in 0..self.cols.min(row.len()) {
                                        if let Some(cell) = row.get_mut(col) {
                                            *cell = TerminalCell::default();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    1 => {
                        // Clear from beginning of screen to cursor with bounds checking
                        for row_idx in 0..self.cursor_row.min(self.buffer.len()) {
                            if let Some(row) = self.buffer.get_mut(row_idx) {
                                for col in 0..self.cols.min(row.len()) {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                        }
                        
                        // Clear current line to cursor with bounds checking
                        if self.cursor_row < self.rows && self.cursor_row < self.buffer.len() {
                            if let Some(row) = self.buffer.get_mut(self.cursor_row) {
                                let end_col = (self.cursor_col + 1).min(self.cols).min(row.len());
                                for col in 0..end_col {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.clear_screen();
                    }
                    _ => {
                        // Invalid mode, ignore
                    }
                }
            }
            'K' => {
                // Clear line with bounds checking
                let mode = params
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                
                // Validate cursor position and buffer bounds
                self.validate_cursor_position();
                
                if self.cursor_row < self.rows && self.cursor_row < self.buffer.len() {
                    if let Some(row) = self.buffer.get_mut(self.cursor_row) {
                        match mode {
                            0 => {
                                // Clear from cursor to end of line with bounds checking
                                for col in self.cursor_col..self.cols.min(row.len()) {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                            1 => {
                                // Clear from beginning of line to cursor with bounds checking
                                let end_col = (self.cursor_col + 1).min(self.cols).min(row.len());
                                for col in 0..end_col {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                            2 => {
                                // Clear entire line with bounds checking
                                for col in 0..self.cols.min(row.len()) {
                                    if let Some(cell) = row.get_mut(col) {
                                        *cell = TerminalCell::default();
                                    }
                                }
                            }
                            _ => {
                                // Invalid mode, ignore
                            }
                        }
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
            // Reset to safe defaults
            self.current_color = egui::Color32::WHITE;
            self.bold = false;
            return;
        }

        let mut i = 0;
        while i < params.len() {
            // Bounds checking to prevent infinite loops
            if i >= 100 {
                break; // Prevent excessive parameter processing
            }
            
            // Safe parameter access
            let param = match params.get(i) {
                Some(p) => *p,
                None => break,
            };
            
            match param {
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
                "38" => {
                    // 256-color foreground with bounds checking
                    if i + 2 < params.len() && 
                       params.get(i + 1) == Some(&"5") &&
                       i + 2 < 100 { // Additional bounds check
                        if let Some(color_param) = params.get(i + 2) {
                            if let Ok(color_index) = color_param.parse::<u8>() {
                                self.current_color = ansi_256_to_rgb(color_index);
                            }
                        }
                        i += 2; // Skip the next two parameters
                    }
                }
                _ => {
                    // Unknown parameter - ignore safely
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
                // Skip placeholder characters
                if cell.character == '\0' {
                    continue;
                }
                
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_width_ascii() {
        // ASCII characters should have width 1
        assert_eq!(TerminalEmulator::get_char_width('a'), 1);
        assert_eq!(TerminalEmulator::get_char_width('Z'), 1);
        assert_eq!(TerminalEmulator::get_char_width('0'), 1);
        assert_eq!(TerminalEmulator::get_char_width(' '), 1);
        assert_eq!(TerminalEmulator::get_char_width('!'), 1);
    }

    #[test]
    fn test_char_width_emojis() {
        // Emojis should have width 2 (some may be 1 depending on unicode version)
        // Test with emojis that are more reliably width 2
        assert_eq!(TerminalEmulator::get_char_width('😀'), 2);
        assert_eq!(TerminalEmulator::get_char_width('🎵'), 2);
        // Some emojis may have width 1, so let's test that the function works
        let fire_width = TerminalEmulator::get_char_width('🔥');
        assert!(fire_width == 1 || fire_width == 2);
        let heart_width = TerminalEmulator::get_char_width('❤');
        assert!(heart_width == 1 || heart_width == 2);
    }

    #[test]
    fn test_char_width_box_drawing() {
        // Box-drawing characters should have width 1
        assert_eq!(TerminalEmulator::get_char_width('─'), 1);
        assert_eq!(TerminalEmulator::get_char_width('│'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┌'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┐'), 1);
        assert_eq!(TerminalEmulator::get_char_width('└'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┘'), 1);
        assert_eq!(TerminalEmulator::get_char_width('├'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┤'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┬'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┴'), 1);
        assert_eq!(TerminalEmulator::get_char_width('┼'), 1);
    }

    #[test]
    fn test_char_width_cjk() {
        // CJK characters should have width 2
        assert_eq!(TerminalEmulator::get_char_width('中'), 2);
        assert_eq!(TerminalEmulator::get_char_width('文'), 2);
        assert_eq!(TerminalEmulator::get_char_width('あ'), 2);
        assert_eq!(TerminalEmulator::get_char_width('한'), 2);
    }

    #[test]
    fn test_char_width_control_chars() {
        // Control characters should have width 0 or 1 (handled by unwrap_or(1))
        assert_eq!(TerminalEmulator::get_char_width('\t'), 1);
        assert_eq!(TerminalEmulator::get_char_width('\n'), 1);
        assert_eq!(TerminalEmulator::get_char_width('\r'), 1);
    }

    #[test]
    fn test_cursor_advancement_ascii() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write ASCII character should advance cursor by 1
        terminal.write_char('a');
        assert_eq!(terminal.cursor_col, 1);
        assert_eq!(terminal.cursor_row, 0);
        
        // Write another ASCII character
        terminal.write_char('b');
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 0);
    }

    #[test]
    fn test_cursor_advancement_wide_chars() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write a wide character (CJK) should advance cursor by 2
        terminal.write_char('中');
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 0);
        
        // Write another wide character
        terminal.write_char('文');
        assert_eq!(terminal.cursor_col, 4);
        assert_eq!(terminal.cursor_row, 0);
    }

    #[test]
    fn test_cursor_advancement_mixed_chars() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Mix ASCII and wide characters
        terminal.write_char('a');  // width 1, cursor at 1
        terminal.write_char('中'); // width 2, cursor at 3
        terminal.write_char('b');  // width 1, cursor at 4
        
        assert_eq!(terminal.cursor_col, 4);
        assert_eq!(terminal.cursor_row, 0);
    }

    #[test]
    fn test_boundary_checking_wide_char() {
        let mut terminal = TerminalEmulator::new(5, 5);
        
        // Position cursor near end of line
        terminal.cursor_col = 4;
        
        // Try to write a wide character that would exceed boundary
        // Should wrap to next line and write the character there
        terminal.write_char('中'); // width 2, should wrap to next line
        
        // Character should be written on the next line at position 0
        assert_eq!(terminal.buffer[1][0].character, '中');
        assert_eq!(terminal.cursor_col, 2); // Advanced by width 2
        assert_eq!(terminal.cursor_row, 1); // Moved to next row
    }

    #[test]
    fn test_line_wrapping_wide_char() {
        let mut terminal = TerminalEmulator::new(5, 6);
        
        // Fill line almost to the end, leaving only 1 position
        terminal.write_char('a'); // pos 0
        terminal.write_char('b'); // pos 1
        terminal.write_char('c'); // pos 2
        terminal.write_char('d'); // pos 3
        terminal.write_char('e'); // pos 4
        // cursor now at position 5, only 1 position left
        
        // Write a wide character that should wrap
        terminal.write_char('中'); // width 2, should wrap to next line
        
        // Verify the wide character is on the next line
        assert_eq!(terminal.buffer[1][0].character, '中');
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
        
        // Verify previous line content is intact
        assert_eq!(terminal.buffer[0][0].character, 'a');
        assert_eq!(terminal.buffer[0][1].character, 'b');
        assert_eq!(terminal.buffer[0][2].character, 'c');
        assert_eq!(terminal.buffer[0][3].character, 'd');
        assert_eq!(terminal.buffer[0][4].character, 'e');
    }

    #[test]
    fn test_no_split_wide_chars() {
        let mut terminal = TerminalEmulator::new(5, 5);
        
        // Position cursor at the last position of a line
        terminal.cursor_col = 4;
        
        // Write a wide character - it should wrap entirely to next line
        terminal.write_char('文'); // width 2
        
        // Character should be on next line, not split
        assert_eq!(terminal.buffer[0][4].character, ' '); // Original position unchanged
        assert_eq!(terminal.buffer[1][0].character, '文'); // Character on next line
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
    }

    #[test]
    fn test_consecutive_wide_chars_wrapping() {
        let mut terminal = TerminalEmulator::new(5, 6);
        
        // Write characters to fill most of the line
        terminal.write_char('a'); // pos 0
        terminal.write_char('b'); // pos 1
        terminal.write_char('c'); // pos 2
        terminal.write_char('d'); // pos 3
        // cursor at pos 4, 2 positions left
        
        // Write two consecutive wide characters
        terminal.write_char('中'); // width 2, fits at pos 4-5
        terminal.write_char('文'); // width 2, should wrap to next line
        
        // Verify first wide char is on first line
        assert_eq!(terminal.buffer[0][4].character, '中');
        // Verify second wide char wrapped to next line
        assert_eq!(terminal.buffer[1][0].character, '文');
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
    }

    #[test]
    fn test_placeholder_cell_creation() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write a wide character
        terminal.write_char('中'); // width 2
        
        // Verify the character is written at position 0
        assert_eq!(terminal.buffer[0][0].character, '中');
        
        // Verify placeholder cell is created at position 1
        assert_eq!(terminal.buffer[0][1].character, '\0');
        assert_eq!(terminal.buffer[0][1].color, egui::Color32::TRANSPARENT);
        assert_eq!(terminal.buffer[0][1].bold, false);
        
        // Verify cursor advanced by 2
        assert_eq!(terminal.cursor_col, 2);
    }

    #[test]
    fn test_placeholder_bounds_checking() {
        let mut terminal = TerminalEmulator::new(5, 5);
        
        // Position cursor at the last column
        terminal.cursor_col = 4;
        
        // Write a wide character that would need a placeholder beyond bounds
        terminal.write_char('文'); // width 2, should wrap to next line
        
        // Character should wrap to next line
        assert_eq!(terminal.buffer[1][0].character, '文');
        // Placeholder should be created at position 1 on the new line
        assert_eq!(terminal.buffer[1][1].character, '\0');
        assert_eq!(terminal.buffer[1][1].color, egui::Color32::TRANSPARENT);
        assert_eq!(terminal.buffer[1][1].bold, false);
    }

    #[test]
    fn test_multiple_wide_chars_placeholders() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write multiple wide characters
        terminal.write_char('中'); // pos 0-1
        terminal.write_char('文'); // pos 2-3
        terminal.write_char('字'); // pos 4-5
        
        // Verify characters and their placeholders
        assert_eq!(terminal.buffer[0][0].character, '中');
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][2].character, '文');
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][4].character, '字');
        assert_eq!(terminal.buffer[0][5].character, '\0'); // placeholder
        
        // Verify cursor position
        assert_eq!(terminal.cursor_col, 6);
    }

    #[test]
    fn test_ascii_no_placeholder() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write ASCII characters
        terminal.write_char('a');
        terminal.write_char('b');
        
        // Verify no placeholders are created for ASCII
        assert_eq!(terminal.buffer[0][0].character, 'a');
        assert_eq!(terminal.buffer[0][1].character, 'b');
        // Position 2 should still be default (space)
        assert_eq!(terminal.buffer[0][2].character, ' ');
        
        // Verify cursor position
        assert_eq!(terminal.cursor_col, 2);
    }

    #[test]
    fn test_placeholder_properties() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set terminal to bold and colored state
        terminal.bold = true;
        terminal.current_color = egui::Color32::RED;
        
        // Write a wide character
        terminal.write_char('中');
        
        // Verify the character inherits terminal state
        assert_eq!(terminal.buffer[0][0].character, '中');
        assert_eq!(terminal.buffer[0][0].color, egui::Color32::RED);
        assert_eq!(terminal.buffer[0][0].bold, true);
        
        // Verify placeholder has correct properties regardless of terminal state
        assert_eq!(terminal.buffer[0][1].character, '\0');
        assert_eq!(terminal.buffer[0][1].color, egui::Color32::TRANSPARENT);
        assert_eq!(terminal.buffer[0][1].bold, false);
    }

    #[test]
    fn test_placeholder_identification() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write mixed content
        terminal.write_char('a');  // ASCII at pos 0
        terminal.write_char('中'); // Wide char at pos 1, placeholder at pos 2
        terminal.write_char('b');  // ASCII at pos 3
        
        // Verify we can identify placeholders correctly
        assert_eq!(terminal.buffer[0][0].character, 'a');
        assert_ne!(terminal.buffer[0][0].character, '\0'); // Not a placeholder
        
        assert_eq!(terminal.buffer[0][1].character, '中');
        assert_ne!(terminal.buffer[0][1].character, '\0'); // Not a placeholder
        
        assert_eq!(terminal.buffer[0][2].character, '\0'); // This is a placeholder
        
        assert_eq!(terminal.buffer[0][3].character, 'b');
        assert_ne!(terminal.buffer[0][3].character, '\0'); // Not a placeholder
    }

    #[test]
    fn test_rendering_skip_logic() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Create a row with mixed content including placeholders
        terminal.write_char('H');  // pos 0
        terminal.write_char('中'); // pos 1, placeholder at pos 2
        terminal.write_char('i');  // pos 3
        
        let row = &terminal.buffer[0];
        
        // Count renderable characters (non-placeholder)
        let _renderable_count = row.iter().filter(|cell| cell.character != '\0').count();
        
        // Should have 3 renderable characters: 'H', '中', 'i'
        // Plus 7 default spaces, but the placeholder at pos 2 should not be counted
        // Actually, let's count non-default characters
        let non_default_chars = row.iter()
            .filter(|cell| cell.character != ' ' && cell.character != '\0')
            .count();
        
        assert_eq!(non_default_chars, 3); // 'H', '中', 'i'
        
        // Verify placeholder exists
        assert_eq!(row[2].character, '\0');
        
        // Verify other characters are correct
        assert_eq!(row[0].character, 'H');
        assert_eq!(row[1].character, '中');
        assert_eq!(row[3].character, 'i');
    }

    #[test]
    fn test_consecutive_placeholders() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write consecutive wide characters
        terminal.write_char('中'); // pos 0, placeholder at pos 1
        terminal.write_char('文'); // pos 2, placeholder at pos 3
        terminal.write_char('字'); // pos 4, placeholder at pos 5
        
        let row = &terminal.buffer[0];
        
        // Verify characters and placeholders
        assert_eq!(row[0].character, '中');
        assert_eq!(row[1].character, '\0'); // placeholder
        assert_eq!(row[2].character, '文');
        assert_eq!(row[3].character, '\0'); // placeholder
        assert_eq!(row[4].character, '字');
        assert_eq!(row[5].character, '\0'); // placeholder
        
        // Count placeholders
        let placeholder_count = row.iter().filter(|cell| cell.character == '\0').count();
        assert_eq!(placeholder_count, 3);
        
        // Count actual wide characters
        let wide_char_count = row.iter()
            .filter(|cell| cell.character != ' ' && cell.character != '\0')
            .count();
        assert_eq!(wide_char_count, 3);
    }

    #[test]
    fn test_cursor_wrapping_with_consecutive_wide_characters() {
        let mut terminal = TerminalEmulator::new(3, 5); // 3 rows, 5 columns
        
        // Test consecutive wide characters that should cause wrapping
        // Each emoji is width 2, so in a 5-column terminal:
        // - First emoji at col 0-1
        // - Second emoji at col 2-3  
        // - Third emoji should wrap to next line at col 0-1
        terminal.write_char('😀'); // width 2, cursor should be at col 2
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 0);
        
        terminal.write_char('😁'); // width 2, cursor should be at col 4
        assert_eq!(terminal.cursor_col, 4);
        assert_eq!(terminal.cursor_row, 0);
        
        terminal.write_char('😂'); // width 2, should wrap to next line, cursor at col 2
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
        
        // Verify the characters are placed correctly
        assert_eq!(terminal.buffer[0][0].character, '😀');
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][2].character, '😁');
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[1][0].character, '😂');
        assert_eq!(terminal.buffer[1][1].character, '\0'); // placeholder
    }

    #[test]
    fn test_cursor_position_validity_after_wide_character_placement() {
        let mut terminal = TerminalEmulator::new(2, 3); // 2 rows, 3 columns
        
        // Place a wide character at the edge
        terminal.cursor_col = 2; // At the last column
        terminal.write_char('😀'); // width 2, should wrap to next line
        
        // Cursor should be valid and on next line
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
        assert!(terminal.cursor_col < terminal.cols);
        assert!(terminal.cursor_row < terminal.rows);
        
        // Character should be on the new line
        assert_eq!(terminal.buffer[1][0].character, '😀');
        assert_eq!(terminal.buffer[1][1].character, '\0'); // placeholder
    }

    #[test]
    fn test_mixed_normal_and_wide_character_wrapping() {
        let mut terminal = TerminalEmulator::new(2, 4); // 2 rows, 4 columns
        
        // Mix normal and wide characters
        terminal.write_char('A'); // width 1, cursor at col 1
        assert_eq!(terminal.cursor_col, 1);
        
        terminal.write_char('😀'); // width 2, cursor at col 3
        assert_eq!(terminal.cursor_col, 3);
        
        terminal.write_char('B'); // width 1, fits at col 3, cursor advances to 4 then wraps to next line
        
        // After writing 'B' and cursor wrapping
        assert_eq!(terminal.cursor_col, 0); // Cursor wraps to start of next line
        assert_eq!(terminal.cursor_row, 1);
        
        // Verify placement - 'B' should be at the last position of first line
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, '😀');
        assert_eq!(terminal.buffer[0][2].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B'); // 'B' at last position of first line
    }

    // ANSI Integration Tests
    
    #[test]
    fn test_wide_characters_with_color_codes() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set red color and write wide character
        terminal.process_ansi_data("\x1b[31m中文\x1b[0m");
        
        // Verify wide characters have correct color
        assert_eq!(terminal.buffer[0][0].character, '中');
        assert_eq!(terminal.buffer[0][0].color, egui::Color32::from_rgb(205, 49, 49)); // Red
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][1].color, egui::Color32::TRANSPARENT);
        
        assert_eq!(terminal.buffer[0][2].character, '文');
        assert_eq!(terminal.buffer[0][2].color, egui::Color32::from_rgb(205, 49, 49)); // Red
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].color, egui::Color32::TRANSPARENT);
        
        // Cursor should be at position 4 after two wide characters
        assert_eq!(terminal.cursor_col, 4);
    }

    #[test]
    fn test_wide_characters_with_bold() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set bold and write wide character
        terminal.process_ansi_data("\x1b[1m😀😁\x1b[22m");
        
        // Verify wide characters have bold attribute
        assert_eq!(terminal.buffer[0][0].character, '😀');
        assert_eq!(terminal.buffer[0][0].bold, true);
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][1].bold, false); // placeholders are never bold
        
        assert_eq!(terminal.buffer[0][2].character, '😁');
        assert_eq!(terminal.buffer[0][2].bold, true);
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].bold, false); // placeholders are never bold
        
        // Cursor should be at position 4
        assert_eq!(terminal.cursor_col, 4);
    }

    #[test]
    fn test_cursor_positioning_with_wide_characters() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write some wide characters
        terminal.process_ansi_data("中文");
        assert_eq!(terminal.cursor_col, 4); // Two wide chars = 4 columns
        
        // Move cursor to specific position
        terminal.process_ansi_data("\x1b[1;3H"); // Move to row 1, col 3 (0-indexed: row 0, col 2)
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 2);
        
        // Write a wide character at this position
        terminal.process_ansi_data("字");
        
        // Verify character placement
        assert_eq!(terminal.buffer[0][2].character, '字');
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.cursor_col, 4);
    }

    #[test]
    fn test_cursor_movement_commands_with_wide_characters() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write wide characters
        terminal.process_ansi_data("😀😁😂");
        assert_eq!(terminal.cursor_col, 6); // Three wide chars = 6 columns
        
        // Move cursor backward by 2
        terminal.process_ansi_data("\x1b[2D");
        assert_eq!(terminal.cursor_col, 4);
        
        // Move cursor forward by 1
        terminal.process_ansi_data("\x1b[1C");
        assert_eq!(terminal.cursor_col, 5);
        
        // Write a normal character
        terminal.process_ansi_data("A");
        assert_eq!(terminal.buffer[0][5].character, 'A');
        assert_eq!(terminal.cursor_col, 6);
    }

    #[test]
    fn test_screen_clearing_with_placeholders() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Fill screen with wide characters
        terminal.process_ansi_data("中文字😀😁");
        
        // Verify characters and placeholders are present
        assert_eq!(terminal.buffer[0][0].character, '中');
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][2].character, '文');
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        
        // Clear entire screen
        terminal.process_ansi_data("\x1b[2J");
        
        // Verify all characters and placeholders are cleared
        for row in 0..terminal.rows {
            for col in 0..terminal.cols {
                assert_eq!(terminal.buffer[row][col].character, ' ');
                assert_eq!(terminal.buffer[row][col].color, egui::Color32::WHITE);
                assert_eq!(terminal.buffer[row][col].bold, false);
            }
        }
        
        // Verify cursor is reset
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 0);
    }

    #[test]
    fn test_line_clearing_with_placeholders() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Write wide characters on first line
        terminal.process_ansi_data("中文字");
        terminal.process_ansi_data("\n"); // Move to next line
        terminal.process_ansi_data("normal text");
        
        // Move cursor back to first line
        terminal.process_ansi_data("\x1b[1;3H"); // Row 1, Col 3 (0-indexed: row 0, col 2)
        
        // Clear from cursor to end of line
        terminal.process_ansi_data("\x1b[0K");
        
        // Verify partial line clearing worked correctly
        assert_eq!(terminal.buffer[0][0].character, '中');
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][2].character, ' '); // cleared
        assert_eq!(terminal.buffer[0][3].character, ' '); // cleared
        assert_eq!(terminal.buffer[0][4].character, ' '); // cleared
        
        // Second line should be intact
        assert_eq!(terminal.buffer[1][0].character, 'n');
        assert_eq!(terminal.buffer[1][1].character, 'o');
    }

    #[test]
    fn test_256_color_with_wide_characters() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set 256-color foreground and write wide character
        terminal.process_ansi_data("\x1b[38;5;196m😀\x1b[0m"); // Bright red (color 196)
        
        // Verify wide character has correct 256-color
        assert_eq!(terminal.buffer[0][0].character, '😀');
        assert_eq!(terminal.buffer[0][0].color, ansi_256_to_rgb(196));
        assert_eq!(terminal.buffer[0][1].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][1].color, egui::Color32::TRANSPARENT);
        
        assert_eq!(terminal.cursor_col, 2);
    }

    #[test]
    fn test_mixed_ansi_and_wide_characters() {
        let mut terminal = TerminalEmulator::new(5, 20); // Increase width to accommodate all characters
        
        // Complex ANSI sequence with wide characters
        terminal.process_ansi_data("Hello \x1b[31m中\x1b[32m文\x1b[0m World 😀");
        
        // Verify mixed content
        assert_eq!(terminal.buffer[0][0].character, 'H');
        assert_eq!(terminal.buffer[0][1].character, 'e');
        assert_eq!(terminal.buffer[0][2].character, 'l');
        assert_eq!(terminal.buffer[0][3].character, 'l');
        assert_eq!(terminal.buffer[0][4].character, 'o');
        assert_eq!(terminal.buffer[0][5].character, ' ');
        
        // Red wide character
        assert_eq!(terminal.buffer[0][6].character, '中');
        assert_eq!(terminal.buffer[0][6].color, egui::Color32::from_rgb(205, 49, 49)); // Red
        assert_eq!(terminal.buffer[0][7].character, '\0'); // placeholder
        
        // Green wide character
        assert_eq!(terminal.buffer[0][8].character, '文');
        assert_eq!(terminal.buffer[0][8].color, egui::Color32::from_rgb(13, 188, 121)); // Green
        assert_eq!(terminal.buffer[0][9].character, '\0'); // placeholder
        
        // Reset to white
        assert_eq!(terminal.buffer[0][10].character, ' ');
        assert_eq!(terminal.buffer[0][10].color, egui::Color32::WHITE);
        assert_eq!(terminal.buffer[0][11].character, 'W');
        assert_eq!(terminal.buffer[0][11].color, egui::Color32::WHITE);
        assert_eq!(terminal.buffer[0][12].character, 'o');
        assert_eq!(terminal.buffer[0][13].character, 'r');
        assert_eq!(terminal.buffer[0][14].character, 'l');
        assert_eq!(terminal.buffer[0][15].character, 'd');
        assert_eq!(terminal.buffer[0][16].character, ' ');
        
        // Emoji at the end
        assert_eq!(terminal.buffer[0][17].character, '😀');
        assert_eq!(terminal.buffer[0][18].character, '\0'); // placeholder
    }

    #[test]
    fn test_existing_terminal_functionality_intact() {
        let mut terminal = TerminalEmulator::new(5, 15); // Increase width to accommodate all characters
        
        // Test basic functionality still works
        terminal.process_ansi_data("Hello");
        assert_eq!(terminal.cursor_col, 5);
        
        // Test newline
        terminal.process_ansi_data("\n");
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 0);
        
        // Test carriage return
        terminal.process_ansi_data("Test\r");
        assert_eq!(terminal.cursor_col, 0);
        assert_eq!(terminal.cursor_row, 1);
        
        // Test tab
        terminal.process_ansi_data("\t");
        assert_eq!(terminal.cursor_col, 8); // Next tab stop
        
        // Test color changes
        terminal.process_ansi_data("\x1b[31mRed\x1b[0m");
        assert_eq!(terminal.buffer[1][8].character, 'R');
        assert_eq!(terminal.buffer[1][8].color, egui::Color32::from_rgb(205, 49, 49));
        assert_eq!(terminal.buffer[1][9].character, 'e');
        assert_eq!(terminal.buffer[1][10].character, 'd');
    }

    #[test]
    fn test_box_drawing_with_ansi_colors() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Draw colored box-drawing characters
        terminal.process_ansi_data("\x1b[36m┌─┐\n│ │\n└─┘\x1b[0m");
        
        // Verify box-drawing characters have correct color (should be gray, not cyan)
        // Box-drawing characters use gray color regardless of current color
        assert_eq!(terminal.buffer[0][0].character, '┌');
        assert_eq!(terminal.buffer[0][0].color, egui::Color32::from_rgb(128, 128, 128)); // Gray
        assert_eq!(terminal.buffer[0][1].character, '─');
        assert_eq!(terminal.buffer[0][1].color, egui::Color32::from_rgb(128, 128, 128)); // Gray
        assert_eq!(terminal.buffer[0][2].character, '┐');
        assert_eq!(terminal.buffer[0][2].color, egui::Color32::from_rgb(128, 128, 128)); // Gray
        
        // Verify structure
        assert_eq!(terminal.buffer[1][0].character, '│');
        assert_eq!(terminal.buffer[1][1].character, ' ');
        assert_eq!(terminal.buffer[1][2].character, '│');
        
        assert_eq!(terminal.buffer[2][0].character, '└');
        assert_eq!(terminal.buffer[2][1].character, '─');
        assert_eq!(terminal.buffer[2][2].character, '┘');
    }

    // Visual Testing and Validation Tests
    
    #[test]
    fn test_emoji_visual_alignment() {
        let mut terminal = TerminalEmulator::new(5, 12);
        
        // Create a pattern with emojis and regular characters for alignment testing
        terminal.process_ansi_data("A😀B😁C\n");
        terminal.process_ansi_data("123456789\n");
        terminal.process_ansi_data("😂😃😄😅\n");
        
        // Verify first line alignment: A😀B😁C
        assert_eq!(terminal.buffer[0][0].character, 'A');      // pos 0
        assert_eq!(terminal.buffer[0][1].character, '😀');     // pos 1 (width 2)
        assert_eq!(terminal.buffer[0][2].character, '\0');     // pos 2 (placeholder)
        assert_eq!(terminal.buffer[0][3].character, 'B');      // pos 3
        assert_eq!(terminal.buffer[0][4].character, '😁');     // pos 4 (width 2)
        assert_eq!(terminal.buffer[0][5].character, '\0');     // pos 5 (placeholder)
        assert_eq!(terminal.buffer[0][6].character, 'C');      // pos 6
        
        // Verify second line for reference alignment
        assert_eq!(terminal.buffer[1][0].character, '1');
        assert_eq!(terminal.buffer[1][1].character, '2');
        assert_eq!(terminal.buffer[1][2].character, '3');
        assert_eq!(terminal.buffer[1][3].character, '4');
        assert_eq!(terminal.buffer[1][4].character, '5');
        assert_eq!(terminal.buffer[1][5].character, '6');
        assert_eq!(terminal.buffer[1][6].character, '7');
        
        // Verify third line with consecutive emojis
        assert_eq!(terminal.buffer[2][0].character, '😂');     // pos 0-1
        assert_eq!(terminal.buffer[2][1].character, '\0');     // placeholder
        assert_eq!(terminal.buffer[2][2].character, '😃');     // pos 2-3
        assert_eq!(terminal.buffer[2][3].character, '\0');     // placeholder
        assert_eq!(terminal.buffer[2][4].character, '😄');     // pos 4-5
        assert_eq!(terminal.buffer[2][5].character, '\0');     // placeholder
        assert_eq!(terminal.buffer[2][6].character, '😅');     // pos 6-7
        assert_eq!(terminal.buffer[2][7].character, '\0');     // placeholder
    }

    #[test]
    fn test_box_drawing_visual_alignment() {
        let mut terminal = TerminalEmulator::new(6, 15);
        
        // Create a simple box for alignment testing
        terminal.process_ansi_data("┌──┐\n");
        terminal.process_ansi_data("│Hi│\n");
        terminal.process_ansi_data("│中│\n");
        terminal.process_ansi_data("└──┘\n");
        
        // Verify top border
        assert_eq!(terminal.buffer[0][0].character, '┌');
        assert_eq!(terminal.buffer[0][1].character, '─');
        assert_eq!(terminal.buffer[0][2].character, '─');
        assert_eq!(terminal.buffer[0][3].character, '┐');
        
        // Verify ASCII content line
        assert_eq!(terminal.buffer[1][0].character, '│');
        assert_eq!(terminal.buffer[1][1].character, 'H');
        assert_eq!(terminal.buffer[1][2].character, 'i');
        assert_eq!(terminal.buffer[1][3].character, '│');
        
        // Verify wide character line - the wide character should fit properly
        assert_eq!(terminal.buffer[2][0].character, '│');
        assert_eq!(terminal.buffer[2][1].character, '中');    // width 2
        assert_eq!(terminal.buffer[2][2].character, '\0');    // placeholder
        assert_eq!(terminal.buffer[2][3].character, '│');
        
        // Verify bottom border
        assert_eq!(terminal.buffer[3][0].character, '└');
        assert_eq!(terminal.buffer[3][1].character, '─');
        assert_eq!(terminal.buffer[3][2].character, '─');
        assert_eq!(terminal.buffer[3][3].character, '┘');
    }

    #[test]
    fn test_cursor_position_visual_consistency() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Test cursor position after various wide character operations
        terminal.process_ansi_data("A");
        assert_eq!(terminal.cursor_col, 1);
        
        terminal.process_ansi_data("😀");
        assert_eq!(terminal.cursor_col, 3); // A + emoji (width 2) = position 3
        
        terminal.process_ansi_data("B");
        assert_eq!(terminal.cursor_col, 4);
        
        terminal.process_ansi_data("中");
        assert_eq!(terminal.cursor_col, 6); // Previous + CJK (width 2) = position 6
        
        // Verify visual layout matches cursor position
        assert_eq!(terminal.buffer[0][0].character, 'A');      // pos 0
        assert_eq!(terminal.buffer[0][1].character, '😀');     // pos 1-2
        assert_eq!(terminal.buffer[0][2].character, '\0');     // placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B');      // pos 3
        assert_eq!(terminal.buffer[0][4].character, '中');     // pos 4-5
        assert_eq!(terminal.buffer[0][5].character, '\0');     // placeholder
        
        // Test cursor positioning command
        terminal.process_ansi_data("\x1b[1;3H"); // Move to row 1, col 3 (0-indexed: row 0, col 2)
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 2);
        
        // Write at cursor position and verify it doesn't corrupt wide characters
        terminal.process_ansi_data("X");
        assert_eq!(terminal.buffer[0][2].character, 'X'); // Should overwrite placeholder
        assert_eq!(terminal.cursor_col, 3);
    }

    #[test]
    fn test_text_selection_behavior_simulation() {
        let mut terminal = TerminalEmulator::new(3, 25);
        
        // Test text selection simulation with wide characters
        terminal.process_ansi_data("A中B\n");
        
        let row = &terminal.buffer[0];
        
        // Simulate text selection by collecting non-placeholder characters
        let mut selectable_chars = Vec::new();
        for cell in row.iter() {
            if cell.character != '\0' && cell.character != ' ' {
                selectable_chars.push(cell.character);
            }
        }
        
        // Should be able to select: A, 中, B
        assert_eq!(selectable_chars, vec!['A', '中', 'B']);
        
        // Verify placeholder exists for wide character
        assert_eq!(row[0].character, 'A');
        assert_eq!(row[1].character, '中');
        assert_eq!(row[2].character, '\0'); // placeholder
        assert_eq!(row[3].character, 'B');
        
        // Count placeholders
        let placeholder_count = row.iter().filter(|cell| cell.character == '\0').count();
        assert_eq!(placeholder_count, 1); // One for 中
    }

    #[test]
    fn test_visual_rendering_skip_logic() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Create content with placeholders
        terminal.process_ansi_data("A😀B中C\n");
        
        let row = &terminal.buffer[0];
        
        // Simulate the rendering logic that skips placeholders
        let mut rendered_chars = Vec::new();
        for cell in row.iter() {
            // This mimics the render_row logic
            if cell.character == '\0' {
                continue; // Skip placeholder
            }
            rendered_chars.push(cell.character);
        }
        
        // Verify only non-placeholder characters would be rendered
        // A😀B中C takes positions: A(0), 😀(1), placeholder(2), B(3), 中(4), placeholder(5), C(6), spaces(7,8,9)
        let expected_rendered = vec!['A', '😀', 'B', '中', 'C', ' ', ' ', ' '];
        assert_eq!(rendered_chars, expected_rendered);
        
        // Count actual content vs placeholders
        let content_count = row.iter().filter(|cell| cell.character != '\0' && cell.character != ' ').count();
        let placeholder_count = row.iter().filter(|cell| cell.character == '\0').count();
        
        assert_eq!(content_count, 5); // A, 😀, B, 中, C
        assert_eq!(placeholder_count, 2); // One for 😀, one for 中
    }

    #[test]
    fn test_wide_character_boundary_visual_behavior() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Test wide character wrapping behavior
        terminal.process_ansi_data("AB");   // positions 0,1
        assert_eq!(terminal.cursor_col, 2);
        
        terminal.process_ansi_data("😀");   // width 2, should fit at positions 2-3
        
        // Verify characters are placed correctly
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, 'B');
        assert_eq!(terminal.buffer[0][2].character, '😀');
        assert_eq!(terminal.buffer[0][3].character, '\0'); // placeholder
        assert_eq!(terminal.cursor_col, 4);
        assert_eq!(terminal.cursor_row, 0);
        
        // Add one more character to fill the line
        terminal.process_ansi_data("C");
        assert_eq!(terminal.buffer[0][4].character, 'C');
        assert_eq!(terminal.cursor_col, 0); // Should wrap to next line
        assert_eq!(terminal.cursor_row, 1);
        
        // Add a wide character that should fit on the new line
        terminal.process_ansi_data("中");
        assert_eq!(terminal.buffer[1][0].character, '中');
        assert_eq!(terminal.buffer[1][1].character, '\0'); // placeholder
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.cursor_row, 1);
    }

    #[test]
    fn test_complex_visual_layout_validation() {
        let mut terminal = TerminalEmulator::new(5, 25);
        
        // Create a simpler layout for testing
        terminal.process_ansi_data("┌─────┐\n");
        terminal.process_ansi_data("│Hello│\n");
        terminal.process_ansi_data("│中文 │\n");
        terminal.process_ansi_data("│😀😁 │\n");
        terminal.process_ansi_data("└─────┘\n");
        
        // Validate the visual structure
        
        // Top border
        assert_eq!(terminal.buffer[0][0].character, '┌');
        assert_eq!(terminal.buffer[0][1].character, '─');
        assert_eq!(terminal.buffer[0][5].character, '─');
        assert_eq!(terminal.buffer[0][6].character, '┐');
        
        // ASCII line
        assert_eq!(terminal.buffer[1][0].character, '│');
        assert_eq!(terminal.buffer[1][1].character, 'H');
        assert_eq!(terminal.buffer[1][2].character, 'e');
        assert_eq!(terminal.buffer[1][3].character, 'l');
        assert_eq!(terminal.buffer[1][4].character, 'l');
        assert_eq!(terminal.buffer[1][5].character, 'o');
        assert_eq!(terminal.buffer[1][6].character, '│');
        
        // CJK line - wide characters
        assert_eq!(terminal.buffer[2][0].character, '│');
        assert_eq!(terminal.buffer[2][1].character, '中');    // width 2
        assert_eq!(terminal.buffer[2][2].character, '\0');    // placeholder
        assert_eq!(terminal.buffer[2][3].character, '文');    // width 2
        assert_eq!(terminal.buffer[2][4].character, '\0');    // placeholder
        assert_eq!(terminal.buffer[2][5].character, ' ');
        assert_eq!(terminal.buffer[2][6].character, '│');
        
        // Emoji line
        assert_eq!(terminal.buffer[3][0].character, '│');
        assert_eq!(terminal.buffer[3][1].character, '�');    /// width 2
        assert_eq!(terminal.buffer[3][2].character, '\0');    // placeholder
        assert_eq!(terminal.buffer[3][3].character, '😁');    // width 2
        assert_eq!(terminal.buffer[3][4].character, '\0');    // placeholder
        assert_eq!(terminal.buffer[3][5].character, ' ');
        assert_eq!(terminal.buffer[3][6].character, '│');
        
        // Bottom border
        assert_eq!(terminal.buffer[4][0].character, '└');
        assert_eq!(terminal.buffer[4][6].character, '┘');
        
        // Verify cursor position
        assert_eq!(terminal.cursor_row, 4);
        assert_eq!(terminal.cursor_col, 0);
    }

    // Error Handling and Edge Case Tests
    
    #[test]
    fn test_bounds_checking_buffer_operations() {
        let mut terminal = TerminalEmulator::new(2, 3);
        
        // Test writing beyond buffer bounds
        terminal.cursor_row = 10; // Way beyond bounds
        terminal.cursor_col = 10; // Way beyond bounds
        
        // This should not panic and should validate cursor position
        terminal.write_char('A');
        
        // Cursor should be corrected to valid bounds
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
        
        // Test ANSI cursor movement beyond bounds
        terminal.process_ansi_data("\x1b[100;100H"); // Move to position 100,100
        
        // Should be clamped to valid bounds
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
    }

    #[test]
    fn test_unicode_width_none_handling() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Test with control characters that might return None from unicode-width
        terminal.write_char('\u{0000}'); // NULL character
        terminal.write_char('\u{0001}'); // SOH character
        terminal.write_char('\u{007F}'); // DEL character
        
        // Should not panic and cursor should remain valid
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
        
        // Test with some other potentially problematic characters
        terminal.write_char('\u{200B}'); // Zero-width space
        terminal.write_char('\u{FEFF}'); // Byte order mark
        
        // Should handle gracefully
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
    }

    #[test]
    fn test_malformed_input_handling() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Test with malformed ANSI sequences
        terminal.process_ansi_data("\x1b["); // Incomplete sequence
        terminal.process_ansi_data("\x1b[999999999999999999999H"); // Huge numbers
        terminal.process_ansi_data("\x1b[;;;;;H"); // Multiple separators
        terminal.process_ansi_data("\x1b[abcdefH"); // Invalid characters
        
        // Should not panic and terminal should remain functional
        terminal.write_char('A');
        assert_eq!(terminal.buffer[terminal.cursor_row][terminal.cursor_col - 1].character, 'A');
        
        // Test with invalid color codes
        terminal.process_ansi_data("\x1b[999999999m"); // Invalid color
        terminal.process_ansi_data("\x1b[38;5;999m"); // Invalid 256-color
        terminal.process_ansi_data("\x1b[38;5;m"); // Missing color index
        
        // Should not panic and colors should remain valid
        terminal.write_char('B');
        assert_eq!(terminal.buffer[terminal.cursor_row][terminal.cursor_col - 1].character, 'B');
    }

    #[test]
    fn test_buffer_integrity_maintenance() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Corrupt buffer dimensions (simulate memory corruption)
        terminal.buffer.clear(); // Empty buffer
        
        // Try to write - should reconstruct buffer
        terminal.write_char('A');
        
        // Buffer should be reconstructed with correct dimensions
        assert_eq!(terminal.buffer.len(), terminal.rows);
        for row in &terminal.buffer {
            assert_eq!(row.len(), terminal.cols);
        }
        
        // Test with inconsistent row lengths
        terminal.buffer[0].clear(); // Make first row empty
        terminal.write_char('B');
        
        // Should fix the inconsistent row
        assert_eq!(terminal.buffer[0].len(), terminal.cols);
    }

    #[test]
    fn test_cursor_position_validation() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Test extreme cursor positions
        terminal.cursor_row = usize::MAX;
        terminal.cursor_col = usize::MAX;
        
        terminal.validate_cursor_position();
        
        // Should be corrected to valid bounds
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
        
        // Test with zero-sized terminal
        let mut zero_terminal = TerminalEmulator::new(0, 0);
        zero_terminal.write_char('A'); // Should not panic
        zero_terminal.move_cursor(10, 10); // Should not panic
        
        // Test cursor movement with overflow protection
        terminal.cursor_col = usize::MAX - 1;
        terminal.process_ansi_data("\x1b[1000C"); // Move right by 1000
        
        // Should not overflow
        assert!(terminal.cursor_col < terminal.cols);
    }

    #[test]
    fn test_wide_character_edge_cases() {
        let mut terminal = TerminalEmulator::new(3, 3);
        
        // Test wide character at exact boundary
        terminal.cursor_col = 2; // Last position
        terminal.write_char('中'); // Width 2, should wrap
        
        // Should wrap to next line
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 2);
        assert_eq!(terminal.buffer[1][0].character, '中');
        assert_eq!(terminal.buffer[1][1].character, '\0');
        
        // Test placeholder creation at buffer edge
        terminal.cursor_col = 2; // Last position
        terminal.cursor_row = 2; // Last row
        terminal.write_char('文'); // Should handle gracefully
        
        // Should not create placeholder beyond bounds
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col <= terminal.cols);
    }

    #[test]
    fn test_ansi_parameter_bounds_checking() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Test with extremely large parameters
        terminal.process_ansi_data("\x1b[999999999A"); // Cursor up by huge amount
        terminal.process_ansi_data("\x1b[999999999B"); // Cursor down by huge amount
        terminal.process_ansi_data("\x1b[999999999C"); // Cursor right by huge amount
        terminal.process_ansi_data("\x1b[999999999D"); // Cursor left by huge amount
        
        // Should not overflow and cursor should remain valid
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
        
        // Test graphics mode with excessive parameters
        let long_params = "38;5;".repeat(50) + "196m"; // Very long parameter string
        terminal.process_ansi_data(&format!("\x1b[{}m", long_params));
        
        // Should not hang or crash
        terminal.write_char('A');
        assert_eq!(terminal.buffer[terminal.cursor_row][terminal.cursor_col - 1].character, 'A');
    }

    #[test]
    fn test_screen_clearing_edge_cases() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Fill terminal with content including wide characters
        terminal.process_ansi_data("A中B\n文字\nTest");
        
        // Test clearing with cursor at various positions
        terminal.cursor_row = 1;
        terminal.cursor_col = 3;
        
        // Clear from cursor to end - should handle placeholders correctly
        terminal.process_ansi_data("\x1b[0J");
        
        // Verify clearing worked and didn't corrupt buffer
        assert_eq!(terminal.buffer.len(), terminal.rows);
        for row in &terminal.buffer {
            assert_eq!(row.len(), terminal.cols);
        }
        
        // Test line clearing with wide characters
        terminal.process_ansi_data("中文字");
        terminal.cursor_col = 2; // In middle of wide character sequence
        terminal.process_ansi_data("\x1b[2K"); // Clear entire line
        
        // Line should be completely cleared
        for cell in &terminal.buffer[terminal.cursor_row] {
            assert_eq!(cell.character, ' ');
        }
    }

    #[test]
    fn test_emoji_detection_and_width() {
        // Test comprehensive emoji detection - including the new ranges
        assert_eq!(TerminalEmulator::get_char_width('😀'), 2); // Grinning face (1F600-1F64F)
        assert_eq!(TerminalEmulator::get_char_width('🎵'), 2); // Musical note (1F300-1F5FF)
        assert_eq!(TerminalEmulator::get_char_width('🔥'), 2); // Fire (1F300-1F5FF)
        assert_eq!(TerminalEmulator::get_char_width('🚀'), 2); // Rocket (1F680-1F6FF)
        assert_eq!(TerminalEmulator::get_char_width('🌟'), 2); // Star (1F300-1F5FF)
        assert_eq!(TerminalEmulator::get_char_width('🎉'), 2); // Party popper (1F300-1F5FF)
        assert_eq!(TerminalEmulator::get_char_width('💻'), 2); // Laptop (1F300-1F5FF)
        assert_eq!(TerminalEmulator::get_char_width('🤖'), 2); // Robot (1F910-1F96B)
        
        // Test dingbats (2700-27BF) - these were causing alignment issues
        assert_eq!(TerminalEmulator::get_char_width('✂'), 2);  // Scissors
        assert_eq!(TerminalEmulator::get_char_width('✅'), 2); // Check mark
        assert_eq!(TerminalEmulator::get_char_width('✈'), 2);  // Airplane
        assert_eq!(TerminalEmulator::get_char_width('✉'), 2);  // Envelope
        assert_eq!(TerminalEmulator::get_char_width('✊'), 2);  // Raised fist
        assert_eq!(TerminalEmulator::get_char_width('✨'), 2); // Sparkles
        
        // Test miscellaneous symbols (2600-26FF) - also problematic
        assert_eq!(TerminalEmulator::get_char_width('☀'), 2);  // Sun
        assert_eq!(TerminalEmulator::get_char_width('⭐'), 2); // Star
        assert_eq!(TerminalEmulator::get_char_width('⚡'), 2); // Lightning
        assert_eq!(TerminalEmulator::get_char_width('❄'), 2);  // Snowflake
        assert_eq!(TerminalEmulator::get_char_width('❤'), 2);  // Heart - now detected!
        
        // Test specific problematic characters
        assert_eq!(TerminalEmulator::get_char_width('♥'), 2);  // Heart suit
        assert_eq!(TerminalEmulator::get_char_width('♠'), 2);  // Spade suit
        assert_eq!(TerminalEmulator::get_char_width('♣'), 2);  // Club suit
        assert_eq!(TerminalEmulator::get_char_width('♦'), 2);  // Diamond suit
        assert_eq!(TerminalEmulator::get_char_width('⚠'), 2);  // Warning sign
        
        // Test that regular characters are still width 1
        assert_eq!(TerminalEmulator::get_char_width('A'), 1);
        assert_eq!(TerminalEmulator::get_char_width('1'), 1);
        assert_eq!(TerminalEmulator::get_char_width(' '), 1);
        
        // Test CJK characters are still width 2
        assert_eq!(TerminalEmulator::get_char_width('中'), 2);
        assert_eq!(TerminalEmulator::get_char_width('文'), 2);
    }

    #[test]
    fn test_emoji_alignment_consistency() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Test consistent emoji alignment
        terminal.process_ansi_data("A😀B😁C\n");
        terminal.process_ansi_data("123456789\n");
        
        // Verify alignment: A😀B😁C should align with 123456789
        // A at pos 0, 😀 at pos 1-2, B at pos 3, 😁 at pos 4-5, C at pos 6
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, '😀');
        assert_eq!(terminal.buffer[0][2].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B');
        assert_eq!(terminal.buffer[0][4].character, '😁');
        assert_eq!(terminal.buffer[0][5].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][6].character, 'C');
        
        // Second line should align
        assert_eq!(terminal.buffer[1][0].character, '1');
        assert_eq!(terminal.buffer[1][1].character, '2');
        assert_eq!(terminal.buffer[1][2].character, '3');
        assert_eq!(terminal.buffer[1][3].character, '4');
        assert_eq!(terminal.buffer[1][4].character, '5');
        assert_eq!(terminal.buffer[1][5].character, '6');
        assert_eq!(terminal.buffer[1][6].character, '7');
    }

    // Performance Testing and Optimization Tests
    
    #[test]
    fn test_rendering_performance_with_placeholders() {
        let mut terminal = TerminalEmulator::new(50, 100);
        
        // Fill terminal with mixed content including many wide characters
        let test_content = "A😀B中C文D😁E字F🚀G🎵H💻I🔥J".repeat(10);
        
        let start_time = std::time::Instant::now();
        
        // Process large amount of mixed content
        for _ in 0..100 {
            terminal.process_ansi_data(&test_content);
            terminal.process_ansi_data("\n");
        }
        
        let processing_duration = start_time.elapsed();
        
        // Simulate rendering by counting non-placeholder characters
        let render_start = std::time::Instant::now();
        let mut rendered_chars = 0;
        
        for row in &terminal.buffer {
            for cell in row {
                if cell.character != '\0' {
                    rendered_chars += 1;
                }
            }
        }
        
        let rendering_duration = render_start.elapsed();
        
        // Performance assertions - these are reasonable thresholds
        assert!(processing_duration.as_millis() < 1000, "Processing took too long: {:?}", processing_duration);
        assert!(rendering_duration.as_millis() < 100, "Rendering took too long: {:?}", rendering_duration);
        assert!(rendered_chars > 0, "Should have rendered some characters");
        
        println!("Performance test results:");
        println!("  Processing time: {:?}", processing_duration);
        println!("  Rendering time: {:?}", rendering_duration);
        println!("  Characters rendered: {}", rendered_chars);
    }

    #[test]
    fn test_memory_usage_with_wide_characters() {
        // Test memory usage with large amounts of wide characters
        let mut terminals = Vec::new();
        
        // Create multiple terminals with wide character content
        for i in 0..10 {
            let mut terminal = TerminalEmulator::new(100, 200);
            
            // Fill with wide characters
            let wide_content = "😀😁😂🤣😃😄😅😆😉😊😋😎😍😘🥰😗😙😚☺️🙂🤗🤩🤔🤨😐😑😶🙄😏😣😥😮🤐😯😪😫🥱😴😌😛😜😝🤤😒😓😔😕🙃🤑😲☹️🙁😖😞😟😤😢😭😦😧😨😩🤯😬😰😱🥵🥶😳🤪😵🥴😠😡🤬😷🤒🤕🤢🤮🤧😇🥳🥺🤠🤡🤥🤫🤭🧐🤓😈👿👹👺💀☠️👻👽👾🤖💩😺😸😹😻😼😽🙀😿😾";
            
            for _ in 0..50 {
                terminal.process_ansi_data(wide_content);
                terminal.process_ansi_data("\n");
            }
            
            terminals.push(terminal);
            
            // Basic memory usage check - ensure we're not leaking memory
            assert_eq!(terminals[i].buffer.len(), 100);
            assert_eq!(terminals[i].buffer[0].len(), 200);
        }
        
        // Verify all terminals are properly structured
        for terminal in &terminals {
            assert_eq!(terminal.rows, 100);
            assert_eq!(terminal.cols, 200);
            assert_eq!(terminal.buffer.len(), terminal.rows);
            
            for row in &terminal.buffer {
                assert_eq!(row.len(), terminal.cols);
            }
        }
        
        println!("Memory test completed with {} terminals", terminals.len());
    }

    #[test]
    fn test_normal_character_performance_regression() {
        let mut terminal = TerminalEmulator::new(50, 100);
        
        // Test with only ASCII characters (baseline performance)
        let ascii_content = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        
        let start_time = std::time::Instant::now();
        
        // Process large amount of ASCII content
        for _ in 0..1000 {
            terminal.process_ansi_data(ascii_content);
            terminal.process_ansi_data("\n");
        }
        
        let ascii_duration = start_time.elapsed();
        
        // Reset terminal
        terminal.clear_screen();
        
        // Test with mixed content
        let mixed_content = "ABC😀DEF中GHI文JKL😁MNO字PQR🚀STU";
        
        let mixed_start = std::time::Instant::now();
        
        // Process same amount of mixed content
        for _ in 0..1000 {
            terminal.process_ansi_data(mixed_content);
            terminal.process_ansi_data("\n");
        }
        
        let mixed_duration = mixed_start.elapsed();
        
        // Performance regression check - mixed content shouldn't be more than 3x slower
        let performance_ratio = mixed_duration.as_nanos() as f64 / ascii_duration.as_nanos() as f64;
        
        assert!(performance_ratio < 3.0, 
            "Performance regression detected: mixed content is {:.2}x slower than ASCII", 
            performance_ratio);
        
        println!("Performance regression test results:");
        println!("  ASCII processing time: {:?}", ascii_duration);
        println!("  Mixed processing time: {:?}", mixed_duration);
        println!("  Performance ratio: {:.2}x", performance_ratio);
    }

    #[test]
    fn test_width_calculation_optimization() {
        // Test the performance of width calculation for different character types
        let test_chars = vec![
            ('A', "ASCII"),
            ('中', "CJK"),
            ('😀', "Emoji"),
            ('│', "Box Drawing"),
            ('⭐', "Symbol"),
        ];
        
        for (ch, char_type) in test_chars {
            let start_time = std::time::Instant::now();
            
            // Perform many width calculations
            for _ in 0..100_000 {
                let _width = TerminalEmulator::get_char_width(ch);
            }
            
            let duration = start_time.elapsed();
            
            // Width calculation should be very fast
            assert!(duration.as_millis() < 100, 
                "Width calculation for {} ({}) took too long: {:?}", 
                char_type, ch, duration);
            
            println!("Width calculation for {} ({}): {:?}", char_type, ch, duration);
        }
    }

    #[test]
    fn test_placeholder_skipping_efficiency() {
        let mut terminal = TerminalEmulator::new(20, 50);
        
        // Create content with many placeholders
        let wide_content = "😀😁😂🤣😃😄😅😆😉😊😋😎😍😘🥰😗😙😚☺️🙂🤗🤩🤔🤨😐😑😶🙄😏😣😥😮🤐😯😪😫🥱😴😌😛😜😝🤤😒😓😔😕🙃🤑😲☹️🙁😖😞😟😤😢😭😦😧😨😩🤯😬😰😱🥵🥶😳🤪😵🥴😠😡🤬😷🤒🤕🤢🤮🤧😇🥳🥺🤠🤡🤥🤫🤭🧐🤓😈👿👹👺💀☠️👻👽👾🤖💩😺😸😹😻😼😽🙀😿😾";
        
        for _ in 0..10 {
            terminal.process_ansi_data(wide_content);
            terminal.process_ansi_data("\n");
        }
        
        // Count placeholders vs real characters
        let mut placeholder_count = 0;
        let mut character_count = 0;
        
        let start_time = std::time::Instant::now();
        
        for row in &terminal.buffer {
            for cell in row {
                if cell.character == '\0' {
                    placeholder_count += 1;
                } else if cell.character != ' ' {
                    character_count += 1;
                }
            }
        }
        
        let counting_duration = start_time.elapsed();
        
        // Simulate rendering by skipping placeholders
        let render_start = std::time::Instant::now();
        let mut rendered_count = 0;
        
        for row in &terminal.buffer {
            for cell in row {
                // This simulates the render_row logic
                if cell.character == '\0' {
                    continue; // Skip placeholder
                }
                rendered_count += 1;
            }
        }
        
        let render_duration = render_start.elapsed();
        
        // Verify we have placeholders and they're being skipped efficiently
        assert!(placeholder_count > 0, "Should have placeholders from wide characters");
        assert!(character_count > 0, "Should have actual characters");
        assert_eq!(rendered_count, terminal.rows * terminal.cols - placeholder_count);
        
        // Performance should be good even with many placeholders
        assert!(counting_duration.as_millis() < 10, "Counting took too long: {:?}", counting_duration);
        assert!(render_duration.as_millis() < 10, "Rendering took too long: {:?}", render_duration);
        
        println!("Placeholder efficiency test results:");
        println!("  Placeholders: {}", placeholder_count);
        println!("  Characters: {}", character_count);
        println!("  Rendered: {}", rendered_count);
        println!("  Counting time: {:?}", counting_duration);
        println!("  Rendering time: {:?}", render_duration);
    }

    #[test]
    fn test_large_buffer_operations() {
        // Test performance with very large terminal buffers
        let mut terminal = TerminalEmulator::new(200, 300); // Large terminal
        
        let start_time = std::time::Instant::now();
        
        // Fill the entire large buffer
        let content = "Mixed content: ABC😀DEF中GHI文JKL😁MNO字PQR🚀STU🎵VWX💻YZ";
        
        for _ in 0..200 {
            terminal.process_ansi_data(content);
            terminal.process_ansi_data("\n");
        }
        
        let fill_duration = start_time.elapsed();
        
        // Test clearing operations
        let clear_start = std::time::Instant::now();
        terminal.clear_screen();
        let clear_duration = clear_start.elapsed();
        
        // Test cursor validation on large buffer
        let validation_start = std::time::Instant::now();
        terminal.cursor_row = 1000; // Invalid position
        terminal.cursor_col = 1000; // Invalid position
        terminal.validate_cursor_position();
        let validation_duration = validation_start.elapsed();
        
        // Performance assertions for large operations
        assert!(fill_duration.as_millis() < 2000, "Large buffer fill took too long: {:?}", fill_duration);
        assert!(clear_duration.as_millis() < 100, "Large buffer clear took too long: {:?}", clear_duration);
        assert!(validation_duration.as_millis() < 10, "Cursor validation took too long: {:?}", validation_duration);
        
        // Verify buffer integrity after operations
        assert_eq!(terminal.buffer.len(), 200);
        assert_eq!(terminal.buffer[0].len(), 300);
        assert!(terminal.cursor_row < terminal.rows);
        assert!(terminal.cursor_col < terminal.cols);
        
        println!("Large buffer test results:");
        println!("  Fill time: {:?}", fill_duration);
        println!("  Clear time: {:?}", clear_duration);
        println!("  Validation time: {:?}", validation_duration);
    }

    #[test]
    fn test_dingbats_and_symbols_alignment() {
        let mut terminal = TerminalEmulator::new(3, 15);
        
        // Test with problematic dingbats and symbols that were causing misalignment
        terminal.process_ansi_data("A✅B❤C⭐D\n");
        terminal.process_ansi_data("123456789012\n");
        
        // Verify alignment: A✅B❤C⭐D should align with numbers
        // A at pos 0, ✅ at pos 1-2, B at pos 3, ❤ at pos 4-5, C at pos 6, ⭐ at pos 7-8, D at pos 9
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, '✅');
        assert_eq!(terminal.buffer[0][2].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B');
        assert_eq!(terminal.buffer[0][4].character, '❤');
        assert_eq!(terminal.buffer[0][5].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][6].character, 'C');
        assert_eq!(terminal.buffer[0][7].character, '⭐');
        assert_eq!(terminal.buffer[0][8].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][9].character, 'D');
        
        // Numbers should align perfectly
        assert_eq!(terminal.buffer[1][0].character, '1');
        assert_eq!(terminal.buffer[1][1].character, '2');
        assert_eq!(terminal.buffer[1][2].character, '3');
        assert_eq!(terminal.buffer[1][3].character, '4');
        assert_eq!(terminal.buffer[1][4].character, '5');
        assert_eq!(terminal.buffer[1][5].character, '6');
        assert_eq!(terminal.buffer[1][6].character, '7');
        assert_eq!(terminal.buffer[1][7].character, '8');
        assert_eq!(terminal.buffer[1][8].character, '9');
        assert_eq!(terminal.buffer[1][9].character, '0');
    }

    #[test]
    fn test_comprehensive_symbol_alignment() {
        let mut terminal = TerminalEmulator::new(4, 25);
        
        // Test simple alignment with problematic symbols
        terminal.process_ansi_data("A✂B\n");
        terminal.process_ansi_data("123\n");
        
        // Debug: check what's actually in the buffer
        println!("Row 0: {:?}", terminal.buffer[0].iter().take(10).map(|c| c.character).collect::<Vec<_>>());
        println!("Row 1: {:?}", terminal.buffer[1].iter().take(10).map(|c| c.character).collect::<Vec<_>>());
        
        // Verify simple alignment: A✂B should align with 123
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, '✂');
        assert_eq!(terminal.buffer[0][2].character, '\0'); // placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B');
        
        // Numbers should align
        assert_eq!(terminal.buffer[1][0].character, '1');
        assert_eq!(terminal.buffer[1][1].character, '2');
        assert_eq!(terminal.buffer[1][2].character, '3');
    }

}