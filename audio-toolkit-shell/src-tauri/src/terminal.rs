//! # Terminal Module
//! 
//! This module provides terminal emulation functionality including ANSI sequence processing,
//! character rendering, and terminal buffer management.
//! 
//! ## Features
//! 
//! - **ANSI Sequence Processing**: Full support for cursor movement, colors, and text formatting
//! - **Unicode Support**: Proper handling of wide characters and emojis
//! - **Buffer Management**: Efficient 2D character buffer with scrolling
//! - **Color Support**: 256-color ANSI support with Catppuccin theming
//! 
//! ## Usage
//! 
//! ```rust
//! use crate::terminal::{TerminalEmulator, TerminalCell};
//! 
//! // Create a new terminal emulator
//! let mut terminal = TerminalEmulator::new(24, 80);
//! 
//! // Process terminal data with ANSI sequences
//! terminal.process_ansi_data("Hello \x1b[31mWorld\x1b[0m\n");
//! 
//! // Access the terminal buffer for rendering
//! let buffer = &terminal.buffer;
//! ```

use eframe::egui;
use unicode_width::UnicodeWidthChar;
use crate::theme::{CatppuccinTheme, ansi_256_to_rgb};

/// Represents a single character cell in the terminal buffer
/// 
/// Each cell contains a character, its display color, and formatting information.
/// This is the fundamental unit of the terminal display, allowing for rich text
/// rendering with colors and formatting attributes.
/// 
/// # Fields
/// 
/// * `character` - The Unicode character to display
/// * `color` - The foreground color for the character
/// * `bold` - Whether the character should be rendered in bold
#[derive(Clone)]
pub struct TerminalCell {
    pub character: char,
    pub color: egui::Color32,
    pub bold: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            character: ' ',
            color: CatppuccinTheme::FRAPPE.text,
            bold: false,
        }
    }
}

/// Terminal emulator that handles ANSI sequences and character rendering
/// 
/// This struct manages a 2D buffer of terminal cells and processes ANSI escape sequences
/// to provide terminal-like functionality including cursor movement, colors, and text formatting.
/// 
/// The emulator supports:
/// - Cursor positioning and movement
/// - Text colors (16-color, 256-color ANSI)
/// - Text formatting (bold)
/// - Screen clearing and line clearing
/// - Unicode character support including emojis
/// - Automatic scrolling when content exceeds buffer size
#[derive(Clone)]
pub struct TerminalEmulator {
    pub buffer: Vec<Vec<TerminalCell>>,
    cursor_row: usize,
    cursor_col: usize,
    rows: usize,
    cols: usize,
    current_color: egui::Color32,
    bold: bool,
}

impl TerminalEmulator {
    /// Creates a new terminal emulator with the specified dimensions
    /// 
    /// Initializes a new terminal emulator with a blank buffer of the specified size.
    /// The cursor starts at position (0, 0) and uses the default Catppuccin text color.
    /// 
    /// # Arguments
    /// 
    /// * `rows` - Number of rows in the terminal buffer (typically 24)
    /// * `cols` - Number of columns in the terminal buffer (typically 80)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let terminal = TerminalEmulator::new(24, 80);
    /// ```
    pub fn new(rows: usize, cols: usize) -> Self {
        let buffer = vec![vec![TerminalCell::default(); cols]; rows];
        Self {
            buffer,
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
            current_color: CatppuccinTheme::FRAPPE.text,
            bold: false,
        }
    }

