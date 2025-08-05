# Design Document

## Overview

The Catppuccin theme integration transforms the Audio Toolkit Shell's visual appearance by implementing the popular Catppuccin Mocha color scheme. This design addresses the current lack of visual cohesion in the terminal application by providing a professionally designed, modern color palette that enhances readability and user experience.

The implementation focuses on three key areas: compile-time theme constants for optimal performance, semantic ANSI color mapping that preserves terminal color meaning while using theme colors, and global egui styling to ensure consistency across all UI elements. The design maintains full backward compatibility while significantly improving the visual appeal of the application.

## Architecture

### Current State Analysis
The existing Audio Toolkit Shell uses default egui colors and basic ANSI color mappings:
- Default egui theme colors (gray backgrounds, basic text colors)
- Simple ANSI color mapping with standard RGB values
- No cohesive visual design language
- Inconsistent color usage across UI elements

### Proposed Architecture Enhancement
The Catppuccin integration will enhance the existing architecture through:

1. **Theme Constants Module**: A structured approach to color management using compile-time constants
2. **Enhanced ANSI Processing**: Updated `handle_graphics_mode` method with theme-aware color mapping
3. **Global Style Application**: Integration with egui's styling system for consistent theming
4. **Semantic Color Usage**: Meaningful color assignments that preserve ANSI semantics

### Integration Points
The theme system integrates seamlessly with existing components:
- `TerminalEmulator::handle_graphics_mode` for ANSI color processing
- `AudioToolkitApp::update` for global style application
- Existing terminal rendering pipeline remains unchanged
- No impact on PTY communication or terminal functionality

## Components and Interfaces

### CatppuccinTheme Structure
```rust
pub struct CatppuccinTheme {
    // Base colors for backgrounds and surfaces
    pub base: egui::Color32,        // Main background (#303446)
    pub mantle: egui::Color32,      // Secondary background (#292c3c)
    pub crust: egui::Color32,       // Darkest background (#232634)

    // Text hierarchy colors
    pub text: egui::Color32,        // Primary text (#c6d0f5)
    pub subtext1: egui::Color32,    // Secondary text (#b5bfe2)
    pub subtext0: egui::Color32,    // Tertiary text (#a5adce)

    // Semantic accent colors
    pub red: egui::Color32,         // Error/danger (#e78284)
    pub green: egui::Color32,       // Success/safe (#a6d189)
    pub yellow: egui::Color32,      // Warning/caution (#e5c890)
    pub blue: egui::Color32,        // Info/primary (#8caaee)
    pub mauve: egui::Color32,       // Special/accent (#ca9ee6)
    pub teal: egui::Color32,        // Highlight/secondary (#81c8be)

    // Surface colors for UI elements
    pub surface0: egui::Color32,    // Elevated surface (#414559)
    pub surface1: egui::Color32,    // More elevated (#51576d)
    pub surface2: egui::Color32,    // Highest elevation (#626880)
}
```

### Theme Implementation Interface
```rust
impl CatppuccinTheme {
    pub const MOCHA: Self = Self {
        // Compile-time constant initialization
        // All colors defined as const values for optimal performance
    };
    
    // Helper methods for common color operations
    pub fn get_ansi_color(&self, code: u8) -> egui::Color32 {
        // Maps ANSI codes to appropriate theme colors
    }
    
    pub fn apply_to_style(&self, style: &mut egui::Style) {
        // Applies theme colors to egui global style
    }
}
```

### Enhanced ANSI Processing Interface
```rust
impl TerminalEmulator {
    fn handle_graphics_mode(&mut self, params: &[&str]) {
        const THEME: &CatppuccinTheme = &CatppuccinTheme::MOCHA;
        
        // Process ANSI codes with theme color mapping
        // Maintains existing parameter parsing logic
        // Updates color assignments to use theme colors
    }
}
```

### Global Style Application Interface
```rust
impl AudioToolkitApp {
    fn apply_catppuccin_theme(&self, ctx: &egui::Context) {
        const THEME: &CatppuccinTheme = &CatppuccinTheme::MOCHA;
        
        // Apply theme to global egui style
        // Set window, panel, and text colors
        // Configure visual elements consistently
    }
}
```

## Data Models

### Color Mapping Model
```rust
struct AnsiColorMapping {
    normal_colors: [egui::Color32; 8],    // ANSI 30-37
    bright_colors: [egui::Color32; 8],    // ANSI 90-97
    reset_color: egui::Color32,           // Default text color
}

impl AnsiColorMapping {
    const CATPPUCCIN_MAPPING: Self = Self {
        normal_colors: [
            CatppuccinTheme::MOCHA.surface1,  // Black (30)
            CatppuccinTheme::MOCHA.red,       // Red (31)
            CatppuccinTheme::MOCHA.green,     // Green (32)
            CatppuccinTheme::MOCHA.yellow,    // Yellow (33)
            CatppuccinTheme::MOCHA.blue,      // Blue (34)
            CatppuccinTheme::MOCHA.mauve,     // Magenta (35)
            CatppuccinTheme::MOCHA.teal,      // Cyan (36)
            CatppuccinTheme::MOCHA.text,      // White (37)
        ],
        bright_colors: [
            CatppuccinTheme::MOCHA.surface2,  // Bright Black (90)
            CatppuccinTheme::MOCHA.red,       // Bright Red (91)
            CatppuccinTheme::MOCHA.green,     // Bright Green (92)
            CatppuccinTheme::MOCHA.yellow,    // Bright Yellow (93)
            CatppuccinTheme::MOCHA.blue,      // Bright Blue (94)
            CatppuccinTheme::MOCHA.mauve,     // Bright Magenta (95)
            CatppuccinTheme::MOCHA.teal,      // Bright Cyan (96)
            CatppuccinTheme::MOCHA.text,      // Bright White (97)
        ],
        reset_color: CatppuccinTheme::MOCHA.text,
    };
}
```

