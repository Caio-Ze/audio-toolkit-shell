# Implementation Plan

- [x] 1. Create CatppuccinTheme struct with compile-time constants
  - Define CatppuccinTheme struct in main.rs with all required color fields
  - Implement MOCHA constant with all Catppuccin Frappé color values as egui::Color32::from_rgb()
  - Add documentation comments explaining each color's purpose and hex values
  - Added complete Frappé palette including rosewater, flamingo, pink, maroon, peach, sky, sapphire, lavender
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2, 4.3_

- [x] 2. Update ANSI color mapping in handle_graphics_mode method
  - Modify TerminalEmulator::handle_graphics_mode to use CatppuccinTheme::MOCHA colors
  - Map ANSI color codes 30-37 to appropriate Catppuccin colors (surface1, red, green, yellow, blue, mauve, teal, text)
  - Map bright ANSI color codes 90-97 to same colors with surface2 for bright black
  - Update color reset (code "0") to use Catppuccin text color
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 5.1, 5.2, 5.3_

- [x] 3. Implement global egui style theming in main update loop
  - Add theme application code in AudioToolkitApp::update method before UI rendering
  - Set style.visuals.window_fill to CatppuccinTheme::MOCHA.base
  - Set style.visuals.panel_fill to CatppuccinTheme::MOCHA.base
  - Set style.visuals.override_text_color to Some(CatppuccinTheme::MOCHA.text)
  - Apply the modified style to egui context using ctx.set_style()
  - _Requirements: 1.1, 1.2, 1.3, 6.1, 6.2, 6.3, 6.4_

- [x] 4. Update terminal panel focus indicators with theme colors
  - Modify terminal panel title rendering to use theme colors for focus indication
  - Set focused terminal title color to CatppuccinTheme::MOCHA.blue
  - Set unfocused terminal title color to CatppuccinTheme::MOCHA.subtext0
  - Apply theme colors to any other focus-related UI elements
  - _Requirements: 3.1, 3.2_

- [ ] 5. Test ANSI color mapping with various terminal outputs
  - Create test cases that output text with different ANSI color codes
  - Verify each ANSI color (30-37) displays with correct Catppuccin color
  - Test bright ANSI colors (90-97) display with appropriate theme colors
  - Ensure color reset functionality works with theme colors
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 5.1, 5.2_

- [ ] 6. Verify theme consistency across all UI elements
  - Test that all panels and windows use Catppuccin base background color
  - Verify all text elements use appropriate Catppuccin text colors
  - Check that interactive elements maintain theme consistency
  - Ensure no UI elements still use default egui colors
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 6.1, 6.2, 6.3, 6.4_

- [ ] 7. Test theme application performance and stability
  - Measure application startup time with theme application
  - Verify no memory leaks from theme constant usage
  - Test that theme application doesn't affect terminal rendering performance
  - Ensure theme persists correctly across application usage
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 8. Validate visual readability and accessibility
  - Test text readability with Catppuccin color combinations
  - Verify sufficient contrast between text and background colors
  - Check that focus indicators are clearly visible
  - Ensure color semantics remain intuitive for ANSI codes
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.2, 3.3_

- [ ] 9. Test integration with existing terminal functionality
  - Verify terminal emulation works correctly with themed colors
  - Test that cursor positioning and text selection work with theme
  - Ensure ANSI escape sequences process correctly with new color mapping
  - Confirm no regression in existing terminal features
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_

- [x] 10. Switch from Catppuccin Mocha to Frappé variant
  - Updated all color values to use Catppuccin Frappé palette
  - Added complete set of Frappé colors including extended palette
  - Updated documentation comments to reflect Frappé variant
  - Verified build compiles successfully with new colors
  - _Requirements: All color-related requirements updated to Frappé_

- [ ] 11. Add comprehensive documentation and code comments
  - Document the CatppuccinTheme struct and its color meanings
  - Add inline comments explaining ANSI color mapping choices
  - Document theme application process in main update loop
  - Add usage examples for theme colors in code comments
  - _Requirements: 4.1, 4.2, 4.3, 4.4_