    /// Clears the entire terminal screen and resets cursor to top-left
    /// 
    /// Fills all cells in the buffer with default empty cells (space character
    /// with default color) and moves the cursor to position (0, 0).
    pub fn clear_screen(&mut self) {
        for row in &mut self.buffer {
            for cell in row {
                *cell = TerminalCell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Moves the cursor to the specified position
    /// 
    /// Positions are clamped to valid buffer bounds to prevent out-of-bounds access.
    /// The cursor position affects where new characters will be written.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Target row (0-based, clamped to buffer height)
    /// * `col` - Target column (0-based, clamped to buffer width)
    pub fn move_cursor(&mut self, row: usize, col: usize) {
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

    /// Processes ANSI data and updates the terminal buffer
    /// 
    /// This method parses ANSI escape sequences and regular text, updating the terminal
    /// buffer accordingly. It handles cursor movement, colors, and text formatting.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The raw terminal data containing text and ANSI sequences
    pub fn process_ansi_data(&mut self, data: &str) {
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
        // Use Catppuccin Frappé theme for ANSI color mapping
        const THEME: &CatppuccinTheme = &CatppuccinTheme::FRAPPE;
        
        if params.is_empty() || (params.len() == 1 && params[0].is_empty()) {
            // Reset to Catppuccin text color instead of white
            self.current_color = THEME.text;
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
                    // Reset to Catppuccin text color
                    self.current_color = THEME.text;
                    self.bold = false;
                }
                "1" => self.bold = true,
                "22" => self.bold = false,
                // ANSI color codes 30-37 mapped to Catppuccin Frappé colors
                "30" => self.current_color = THEME.surface1,  // Black -> surface1
                "31" => self.current_color = THEME.red,       // Red -> red
                "32" => self.current_color = THEME.green,     // Green -> green
                "33" => self.current_color = THEME.yellow,    // Yellow -> yellow
                "34" => self.current_color = THEME.blue,      // Blue -> blue
                "35" => self.current_color = THEME.mauve,     // Magenta -> mauve
                "36" => self.current_color = THEME.teal,      // Cyan -> teal
                "37" => self.current_color = THEME.text,      // White -> text
                // Bright ANSI color codes 90-97 mapped to same Catppuccin colors with surface2 for bright black
                "90" => self.current_color = THEME.surface2,  // Bright Black -> surface2
                "91" => self.current_color = THEME.red,       // Bright Red -> red
                "92" => self.current_color = THEME.green,     // Bright Green -> green
                "93" => self.current_color = THEME.yellow,    // Bright Yellow -> yellow
                "94" => self.current_color = THEME.blue,      // Bright Blue -> blue
                "95" => self.current_color = THEME.mauve,     // Bright Magenta -> mauve
                "96" => self.current_color = THEME.teal,      // Bright Cyan -> teal
                "97" => self.current_color = THEME.text,      // Bright White -> text
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_cell_default() {
        let cell = TerminalCell::default();
        assert_eq!(cell.character, ' ');
        assert_eq!(cell.color, CatppuccinTheme::FRAPPE.text);
        assert!(!cell.bold);
    }

    #[test]
    fn test_terminal_emulator_new() {
        let terminal = TerminalEmulator::new(24, 80);
        assert_eq!(terminal.buffer.len(), 24);
        assert_eq!(terminal.buffer[0].len(), 80);
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 0);
        assert_eq!(terminal.rows, 24);
        assert_eq!(terminal.cols, 80);
    }

    #[test]
    fn test_clear_screen() {
        let mut terminal = TerminalEmulator::new(3, 3);
        
        // Fill with some data
        terminal.process_ansi_data("Hello");
        
        // Clear screen
        terminal.clear_screen();
        
        // Check that all cells are default
        for row in &terminal.buffer {
            for cell in row {
                assert_eq!(cell.character, ' ');
            }
        }
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 0);
    }

    #[test]
    fn test_move_cursor() {
        let mut terminal = TerminalEmulator::new(10, 10);
        
        terminal.move_cursor(5, 7);
        assert_eq!(terminal.cursor_row, 5);
        assert_eq!(terminal.cursor_col, 7);
        
        // Test bounds clamping
        terminal.move_cursor(20, 30);
        assert_eq!(terminal.cursor_row, 9);  // Clamped to rows-1
        assert_eq!(terminal.cursor_col, 9);  // Clamped to cols-1
    }

    #[test]
    fn test_process_simple_text() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        terminal.process_ansi_data("Hello");
        
        assert_eq!(terminal.buffer[0][0].character, 'H');
        assert_eq!(terminal.buffer[0][1].character, 'e');
        assert_eq!(terminal.buffer[0][2].character, 'l');
        assert_eq!(terminal.buffer[0][3].character, 'l');
        assert_eq!(terminal.buffer[0][4].character, 'o');
        assert_eq!(terminal.cursor_col, 5);
    }

    #[test]
    fn test_process_newline() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        terminal.process_ansi_data("Hi\nThere");
        
        assert_eq!(terminal.buffer[0][0].character, 'H');
        assert_eq!(terminal.buffer[0][1].character, 'i');
        assert_eq!(terminal.buffer[1][0].character, 'T');
        assert_eq!(terminal.buffer[1][1].character, 'h');
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 5);
    }