### UI Element Color Model
```rust
struct UiColorScheme {
    // Panel and window colors
    background: egui::Color32,
    panel_fill: egui::Color32,
    window_fill: egui::Color32,
    
    // Text colors
    primary_text: egui::Color32,
    secondary_text: egui::Color32,
    disabled_text: egui::Color32,
    
    // Interactive element colors
    focused_accent: egui::Color32,
    unfocused_accent: egui::Color32,
    hover_color: egui::Color32,
    selection_color: egui::Color32,
}
```

### Theme State Model
```rust
struct ThemeState {
    current_theme: &'static CatppuccinTheme,
    is_applied: bool,
    last_style_hash: u64,  // For detecting style changes
}

impl ThemeState {
    fn ensure_applied(&mut self, ctx: &egui::Context) {
        // Check if theme needs to be reapplied
        // Handle style updates efficiently
    }
}
```

## Error Handling

### Theme Application Errors
- **Style Conflicts**: Handle cases where external code modifies egui style after theme application
- **Color Conversion Issues**: Ensure all RGB values are valid and within expected ranges
- **Context Availability**: Gracefully handle cases where egui context is not available during theme application

### ANSI Processing Robustness
- **Invalid Color Codes**: Maintain existing behavior for unrecognized ANSI codes
- **Malformed Parameters**: Handle edge cases in ANSI parameter parsing without breaking theme colors
- **State Consistency**: Ensure theme colors don't interfere with existing terminal state management

### Performance Safeguards
- **Compile-Time Validation**: All color constants validated at compile time to prevent runtime errors
- **Memory Efficiency**: Theme constants use minimal memory footprint with no runtime allocation
- **Style Update Optimization**: Minimize unnecessary style updates to prevent performance degradation

## Testing Strategy

### Visual Consistency Testing
1. **Theme Application Verification**
   - Verify all UI elements use Catppuccin colors
   - Test background colors match specification
   - Validate text readability with theme colors
   - Check focus indicators use correct accent colors

2. **ANSI Color Mapping Tests**
   - Test each ANSI color code (30-37, 90-97) maps correctly
   - Verify bright colors maintain semantic meaning
   - Test color reset functionality with theme colors
   - Validate color combinations remain readable

3. **Cross-Component Consistency**
   - Ensure terminal panels use consistent theming
   - Verify UI elements maintain theme coherence
   - Test theme application across different screen sizes
   - Validate theme persistence across application restarts

### Integration Testing Strategy
1. **Existing Functionality Preservation**
   - All terminal features continue working with new colors
   - ANSI processing maintains backward compatibility
   - Terminal rendering performance remains unchanged
   - No regression in text selection or cursor behavior

2. **Theme System Integration**
   - Theme application doesn't interfere with terminal functionality
   - Color changes apply immediately without restart
   - Theme constants accessible throughout application
   - No memory leaks from theme application

3. **Performance Impact Assessment**
   - Measure theme application overhead
   - Verify no performance regression in terminal rendering
   - Test memory usage with theme constants
   - Validate startup time impact

### User Experience Testing
1. **Readability Assessment**
   - Test text readability in various lighting conditions
   - Verify color contrast meets accessibility standards
   - Validate color combinations don't cause eye strain
   - Test with different terminal content types

2. **Visual Appeal Evaluation**
   - Compare before/after visual appearance
   - Verify professional appearance of themed interface
   - Test visual consistency across all UI elements
   - Validate theme matches Catppuccin design guidelines

3. **Usability Testing**
   - Ensure focus indicators are clearly visible
   - Test that important information stands out appropriately
   - Verify theme doesn't interfere with workflow
   - Validate color semantics remain intuitive

## Implementation Phases

### Phase 1: Theme Constants and Structure
- Define CatppuccinTheme struct with all required colors
- Implement compile-time constants for Mocha variant
- Create helper methods for color access and manipulation
- Add comprehensive documentation for theme structure

### Phase 2: ANSI Color Mapping Integration
- Update handle_graphics_mode method with theme color mapping
- Implement semantic color assignments for all ANSI codes
- Ensure bright colors maintain consistency with normal colors
- Test ANSI processing with theme colors

### Phase 3: Global Style Application
- Implement theme application in main update loop
- Configure egui global style with Catppuccin colors
- Apply theme to panels, windows, and text elements
- Ensure consistent theming across all UI components

### Phase 4: Focus and Interactive Elements
- Implement themed focus indicators for terminal panels
- Apply theme colors to hover states and selections
- Configure interactive element colors for consistency
- Test user interaction feedback with theme colors

### Phase 5: Testing and Refinement
- Comprehensive visual testing with various terminal content
- Performance testing to ensure no regression
- User experience validation and refinement
- Documentation updates and code cleanup

## Compatibility Considerations

### Backward Compatibility
- All existing terminal functionality preserved
- ANSI color processing maintains semantic meaning
- No changes to configuration files or external APIs
- Existing color-based scripts continue working

### Performance Compatibility
- Compile-time constants ensure zero runtime overhead
- Theme application has minimal performance impact
- No additional memory allocation for color management
- Rendering performance remains unchanged

### Visual Compatibility
- Theme colors maintain sufficient contrast for readability
- Color semantics preserved for ANSI codes
- Professional appearance suitable for development work
- Consistent with modern terminal application standards

### Platform Compatibility
- Theme works consistently across all supported platforms
- No platform-specific color handling required
- egui color system ensures cross-platform consistency
- Theme constants work with all target architectures