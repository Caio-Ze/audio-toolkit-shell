# Design Document

## Overview

The terminal rendering alignment fix addresses a critical issue in the Audio Toolkit Shell's terminal emulator where characters with different display widths cause layout misalignments and cursor desynchronization. The current implementation in `TerminalEmulator::write_char` assumes all characters occupy exactly one column, which breaks when displaying wide characters like emojis (width=2) or certain Unicode symbols.

The solution involves implementing Unicode-width-aware character handling using the `unicode-width` crate, modifying the cursor advancement logic, and updating the rendering system to properly handle wide character placeholders.

## Architecture

### Current Architecture Issues
The existing `TerminalEmulator` struct in `main.rs` has these problematic behaviors:
- `write_char` method always advances cursor by 1 regardless of character width
- No handling for wide characters that span multiple columns
- Rendering system doesn't account for character width variations
- Box-drawing characters and emojis cause visual misalignment

### Proposed Architecture Changes
The fix will enhance the existing architecture with minimal structural changes:

1. **Dependency Addition**: Add `unicode-width` crate for accurate character width detection
2. **Enhanced TerminalCell**: Modify to support placeholder cells for wide characters
3. **Width-Aware Cursor Logic**: Update cursor advancement to use actual character width
4. **Smart Rendering**: Skip placeholder cells during display rendering

### Component Integration
The changes integrate seamlessly with the existing eframe/egui architecture:
- `TerminalEmulator` remains the core component with enhanced width handling
- `TerminalCell` structure extended to support wide character placeholders
- Rendering logic in `AudioToolkitApp::render_row` updated to skip placeholders
- ANSI processing and color handling remain unchanged

## Components and Interfaces

### Enhanced TerminalCell Structure
```rust
#[derive(Clone)]
struct TerminalCell {
    character: char,
    color: egui::Color32,
    bold: bool,
    // No structural changes needed - we'll use '\0' as placeholder indicator
}
```

### Modified TerminalEmulator Interface
```rust
impl TerminalEmulator {
    // Enhanced write_char method with width awareness
    fn write_char(&mut self, ch: char) {
        // 1. Calculate character width using unicode-width crate
        // 2. Check line boundary with width consideration
        // 3. Handle line wrapping for wide characters
        // 4. Place character and placeholder if needed
        // 5. Advance cursor by actual width
    }
    
    // New helper method for line wrapping logic
    fn handle_newline(&mut self) {
        // Existing logic remains the same
    }
    
    // Enhanced boundary checking
    fn fits_on_current_line(&self, width: usize) -> bool {
        self.cursor_col + width <= self.cols
    }
}
```

### Updated Rendering Interface
```rust
impl AudioToolkitApp {
    // Modified render_row to skip placeholder characters
    fn render_row(row: &[TerminalCell], ui: &mut egui::Ui) {
        // Skip cells with character == '\0' (placeholders)
        // Render only actual characters with proper spacing
    }
}
```

### Unicode Width Integration
```rust
use unicode_width::UnicodeWidthChar;

// Character width detection
fn get_char_width(ch: char) -> usize {
    ch.width().unwrap_or(1)  // Default to 1 for undefined width
}
```

## Data Models

### Character Width Classification
```rust
enum CharacterWidth {
    Normal = 1,    // Standard ASCII and most Unicode
    Wide = 2,      // Emojis, CJK characters, some symbols
    Zero = 0,      // Combining characters (future enhancement)
}
```

### Wide Character Handling Model
```rust
struct WideCharacterPlacement {
    main_position: (usize, usize),      // Row, col of actual character
    placeholder_position: (usize, usize), // Row, col of placeholder
    character: char,
    width: usize,
}
```

### Cursor State Model
```rust
struct CursorState {
    row: usize,
    col: usize,
    max_rows: usize,
    max_cols: usize,
}

impl CursorState {
    fn advance_by_width(&mut self, width: usize) -> bool {
        // Returns true if line wrap occurred
        let new_col = self.col + width;
        if new_col >= self.max_cols {
            self.wrap_to_next_line();
            true
        } else {
            self.col = new_col;
            false
        }
    }
    
    fn wrap_to_next_line(&mut self) {
        self.col = 0;
        self.row += 1;
        // Handle scrolling if needed
    }
}
```

## Error Handling

### Character Width Detection Errors
- **Undefined Width Characters**: Use default width of 1 for characters where `unicode_width` returns `None`
- **Invalid Unicode**: Handle malformed Unicode sequences gracefully without crashing
- **Zero-Width Characters**: Treat as width 1 for simplicity (future enhancement can handle properly)

### Boundary Condition Handling
- **Wide Character at Line End**: If a wide character would exceed line boundary, wrap entire character to next line
- **Buffer Overflow Protection**: Ensure cursor never exceeds buffer boundaries even with width calculations
- **Placeholder Overflow**: Prevent placeholder cells from being written outside buffer bounds