    #[test]
    fn test_process_carriage_return() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        terminal.process_ansi_data("Hello\rWorld");
        
        // Should overwrite from beginning of line
        assert_eq!(terminal.buffer[0][0].character, 'W');
        assert_eq!(terminal.buffer[0][1].character, 'o');
        assert_eq!(terminal.buffer[0][2].character, 'r');
        assert_eq!(terminal.buffer[0][3].character, 'l');
        assert_eq!(terminal.buffer[0][4].character, 'd');
        assert_eq!(terminal.cursor_col, 5);
    }

    #[test]
    fn test_ansi_cursor_movement() {
        let mut terminal = TerminalEmulator::new(10, 10);
        
        // Move cursor to position (3, 5) - ANSI uses 1-based indexing
        terminal.process_ansi_data("\x1b[4;6H");
        assert_eq!(terminal.cursor_row, 3);
        assert_eq!(terminal.cursor_col, 5);
        
        // Move cursor up
        terminal.process_ansi_data("\x1b[2A");
        assert_eq!(terminal.cursor_row, 1);
        
        // Move cursor down
        terminal.process_ansi_data("\x1b[3B");
        assert_eq!(terminal.cursor_row, 4);
        
        // Move cursor right
        terminal.process_ansi_data("\x1b[2C");
        assert_eq!(terminal.cursor_col, 7);
        
        // Move cursor left
        terminal.process_ansi_data("\x1b[1D");
        assert_eq!(terminal.cursor_col, 6);
    }

    #[test]
    fn test_ansi_colors() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set red color and write text
        terminal.process_ansi_data("\x1b[31mRed");
        
        assert_eq!(terminal.buffer[0][0].character, 'R');
        assert_eq!(terminal.buffer[0][0].color, CatppuccinTheme::FRAPPE.red);
        assert_eq!(terminal.buffer[0][1].color, CatppuccinTheme::FRAPPE.red);
        assert_eq!(terminal.buffer[0][2].color, CatppuccinTheme::FRAPPE.red);
    }

    #[test]
    fn test_ansi_bold() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set bold and write text
        terminal.process_ansi_data("\x1b[1mBold");
        
        assert_eq!(terminal.buffer[0][0].character, 'B');
        assert!(terminal.buffer[0][0].bold);
        assert!(terminal.buffer[0][1].bold);
        assert!(terminal.buffer[0][2].bold);
        assert!(terminal.buffer[0][3].bold);
    }

    #[test]
    fn test_ansi_reset() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set red and bold, then reset
        terminal.process_ansi_data("\x1b[31;1mRed\x1b[0mNormal");
        
        assert_eq!(terminal.buffer[0][0].color, CatppuccinTheme::FRAPPE.red);
        assert!(terminal.buffer[0][0].bold);
        
        assert_eq!(terminal.buffer[0][3].color, CatppuccinTheme::FRAPPE.text);
        assert!(!terminal.buffer[0][3].bold);
    }

    #[test]
    fn test_ansi_256_color() {
        let mut terminal = TerminalEmulator::new(5, 10);
        
        // Set 256-color red (color index 196)
        terminal.process_ansi_data("\x1b[38;5;196mRed");
        
        assert_eq!(terminal.buffer[0][0].character, 'R');
        // Should use the 256-color conversion
        assert_eq!(terminal.buffer[0][0].color, ansi_256_to_rgb(196));
    }

    #[test]
    fn test_ansi_clear_screen() {
        let mut terminal = TerminalEmulator::new(3, 3);
        
        // Fill with data
        terminal.process_ansi_data("123456789");
        
        // Clear entire screen
        terminal.process_ansi_data("\x1b[2J");
        
        // Check all cells are cleared
        for row in &terminal.buffer {
            for cell in row {
                assert_eq!(cell.character, ' ');
            }
        }
    }

    #[test]
    fn test_ansi_clear_line() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Fill first line
        terminal.process_ansi_data("ABCDE");
        terminal.move_cursor(0, 2); // Move to middle of line
        
        // Clear from cursor to end of line
        terminal.process_ansi_data("\x1b[K");
        
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, 'B');
        assert_eq!(terminal.buffer[0][2].character, ' '); // Cleared
        assert_eq!(terminal.buffer[0][3].character, ' '); // Cleared
        assert_eq!(terminal.buffer[0][4].character, ' '); // Cleared
    }

    #[test]
    fn test_wide_character_handling() {
        let mut terminal = TerminalEmulator::new(3, 5);
        
        // Test emoji (should take 2 columns)
        terminal.process_ansi_data("A😀B");
        
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, '😀');
        assert_eq!(terminal.buffer[0][2].character, '\0'); // Placeholder
        assert_eq!(terminal.buffer[0][3].character, 'B');
        assert_eq!(terminal.cursor_col, 4);
    }

    #[test]
    fn test_line_wrapping() {
        let mut terminal = TerminalEmulator::new(3, 3);
        
        // Write more than line width
        terminal.process_ansi_data("ABCDEF");
        
        // Should wrap to next line
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, 'B');
        assert_eq!(terminal.buffer[0][2].character, 'C');
        assert_eq!(terminal.buffer[1][0].character, 'D');
        assert_eq!(terminal.buffer[1][1].character, 'E');
        assert_eq!(terminal.buffer[1][2].character, 'F');
    }

    #[test]
    fn test_scrolling() {
        let mut terminal = TerminalEmulator::new(2, 3);
        
        // Fill the terminal with more content than it can hold
        terminal.process_ansi_data("ABC\nDEF\nGHI");
        
        // The terminal should have scrolled, so the first line "ABC" should be gone
        // and we should have "DEF" on the first line and "GHI" on the second line
        assert_eq!(terminal.buffer[0][0].character, 'D');
        assert_eq!(terminal.buffer[0][1].character, 'E');
        assert_eq!(terminal.buffer[0][2].character, 'F');
        assert_eq!(terminal.buffer[1][0].character, 'G');
        assert_eq!(terminal.buffer[1][1].character, 'H');
        assert_eq!(terminal.buffer[1][2].character, 'I');
        
        // Cursor should be at the end of the last line
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 3);
    }

    #[test]
    fn test_tab_handling() {
        let mut terminal = TerminalEmulator::new(3, 16);
        
        terminal.process_ansi_data("A\tB");
        
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][8].character, 'B'); // Tab stops at column 8
    }

    #[test]
    fn test_get_char_width() {
        assert_eq!(TerminalEmulator::get_char_width('A'), 1);
        assert_eq!(TerminalEmulator::get_char_width('中'), 2); // CJK character
        assert_eq!(TerminalEmulator::get_char_width('😀'), 2); // Emoji
        assert_eq!(TerminalEmulator::get_char_width('│'), 1); // Box drawing
    }

    #[test]
    fn test_is_emoji_char() {
        assert!(TerminalEmulator::is_emoji_char('😀')); // Emoticon
        assert!(TerminalEmulator::is_emoji_char('🚀')); // Transport symbol
        assert!(TerminalEmulator::is_emoji_char('⭐')); // Star symbol
        assert!(TerminalEmulator::is_emoji_char('✅')); // Check mark
        assert!(!TerminalEmulator::is_emoji_char('A')); // Regular ASCII
        assert!(!TerminalEmulator::is_emoji_char('中')); // CJK but not emoji
    }

    #[test]
    fn test_bounds_safety() {
        let mut terminal = TerminalEmulator::new(2, 2);
        
        // Try to move cursor out of bounds
        terminal.move_cursor(10, 10);
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 1);
        
        // Try ANSI movement out of bounds
        terminal.process_ansi_data("\x1b[100A"); // Move up 100 lines
        assert_eq!(terminal.cursor_row, 0);
        
        terminal.process_ansi_data("\x1b[100C"); // Move right 100 columns
        assert_eq!(terminal.cursor_col, 1);
    }
}