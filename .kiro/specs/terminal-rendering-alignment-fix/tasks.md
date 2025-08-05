# Implementation Plan

- [x] 1. Add unicode-width dependency to project
  - Add `unicode-width = "0.1"` to Cargo.toml dependencies section
  - Verify the dependency builds correctly with `cargo build`
  - _Requirements: 4.1, 4.2_

- [x] 2. Import unicode-width and implement character width detection
  - Add `use unicode_width::UnicodeWidthChar;` import to main.rs
  - Create helper function `get_char_width(ch: char) -> usize` that uses `ch.width().unwrap_or(1)`
  - Write unit tests to verify width detection for ASCII, emojis, and box-drawing characters
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 3. Implement width-aware cursor advancement in write_char method
  - Modify `TerminalEmulator::write_char` to calculate character width using the helper function
  - Update cursor advancement logic to use `self.cursor_col += width` instead of `+= 1`
  - Add boundary checking that considers character width before writing
  - _Requirements: 2.1, 2.2, 5.1, 5.2_

- [x] 4. Implement line wrapping logic for wide characters
  - Add check if character width would exceed line boundary (`self.cursor_col + width > self.cols`)
  - If character doesn't fit, call `self.handle_newline()` before writing the character
  - Ensure wide characters are never split across lines
  - _Requirements: 1.4, 2.3, 5.3_

- [x] 5. Implement placeholder cell creation for wide characters
  - After writing a wide character (width == 2), create placeholder cell at next position
  - Set placeholder cell with `character: '\0'`, `color: egui::Color32::TRANSPARENT`, `bold: false`
  - Add bounds checking to ensure placeholder doesn't exceed buffer limits
  - _Requirements: 2.4, 3.2_

- [x] 6. Update rendering logic to skip placeholder characters
  - Modify `AudioToolkitApp::render_row` method to check for `cell.character == '\0'`
  - Add `continue` statement to skip rendering placeholder cells
  - Verify that skipping placeholders doesn't affect horizontal spacing
  - _Requirements: 3.1, 3.2, 3.3_

- [X] 7. Handle cursor wrapping with width-aware logic
  - Update line wrapping condition to use `self.cursor_col >= self.cols` after width advancement
  - Ensure cursor position remains valid after wide character placement
  - Test cursor wrapping with consecutive wide characters
  - _Requirements: 5.2, 5.3, 5.4_

- [x] 8. Add comprehensive unit tests for width handling
  - Test character width detection for various Unicode categories
  - Test cursor advancement with mixed normal and wide characters
  - Test line wrapping behavior with wide characters at boundaries
  - Test placeholder cell creation and properties
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4_

- [x] 9. Test integration with existing ANSI processing
  - Verify wide characters work correctly with color codes
  - Test cursor positioning commands with wide characters present
  - Ensure screen clearing operations handle placeholders correctly
  - Test that existing terminal functionality remains intact
  - _Requirements: 1.1, 1.2, 1.3, 4.4_

- [x] 10. Perform visual testing and validation
  - Test terminal display with emoji characters to verify alignment
  - Test box-drawing characters for proper alignment
  - Verify cursor position matches visual layout after wide characters
  - Test text selection behavior with wide characters and placeholders
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.3_

- [x] 11. Add error handling and edge case protection
  - Add bounds checking for all buffer operations involving width
  - Handle cases where unicode-width returns None gracefully
  - Ensure buffer integrity is maintained even with malformed input
  - Add defensive programming for cursor position validation
  - _Requirements: 2.1, 2.2, 5.4_

- [x] 12. Performance testing and optimization
  - Measure rendering performance impact of placeholder skipping
  - Test memory usage with large amounts of wide characters
  - Verify no performance regression in normal character handling
  - Optimize width calculation if needed for high-frequency operations
  - _Requirements: 3.3, 4.3_