### Rendering Error Recovery
- **Null Character Display**: Skip rendering of '\0' placeholder characters without error
- **Malformed Buffer State**: Gracefully handle cases where placeholders are missing or misplaced
- **Font Rendering Issues**: Fallback to single-width rendering if font doesn't support wide characters

### Memory and Performance Safeguards
- **Buffer Integrity**: Ensure wide character operations don't corrupt adjacent cells
- **Cursor Synchronization**: Maintain cursor position consistency even if width calculations fail
- **Rendering Performance**: Minimize overhead of placeholder checking during display

## Testing Strategy

### Unit Testing Approach
1. **Character Width Detection Tests**
   - Test ASCII characters return width 1
   - Test emojis return width 2
   - Test box-drawing characters return width 1
   - Test undefined characters default to width 1

2. **Cursor Advancement Tests**
   - Verify cursor advances by 1 for normal characters
   - Verify cursor advances by 2 for wide characters
   - Test line wrapping with wide characters
   - Test boundary conditions at line edges

3. **Placeholder Handling Tests**
   - Verify placeholder cells are created for wide characters
   - Test placeholder cells use '\0' character
   - Verify placeholders have transparent color
   - Test placeholder cells are skipped during rendering

### Integration Testing Strategy
1. **Terminal Buffer Integrity**
   - Write mixed normal and wide characters
   - Verify buffer state remains consistent
   - Test cursor position matches visual layout
   - Verify no buffer corruption occurs

2. **ANSI Sequence Compatibility**
   - Test wide characters with color codes
   - Verify cursor positioning commands work correctly
   - Test screen clearing with wide characters present
   - Ensure existing ANSI handling remains functional

3. **Rendering Verification**
   - Visual test with emoji characters
   - Test box-drawing character alignment
   - Verify text selection works correctly
   - Test scrolling with wide characters

### Manual Testing Scenarios
1. **Real-World Usage Tests**
   - Display output containing emojis
   - Show box-drawing based UI elements
   - Test with various Unicode symbols
   - Verify alignment in complex terminal UIs

2. **Edge Case Testing**
   - Wide character at exact line boundary
   - Multiple consecutive wide characters
   - Mixed wide and normal characters on same line
   - Wide characters with ANSI color codes

3. **Performance Testing**
   - Large amounts of mixed character content
   - Rapid character output with width calculations
   - Memory usage with placeholder cells
   - Rendering performance with skip logic

### Regression Testing
1. **Existing Functionality Preservation**
   - All current terminal features continue working
   - ANSI color codes still function correctly
   - Cursor positioning commands work as before
   - Screen clearing and line operations unchanged

2. **Configuration Compatibility**
   - All existing tab configurations continue working
   - Terminal size and buffer operations unchanged
   - PTY integration remains functional
   - No breaking changes to external interfaces

## Implementation Phases

### Phase 1: Dependency and Core Logic (Critical Priority)
- Add `unicode-width = "0.1"` to Cargo.toml
- Implement width-aware `write_char` method
- Add character width detection using `UnicodeWidthChar::width()`
- Implement placeholder cell creation for wide characters
- Update cursor advancement logic to use actual character width

### Phase 2: Line Wrapping and Boundary Handling
- Implement proper line wrapping for wide characters
- Add boundary checking that considers character width
- Ensure wide characters that don't fit wrap to next line entirely
- Handle edge cases where wide characters are at line boundaries

### Phase 3: Rendering Updates
- Modify `render_row` method to skip placeholder characters ('\0')
- Ensure placeholder cells don't affect visual spacing
- Verify wide characters display correctly without artifacts
- Test rendering performance with placeholder skip logic

### Phase 4: Testing and Validation
- Create comprehensive test suite for width handling
- Test with various Unicode characters and emojis
- Verify existing functionality remains intact
- Performance testing and optimization if needed

### Phase 5: Documentation and Cleanup
- Update code comments to explain wide character handling
- Document the placeholder cell approach
- Add inline documentation for width-related methods
- Clean up any temporary debugging code

## Compatibility Considerations

### Backward Compatibility
- All existing terminal functionality preserved
- No changes to external APIs or configuration format
- Existing ANSI sequence handling unchanged
- PTY integration remains identical

### Font Compatibility
- Solution works with any monospace font
- No dependency on specific font features
- Graceful degradation if font doesn't support wide characters
- Maintains consistent spacing regardless of font choice

### Platform Compatibility
- Unicode-width crate supports all target platforms
- No platform-specific character width handling needed
- Consistent behavior across macOS, Linux, and Windows
- No additional system dependencies required

### Performance Impact
- Minimal overhead from width calculations
- Placeholder cells use same memory as regular cells
- Skip logic in rendering is O(1) per cell
- No significant impact on terminal responsiveness