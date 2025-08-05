### **Technical Implementation Plan: Rendering and Theming Enhancements**

This document outlines the strategy for two key improvements to the terminal application: correcting rendering misalignments and implementing a new visual theme.

***

### **I. Rendering Alignment Correction**

* **Objective**: Eliminate layout misalignments in the terminal grid, ensuring that elements like box-drawing characters align correctly.

* **Problem Analysis**: The root cause of the alignment issue is the application's incorrect assumption that all `char` types occupy a single column of space. Wide characters, particularly emojis (e.g., `✅`, `●`), occupy two columns. The current logic advances the cursor by a single unit for every character, causing a desynchronization between the internal buffer's coordinates and the actual visual layout.

* **Implementation Strategy**: The cursor and buffer logic must be updated to be width-aware.

    1.  **Integrate `unicode-width`**: Add the `unicode-width` crate to calculate the true column-width of each character.
    2.  **Width-Aware Cursor**: In `TerminalEmulator::write_char`, modify the cursor advancement logic to increment `cursor_col` by the character's actual width (`1` or `2`), not by a fixed value.
    3.  **Buffer Placeholders**: When a wide character is written to the buffer, the subsequent cell in the same row must be filled with a `\0` null character. This acts as a placeholder, signifying that the space is occupied.
    4.  **Update Rendering**: The rendering function (`AudioToolkitApp::render_row`) must be updated to explicitly ignore and skip drawing any cell containing the `\0` placeholder.

***

### **II. Visual Theming with Catppuccin**

* **Objective**: Integrate the **Catppuccin** (Mocha flavor) color scheme to create a visually appealing and cohesive user interface.

* **Implementation Strategy**: The approach involves centralizing color definitions and applying them throughout the application.

    1.  **Define a `Theme` Struct**: Create a new struct that holds all the Catppuccin colors as `egui::Color32` fields. This struct will act as a single source of truth for all UI colors.

        ```rust
        // Example struct
        struct CatppuccinTheme {
            rosewater: egui::Color32,
            flamingo: egui::Color32,
            // ... all other colors
            base: egui::Color32,
            text: egui::Color32,
        }

        impl Default for CatppuccinTheme {
            fn default() -> Self {
                Self {
                    rosewater: egui::Color32::from_rgb(242, 213, 207),
                    // ... initialize all colors
                    base: egui::Color32::from_rgb(48, 52, 70),
                    text: egui::Color32::from_rgb(198, 208, 245),
                }
            }
        }
        ```

    2.  **Map ANSI Colors**: Update the `TerminalEmulator::handle_graphics_mode` function. Instead of using a generic palette, it will now map the 16 standard ANSI color codes (30-37, 90-97) to their semantic equivalents in the `CatppuccinTheme` struct.

        * **Default**: `text` on `base`.
        * **ANSI Black (30)**: `surface1`
        * **ANSI Red (31)**: `red`
        * **ANSI Green (32)**: `green`
        * **ANSI Yellow (33)**: `yellow`
        * **ANSI Blue (34)**: `blue`
        * **ANSI Magenta (35)**: `mauve`
        * **ANSI Cyan (36)**: `teal`
        * **ANSI White (37)**: `text`
        * *Bright variants (90-97) will map to the same colors for consistency.*

    3.  **Apply Theme to UI**: In the main `AudioToolkitApp::update` method, replace all hardcoded `egui::Color32` values with fields from an instance of the `CatppuccinTheme` struct. This includes:
        * Setting the background of `egui::SidePanel` and `egui::CentralPanel` to `theme.base`.
        * Changing the text color of UI labels (like the `$` prompt) to `theme.text`.
        * Using theme colors like `theme.blue` for focus indicators and `theme.surface0` for separators.

By executing this plan, the application will be both functionally correct, with perfectly aligned text, and aesthetically refined with the professional Catppuccin theme.
