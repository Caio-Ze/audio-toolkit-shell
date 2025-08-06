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

/// Represents different states of ANSI parameters during parsing
/// 
/// This enum helps handle edge cases in ANSI parameter parsing by distinguishing
/// between valid values, empty parameters, and invalid parameters.
#[derive(Debug, Clone, PartialEq)]
enum AnsiParameter {
    /// Valid numeric parameter value
    Value(usize),
    /// Empty parameter (e.g., in `\x1b[;5H` the first parameter is empty)
    Empty(usize), // index for debugging
    /// Invalid parameter (non-numeric or out of bounds)
    Invalid(usize), // index for debugging
}

/// State machine for atomic ANSI sequence processing
/// 
/// This enum tracks the current state of ANSI sequence parsing to ensure
/// that sequences are processed atomically and prevent race conditions
/// between cursor positioning and text writing.
#[derive(Debug, Clone, PartialEq)]
enum AnsiState {
    /// Normal text processing state
    Normal,
    /// Escape character detected, waiting for sequence type
    Escape,
    /// CSI sequence detected (ESC[), accumulating parameters
    CsiSequence,
    /// Complete sequence ready for atomic processing
    SequenceComplete,
}

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
    /// Flag to track if cursor was recently positioned, indicating potential need for clearing
    cursor_recently_positioned: bool,
    /// Buffer for accumulating partial ANSI sequences to ensure atomic processing
    ansi_sequence_buffer: String,
    /// State machine for ANSI sequence processing
    ansi_state: AnsiState,
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
            cursor_recently_positioned: false,
            ansi_sequence_buffer: String::new(),
            ansi_state: AnsiState::Normal,
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
        
        // Clamp cursor position to valid bounds - ensure we don't exceed buffer
        let new_row = row.min(self.rows.saturating_sub(1));
        let new_col = col.min(self.cols.saturating_sub(1));
        
        // Only update if the position is actually different to avoid unnecessary work
        if self.cursor_row != new_row || self.cursor_col != new_col {
            self.cursor_row = new_row;
            self.cursor_col = new_col;
        }
        
        // Additional validation to ensure cursor position is within buffer bounds
        self.validate_cursor_position();
    }

    /// Moves the cursor to the specified position and clears the target area
    /// 
    /// This method is specifically designed to prevent text contamination by clearing
    /// the target area before positioning the cursor. This helps prevent issues where
    /// old text remains visible when new text is written over it.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Target row (0-based, clamped to buffer height)
    /// * `col` - Target column (0-based, clamped to buffer width)
    /// * `clear_length` - Number of characters to clear from the cursor position
    pub fn move_cursor_and_clear(&mut self, row: usize, col: usize, clear_length: usize) {
        // First move the cursor to the target position
        self.move_cursor(row, col);
        
        // Clear the target area to prevent text contamination
        self.clear_cursor_area(clear_length);
        
        // Mark that cursor was recently positioned for potential additional clearing
        self.cursor_recently_positioned = true;
    }

    /// Clears a specified number of characters from the current cursor position
    /// 
    /// This method helps prevent text contamination by clearing cells that will
    /// be overwritten with new content. It ensures a clean slate for new text.
    /// 
    /// # Arguments
    /// 
    /// * `length` - Number of characters to clear from the cursor position
    fn clear_cursor_area(&mut self, length: usize) {
        if length == 0 || self.cursor_row >= self.rows {
            return;
        }
        
        // Get the current row and clear the specified number of cells
        if let Some(row) = self.buffer.get_mut(self.cursor_row) {
            let start_col = self.cursor_col;
            let end_col = (start_col + length).min(self.cols);
            
            for col in start_col..end_col {
                if let Some(cell) = row.get_mut(col) {
                    *cell = TerminalCell::default();
                }
            }
        }
    }

    /// Writes a character with proactive buffer clearing
    /// 
    /// This method writes a character and clears a few cells ahead to prevent
    /// text contamination from previous content. This is particularly useful
    /// when writing text that might overlap with existing content.
    /// 
    /// # Arguments
    /// 
    /// * `ch` - The character to write
    /// * `clear_ahead` - Number of additional characters to clear ahead
    fn write_char_with_clearing(&mut self, ch: char, clear_ahead: usize) {
        // Clear the area ahead before writing the character
        if clear_ahead > 0 {
            self.clear_cursor_area(clear_ahead + 1);
        }
        
        // Write the character normally
        self.write_char(ch);
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
        
        // Skip null characters that might be causing issues
        if ch == '\0' {
            return;
        }
        
        // Handle malformed input gracefully
        if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
            // Skip most control characters to maintain buffer integrity
            return;
        }
        
        // If cursor was recently positioned, clear additional area to prevent contamination
        if self.cursor_recently_positioned && !ch.is_whitespace() {
            // Clear more characters ahead when writing the first character after positioning
            // With atomic processing, we can be more aggressive about clearing
            self.clear_cursor_area(10);
            self.cursor_recently_positioned = false;
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
                    // Always overwrite the cell completely
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
        
        // Reset cursor positioning flag on newline
        self.cursor_recently_positioned = false;
        
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
        // Reset cursor positioning flag on carriage return
        self.cursor_recently_positioned = false;
    }

    /// Processes ANSI data and updates the terminal buffer atomically
    /// 
    /// This method uses a state machine to ensure ANSI sequences are processed
    /// atomically, preventing race conditions between cursor positioning and text writing.
    /// Complete sequences are accumulated before being processed as single operations.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The raw terminal data containing text and ANSI sequences
    pub fn process_ansi_data(&mut self, data: &str) {
        // Ensure we have valid data to process
        if data.is_empty() {
            return;
        }
        
        for ch in data.chars() {
            self.process_char_atomic(ch);
        }
    }

    /// Processes a single character through the atomic ANSI state machine
    /// 
    /// This method implements a proper state machine for ANSI sequence processing
    /// to ensure atomic operations and prevent partial sequence processing.
    /// 
    /// # Arguments
    /// 
    /// * `ch` - The character to process
    fn process_char_atomic(&mut self, ch: char) {
        match self.ansi_state {
            AnsiState::Normal => {
                if ch == '\u{1b}' {
                    // Start of escape sequence
                    self.ansi_state = AnsiState::Escape;
                    self.ansi_sequence_buffer.clear();
                } else if ch == '\n' {
                    self.handle_newline();
                } else if ch == '\r' {
                    self.handle_carriage_return();
                } else if ch == '\t' {
                    // Handle tab - move to next tab stop (every 8 characters)
                    let next_tab = ((self.cursor_col / 8) + 1) * 8;
                    self.cursor_col = next_tab.min(self.cols - 1);
                } else if ch.is_control() {
                    // Skip other control characters
                } else {
                    self.write_char(ch);
                }
            }
            AnsiState::Escape => {
                if ch == '[' {
                    // CSI sequence (Control Sequence Introducer)
                    self.ansi_state = AnsiState::CsiSequence;
                    self.ansi_sequence_buffer.clear();
                } else {
                    // Other escape sequences - treat as normal character for now
                    self.ansi_state = AnsiState::Normal;
                    self.write_char('\u{1b}');
                    self.write_char(ch);
                }
            }
            AnsiState::CsiSequence => {
                if ch.is_ascii_alphabetic() || "~".contains(ch) {
                    // Sequence terminator found - complete sequence
                    self.ansi_sequence_buffer.push(ch);
                    self.ansi_state = AnsiState::SequenceComplete;
                    self.process_complete_ansi_sequence();
                } else if ch.is_ascii_digit() || ch == ';' || ch == '?' {
                    // Valid sequence parameter character
                    self.ansi_sequence_buffer.push(ch);
                } else {
                    // Invalid character - abort sequence and treat as normal text
                    self.ansi_state = AnsiState::Normal;
                    self.write_char('\u{1b}');
                    self.write_char('[');
                    // Clone the buffer to avoid borrowing issues
                    let buffer_copy = self.ansi_sequence_buffer.clone();
                    for seq_ch in buffer_copy.chars() {
                        self.write_char(seq_ch);
                    }
                    self.write_char(ch);
                    self.ansi_sequence_buffer.clear();
                }
            }
            AnsiState::SequenceComplete => {
                // This state should not be reached as we immediately process and reset
                self.ansi_state = AnsiState::Normal;
                self.process_char_atomic(ch);
            }
        }
    }

    /// Processes a complete ANSI sequence atomically
    /// 
    /// This method handles complete ANSI sequences as single atomic operations,
    /// ensuring that cursor positioning and any related operations happen together
    /// without interference from other operations.
    fn process_complete_ansi_sequence(&mut self) {
        // Reset state first
        self.ansi_state = AnsiState::Normal;
        
        // Process the complete sequence atomically
        if !self.ansi_sequence_buffer.is_empty() {
            // Clone the buffer to avoid borrowing issues
            let sequence = self.ansi_sequence_buffer.clone();
            self.handle_ansi_sequence(&sequence);
        }
        
        // Clear the buffer for next sequence
        self.ansi_sequence_buffer.clear();
    }

    /// Writes text atomically with enhanced contamination prevention
    /// 
    /// This method writes a string of text as an atomic operation, ensuring
    /// that if cursor was recently positioned, the entire text area is cleared
    /// before writing to prevent any contamination.
    /// 
    /// # Arguments
    /// 
    /// * `text` - The text to write atomically
    pub fn write_text_atomic(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        
        // If cursor was recently positioned, clear the entire text area first
        if self.cursor_recently_positioned {
            self.clear_cursor_area(text.len() + 5); // Clear text length + buffer
            self.cursor_recently_positioned = false;
        }
        
        // Write each character
        for ch in text.chars() {
            self.write_char(ch);
        }
    }

    fn handle_ansi_sequence(&mut self, sequence: &str) {
        if sequence.is_empty() {
            return;
        }
        
        // Validate sequence format
        if sequence.len() > 100 {
            // Reject extremely long sequences that might be malformed
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
        
        // Enhanced parameter parsing with proper edge case handling
        let params = self.parse_ansi_parameters(param_str);

        match cmd {
            'H' | 'f' => {
                // Cursor position - ANSI coordinates are 1-based, convert to 0-based
                // Handle missing/empty parameters with proper defaults (1 in ANSI spec)
                let ansi_row = self.get_ansi_param_value(&params, 0, 1);
                let ansi_col = self.get_ansi_param_value(&params, 1, 1);
                
                // Convert from 1-based ANSI to 0-based internal coordinates
                let row = ansi_row.saturating_sub(1);
                let col = ansi_col.saturating_sub(1);
                
                // Clamp coordinates to valid buffer bounds (ANSI standard behavior)
                let clamped_row = row.min(self.rows.saturating_sub(1));
                let clamped_col = col.min(self.cols.saturating_sub(1));
                
                // Use buffer clearing cursor positioning to prevent text contamination
                // Clear a larger area (30 characters) since we now have atomic processing
                self.move_cursor_and_clear(clamped_row, clamped_col, 30);
            }
            'A' => {
                // Cursor up with bounds checking and proper parameter handling
                let count = self.get_ansi_param_value(&params, 0, 1);
                self.cursor_row = self.cursor_row.saturating_sub(count);
                self.validate_cursor_position();
            }
            'B' => {
                // Cursor down with bounds checking and proper parameter handling
                let count = self.get_ansi_param_value(&params, 0, 1);
                if self.rows > 0 {
                    self.cursor_row = (self.cursor_row.saturating_add(count)).min(self.rows - 1);
                }
                self.validate_cursor_position();
            }
            'C' => {
                // Cursor forward with bounds checking and proper parameter handling
                let count = self.get_ansi_param_value(&params, 0, 1);
                if self.cols > 0 {
                    self.cursor_col = (self.cursor_col.saturating_add(count)).min(self.cols - 1);
                }
                self.validate_cursor_position();
            }
            'D' => {
                // Cursor backward with bounds checking and proper parameter handling
                let count = self.get_ansi_param_value(&params, 0, 1);
                self.cursor_col = self.cursor_col.saturating_sub(count);
                self.validate_cursor_position();
            }
            'J' => {
                // Clear screen with bounds checking and proper parameter handling
                let mode = self.get_ansi_param_value(&params, 0, 0);
                
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
                // Clear line with bounds checking and proper parameter handling
                let mode = self.get_ansi_param_value(&params, 0, 0);
                
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

    /// Parse ANSI parameters with proper edge case handling
    /// 
    /// This method handles various edge cases in ANSI parameter parsing:
    /// - Missing parameters (e.g., `\x1b[10;H` missing column parameter)
    /// - Empty parameters (e.g., `\x1b[;5H` empty row parameter)
    /// - Malformed parameters (non-numeric values)
    /// - Parameter validation and bounds checking
    /// 
    /// # Arguments
    /// 
    /// * `param_str` - The parameter string from the ANSI sequence (without command)
    /// 
    /// # Returns
    /// 
    /// A vector of parsed parameters with proper defaults for missing/empty values
    fn parse_ansi_parameters(&self, param_str: &str) -> Vec<AnsiParameter> {
        if param_str.is_empty() {
            return vec![];
        }
        
        // Split by semicolon and handle each parameter
        let raw_params: Vec<&str> = param_str.split(';').collect();
        let mut parsed_params = Vec::with_capacity(raw_params.len());
        
        for (index, raw_param) in raw_params.iter().enumerate() {
            let trimmed = raw_param.trim();
            
            if trimmed.is_empty() {
                // Empty parameter - use default value based on context
                parsed_params.push(AnsiParameter::Empty(index));
            } else if let Ok(value) = trimmed.parse::<usize>() {
                // Valid numeric parameter with bounds checking
                if value <= 10000 { // Reasonable upper bound to prevent overflow
                    parsed_params.push(AnsiParameter::Value(value));
                } else {
                    // Value too large, treat as invalid
                    parsed_params.push(AnsiParameter::Invalid(index));
                }
            } else {
                // Non-numeric parameter, mark as invalid
                parsed_params.push(AnsiParameter::Invalid(index));
            }
        }
        
        parsed_params
    }
    
    /// Get parameter value with proper default handling
    /// 
    /// This method extracts parameter values with appropriate defaults based on
    /// the ANSI command context and parameter position.
    /// 
    /// # Arguments
    /// 
    /// * `params` - Vector of parsed ANSI parameters
    /// * `index` - Parameter index to retrieve
    /// * `default_value` - Default value for missing/empty/invalid parameters
    /// 
    /// # Returns
    /// 
    /// The parameter value or default if missing/empty/invalid
    fn get_ansi_param_value(&self, params: &[AnsiParameter], index: usize, default_value: usize) -> usize {
        match params.get(index) {
            Some(AnsiParameter::Value(value)) => *value,
            Some(AnsiParameter::Empty(_)) | Some(AnsiParameter::Invalid(_)) | None => default_value,
        }
    }

    fn handle_graphics_mode(&mut self, params: &[AnsiParameter]) {
        // Use Catppuccin Frappé theme for ANSI color mapping
        const THEME: &CatppuccinTheme = &CatppuccinTheme::FRAPPE;
        
        // Handle empty parameters or single empty parameter (reset case)
        if params.is_empty() || (params.len() == 1 && matches!(params[0], AnsiParameter::Empty(_))) {
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
            
            // Get parameter value with proper handling of empty/invalid parameters
            let param_value = match params.get(i) {
                Some(AnsiParameter::Value(val)) => *val,
                Some(AnsiParameter::Empty(_)) => 0, // Empty parameter defaults to 0 (reset)
                Some(AnsiParameter::Invalid(_)) => {
                    i += 1;
                    continue; // Skip invalid parameters
                }
                None => break,
            };
            
            match param_value {
                0 => {
                    // Reset to Catppuccin text color
                    self.current_color = THEME.text;
                    self.bold = false;
                }
                1 => self.bold = true,
                22 => self.bold = false,
                // ANSI color codes 30-37 mapped to Catppuccin Frappé colors
                30 => self.current_color = THEME.surface1,  // Black -> surface1
                31 => self.current_color = THEME.red,       // Red -> red
                32 => self.current_color = THEME.green,     // Green -> green
                33 => self.current_color = THEME.yellow,    // Yellow -> yellow
                34 => self.current_color = THEME.blue,      // Blue -> blue
                35 => self.current_color = THEME.mauve,     // Magenta -> mauve
                36 => self.current_color = THEME.teal,      // Cyan -> teal
                37 => self.current_color = THEME.text,      // White -> text
                // Bright ANSI color codes 90-97 mapped to same Catppuccin colors with surface2 for bright black
                90 => self.current_color = THEME.surface2,  // Bright Black -> surface2
                91 => self.current_color = THEME.red,       // Bright Red -> red
                92 => self.current_color = THEME.green,     // Bright Green -> green
                93 => self.current_color = THEME.yellow,    // Bright Yellow -> yellow
                94 => self.current_color = THEME.blue,      // Bright Blue -> blue
                95 => self.current_color = THEME.mauve,     // Bright Magenta -> mauve
                96 => self.current_color = THEME.teal,      // Bright Cyan -> teal
                97 => self.current_color = THEME.text,      // Bright White -> text
                38 => {
                    // 256-color foreground with bounds checking and proper parameter handling
                    if i + 2 < params.len() && i + 2 < 100 { // Additional bounds check
                        // Check if next parameter is "5" (256-color mode indicator)
                        let mode_param = match params.get(i + 1) {
                            Some(AnsiParameter::Value(5)) => true,
                            _ => false,
                        };
                        
                        if mode_param {
                            // Get color index parameter
                            if let Some(AnsiParameter::Value(color_index)) = params.get(i + 2) {
                                if *color_index <= 255 {
                                    self.current_color = ansi_256_to_rgb(*color_index as u8);
                                }
                            }
                            i += 2; // Skip the next two parameters
                        }
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
    }

    #[test]
    fn test_ansi_parameter_parsing_edge_cases() {
        let mut terminal = TerminalEmulator::new(24, 80);
        
        // Test empty parameters
        let params = terminal.parse_ansi_parameters("");
        assert!(params.is_empty());
        
        // Test single empty parameter
        let params = terminal.parse_ansi_parameters(";");
        assert_eq!(params.len(), 2);
        assert!(matches!(params[0], AnsiParameter::Empty(0)));
        assert!(matches!(params[1], AnsiParameter::Empty(1)));
        
        // Test missing parameter (e.g., "10;" missing second parameter)
        let params = terminal.parse_ansi_parameters("10;");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], AnsiParameter::Value(10));
        assert!(matches!(params[1], AnsiParameter::Empty(1)));
        
        // Test empty first parameter (e.g., ";5")
        let params = terminal.parse_ansi_parameters(";5");
        assert_eq!(params.len(), 2);
        assert!(matches!(params[0], AnsiParameter::Empty(0)));
        assert_eq!(params[1], AnsiParameter::Value(5));
        
        // Test invalid parameters
        let params = terminal.parse_ansi_parameters("abc;5;xyz");
        assert_eq!(params.len(), 3);
        assert!(matches!(params[0], AnsiParameter::Invalid(0)));
        assert_eq!(params[1], AnsiParameter::Value(5));
        assert!(matches!(params[2], AnsiParameter::Invalid(2)));
        
        // Test parameter bounds checking (too large values)
        let params = terminal.parse_ansi_parameters("99999");
        assert_eq!(params.len(), 1);
        assert!(matches!(params[0], AnsiParameter::Invalid(0)));
        
        // Test valid parameters
        let params = terminal.parse_ansi_parameters("1;2;3");
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], AnsiParameter::Value(1));
        assert_eq!(params[1], AnsiParameter::Value(2));
        assert_eq!(params[2], AnsiParameter::Value(3));
    }

    #[test]
    fn test_cursor_positioning_edge_cases() {
        let mut terminal = TerminalEmulator::new(24, 80);
        
        // Test missing column parameter (should default to 1,1 -> 0,0)
        terminal.process_ansi_data("\x1b[10;H");
        assert_eq!(terminal.cursor_row, 9); // 10-1 = 9
        assert_eq!(terminal.cursor_col, 0); // missing col defaults to 1, 1-1 = 0
        
        // Test missing row parameter (should default to 1,1 -> 0,0)
        terminal.process_ansi_data("\x1b[;5H");
        assert_eq!(terminal.cursor_row, 0); // empty row defaults to 1, 1-1 = 0
        assert_eq!(terminal.cursor_col, 4); // 5-1 = 4
        
        // Test both parameters missing (should default to 1,1 -> 0,0)
        terminal.process_ansi_data("\x1b[H");
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 0);
        
        // Test empty parameters (should default to 1,1 -> 0,0)
        terminal.process_ansi_data("\x1b[;H");
        assert_eq!(terminal.cursor_row, 0);
        assert_eq!(terminal.cursor_col, 0);
        
        // Test bounds checking - position beyond buffer should be clamped
        terminal.process_ansi_data("\x1b[100;200H");
        assert_eq!(terminal.cursor_row, 23); // Clamped to max row (24-1)
        assert_eq!(terminal.cursor_col, 79); // Clamped to max col (80-1)
    }

    #[test]
    fn test_cursor_movement_edge_cases() {
        let mut terminal = TerminalEmulator::new(24, 80);
        terminal.move_cursor(10, 10); // Start at middle position
        
        // Test cursor up with missing parameter (should default to 1)
        terminal.process_ansi_data("\x1b[A");
        assert_eq!(terminal.cursor_row, 9);
        
        // Reset position for next test
        terminal.move_cursor(10, 10);
        
        // Test cursor up with empty parameter (should default to 1)
        terminal.process_ansi_data("\x1b[;A");
        assert_eq!(terminal.cursor_row, 9); // 10 - 1 = 9
        
        // Test cursor movement with invalid parameter (sequence should be ignored)
        // Note: \x1b[abcA is not a valid ANSI sequence and will be ignored entirely
        terminal.process_ansi_data("\x1b[abcA");
        assert_eq!(terminal.cursor_row, 9); // Should remain unchanged
        
        // Test bounds checking - moving beyond buffer bounds
        terminal.move_cursor(0, 0);
        terminal.process_ansi_data("\x1b[5A"); // Try to move up from top
        assert_eq!(terminal.cursor_row, 0); // Should stay at 0
        
        terminal.move_cursor(23, 79);
        terminal.process_ansi_data("\x1b[5B"); // Try to move down from bottom
        assert_eq!(terminal.cursor_row, 23); // Should stay at bottom
    }

    #[test]
    fn test_graphics_mode_edge_cases() {
        let mut terminal = TerminalEmulator::new(24, 80);
        
        // Test empty graphics mode (should reset)
        terminal.bold = true;
        terminal.process_ansi_data("\x1b[m");
        assert!(!terminal.bold);
        assert_eq!(terminal.current_color, CatppuccinTheme::FRAPPE.text);
        
        // Test empty parameter in graphics mode (should reset)
        terminal.bold = true;
        terminal.process_ansi_data("\x1b[;m");
        assert!(!terminal.bold);
        
        // Test invalid parameter in graphics mode (should be ignored)
        let original_color = terminal.current_color;
        terminal.process_ansi_data("\x1b[abcm");
        assert_eq!(terminal.current_color, original_color);
        
        // Test 256-color with missing parameters
        terminal.process_ansi_data("\x1b[38m"); // Missing color mode and index
        // Should not crash and should ignore the incomplete sequence
        
        // Test 256-color with partial parameters
        terminal.process_ansi_data("\x1b[38;5m"); // Missing color index
        // Should not crash and should ignore the incomplete sequence
    }

    #[test]
    fn test_buffer_clearing_on_cursor_positioning() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Fill the buffer with some initial content
        terminal.process_ansi_data("OLDTEXT123");
        
        // Move cursor to beginning and write new content
        terminal.process_ansi_data("\x1b[1;1HNEW");
        
        // The old text should be cleared where new text was written
        assert_eq!(terminal.buffer[0][0].character, 'N');
        assert_eq!(terminal.buffer[0][1].character, 'E');
        assert_eq!(terminal.buffer[0][2].character, 'W');
        
        // The area that was cleared should be empty (spaces)
        assert_eq!(terminal.buffer[0][3].character, ' ');
        assert_eq!(terminal.buffer[0][4].character, ' ');
    }

    #[test]
    fn test_cursor_recently_positioned_flag() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Initially flag should be false
        assert!(!terminal.cursor_recently_positioned);
        
        // After cursor positioning, flag should be true
        terminal.move_cursor_and_clear(1, 1, 5);
        assert!(terminal.cursor_recently_positioned);
        
        // After writing a character, flag should be reset
        terminal.write_char('A');
        assert!(!terminal.cursor_recently_positioned);
    }

    #[test]
    fn test_clear_cursor_area() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Fill a row with content
        terminal.process_ansi_data("ABCDEFGHIJ");
        
        // Move cursor to position 2 and clear 3 characters
        terminal.move_cursor(0, 2);
        terminal.clear_cursor_area(3);
        
        // Check that the specified area was cleared
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, 'B');
        assert_eq!(terminal.buffer[0][2].character, ' '); // Cleared
        assert_eq!(terminal.buffer[0][3].character, ' '); // Cleared
        assert_eq!(terminal.buffer[0][4].character, ' '); // Cleared
        assert_eq!(terminal.buffer[0][5].character, 'F');
    }

    #[test]
    fn test_text_contamination_prevention() {
        let mut terminal = TerminalEmulator::new(3, 20);
        
        // Simulate the contamination scenario
        // First write some text that might contaminate
        terminal.process_ansi_data("MONITORING.app");
        
        // Move cursor to a different position and write status text
        terminal.process_ansi_data("\x1b[2;1HStatus: ");
        
        // The status line should not contain contamination from the previous text
        let status_line = &terminal.buffer[1];
        let status_text: String = status_line.iter()
            .take(8)
            .map(|cell| cell.character)
            .collect();
        
        assert_eq!(status_text, "Status: ");
        
        // Verify no contamination characters are present
        for cell in status_line.iter().take(8) {
            assert_ne!(cell.character, 'M');
            assert_ne!(cell.character, 'O');
            assert_ne!(cell.character, 'N');
        }
    }

    #[test]
    fn test_atomic_ansi_sequence_processing() {
        let mut terminal = TerminalEmulator::new(3, 20);
        
        // Test that ANSI sequences are processed atomically
        // This should position cursor and clear area in one atomic operation
        terminal.process_ansi_data("\x1b[2;5HTest");
        
        // Verify cursor was positioned correctly
        assert_eq!(terminal.cursor_row, 1); // 2-1 = 1
        assert_eq!(terminal.cursor_col, 8); // 5-1+4 = 8 (after writing "Test")
        
        // Verify text was written correctly
        assert_eq!(terminal.buffer[1][4].character, 'T');
        assert_eq!(terminal.buffer[1][5].character, 'e');
        assert_eq!(terminal.buffer[1][6].character, 's');
        assert_eq!(terminal.buffer[1][7].character, 't');
    }

    #[test]
    fn test_ansi_state_machine() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Initially should be in Normal state
        assert_eq!(terminal.ansi_state, AnsiState::Normal);
        
        // Process escape character
        terminal.process_char_atomic('\u{1b}');
        assert_eq!(terminal.ansi_state, AnsiState::Escape);
        
        // Process CSI introducer
        terminal.process_char_atomic('[');
        assert_eq!(terminal.ansi_state, AnsiState::CsiSequence);
        
        // Process parameters
        terminal.process_char_atomic('1');
        terminal.process_char_atomic(';');
        terminal.process_char_atomic('1');
        assert_eq!(terminal.ansi_state, AnsiState::CsiSequence);
        
        // Process terminator - should complete and reset to Normal
        terminal.process_char_atomic('H');
        assert_eq!(terminal.ansi_state, AnsiState::Normal);
        assert!(terminal.ansi_sequence_buffer.is_empty());
    }

    #[test]
    fn test_partial_ansi_sequence_handling() {
        let mut terminal = TerminalEmulator::new(3, 10);
        
        // Test that ANSI sequences are processed correctly with text
        terminal.process_ansi_data("ABC\x1b[2;1HDEF");
        
        // First text should be written normally on first line
        assert_eq!(terminal.buffer[0][0].character, 'A');
        assert_eq!(terminal.buffer[0][1].character, 'B');
        assert_eq!(terminal.buffer[0][2].character, 'C');
        
        // After cursor positioning to line 2, text should be on second line
        assert_eq!(terminal.buffer[1][0].character, 'D');
        assert_eq!(terminal.buffer[1][1].character, 'E');
        assert_eq!(terminal.buffer[1][2].character, 'F');
    }

    #[test]
    fn test_atomic_text_writing() {
        let mut terminal = TerminalEmulator::new(3, 20);
        
        // Fill with contaminating text
        terminal.process_ansi_data("CONTAMINATION");
        
        // Position cursor and write text atomically
        terminal.move_cursor_and_clear(1, 0, 15);
        terminal.write_text_atomic("CLEAN TEXT");
        
        // Verify the text was written cleanly without contamination
        let clean_line = &terminal.buffer[1];
        let clean_text: String = clean_line.iter()
            .take(10)
            .map(|cell| cell.character)
            .collect();
        
        assert_eq!(clean_text, "CLEAN TEXT");
        
        // Verify that the line doesn't contain the contaminating word "CONTAMINATION"
        let full_line: String = clean_line.iter()
            .take(20)
            .map(|cell| cell.character)
            .collect();
        
        assert!(!full_line.contains("CONTAMINATION"));
        assert!(full_line.contains("CLEAN TEXT"));
    }

    #[test]
    fn test_enhanced_contamination_prevention() {
        let mut terminal = TerminalEmulator::new(3, 30);
        
        // Simulate the exact contamination scenario from the bug report
        terminal.process_ansi_data("WINDOWN.app");
        
        // Position cursor to write status (this should clear aggressively)
        terminal.process_ansi_data("\x1b[2;1HStatus: MONITORING");
        
        // Check that status line is exactly what we expect
        let status_line = &terminal.buffer[1];
        let status_text: String = status_line.iter()
            .take(19)
            .map(|cell| cell.character)
            .collect();
        
        // The key test: the text should be exactly "Status: MONITORING " without contamination
        assert_eq!(status_text, "Status: MONITORING ");
        
        // More specific contamination test: check that we don't have the contaminated patterns
        // from the original bug report like "MONITORINGOWN.app" or "MONITORING WN.app"
        let full_line: String = status_line.iter()
            .take(30)
            .map(|cell| cell.character)
            .collect();
        
        // Should not contain contaminated patterns
        assert!(!full_line.contains("OWN.app"));
        assert!(!full_line.contains("WN.app"));
        assert!(!full_line.contains("WINDOWN"));
        
        // Should contain the correct text
        assert!(full_line.contains("Status: MONITORING"));
    }

    #[test]
    fn test_terminal_emulator_basic_properties() {
        let terminal = TerminalEmulator::new(24, 80);
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
        
        // The terminal should have scrolled twice, so "ABC" and "DEF" should be gone
        // and we should have "GHI" on the first line and empty second line
        assert_eq!(terminal.buffer[0][0].character, 'G');
        assert_eq!(terminal.buffer[0][1].character, 'H');
        assert_eq!(terminal.buffer[0][2].character, 'I');
        assert_eq!(terminal.buffer[1][0].character, ' ');
        assert_eq!(terminal.buffer[1][1].character, ' ');
        assert_eq!(terminal.buffer[1][2].character, ' ');
        
        // Cursor should be at the beginning of the second line after the last newline
        assert_eq!(terminal.cursor_row, 1);
        assert_eq!(terminal.cursor_col, 0);
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