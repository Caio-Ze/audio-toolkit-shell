The Problem: Misaligned Lines
The box-drawing lines in your terminal are misaligned because the application's internal grid is out of sync with what's on screen.

This happens because your code assumes every character, including emojis (✅, ●), occupies a single column. In reality, these are wide characters that take up two columns. When your program encounters one, it advances the cursor by only one space, causing all subsequent characters on that line—including the vertical bars | of your table—to be drawn at the wrong position.

The Solution: Width-Aware Cursor Logic
To fix the line alignment, the cursor's movement must be based on the actual rendered width of each character, not just its count.

The precise fix involves these steps:

Calculate True Width: Use the unicode-width crate to get the correct column width (1 or 2) for every character before it's placed in the buffer.

Update Cursor Position: Modify TerminalEmulator::write_char to advance the cursor by the character's true width.

Rust

// Before
self.cursor_col += 1;

// After
let width = ch.width().unwrap_or(1);
self.cursor_col += width;
Reserve Space for Wide Characters: When a wide character (width > 1) is written to buffer[row][col], you must mark the next cell, buffer[row][col + 1], as a placeholder (e.g., with a \0 character). This tells your application that the space is already occupied.

Skip Rendering Placeholders: In your rendering function, AudioToolkitApp::render_row, add a condition to skip drawing any cell that contains the \0 placeholder. This prevents visual artifacts and ensures the wide character is drawn correctly across two cells.

By implementing these changes, the internal grid will accurately map to the visual display, guaranteeing your box-drawing lines will always be perfectly aligned.

Alternative Strategy (If Needed)
If the primary solution fails, the only other robust alternative is to use a text shaping engine (e.g., cosmic-text). Instead of managing a grid of characters, a shaper calculates the exact pixel position of each glyph directly from the font file. This is a more complex architectural change but provides perfect layout accuracy for all text, including ligatures and complex scripts. However, for fixing the line alignment caused by wide characters, the first solution is standard and sufficient.