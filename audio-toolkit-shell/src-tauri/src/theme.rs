//! # Theme Module
//! 
//! This module provides the Catppuccin Frappé color theme for the Audio Toolkit Shell.
//! It includes the complete color palette and utility functions for color conversion.

use eframe::egui;

/// Catppuccin Frappé color theme for the Audio Toolkit Shell
/// 
/// This struct provides compile-time constants for all Catppuccin Frappé colors,
/// ensuring optimal performance and consistent theming throughout the application.
/// 
/// Color palette based on Catppuccin Frappé variant:
/// https://github.com/catppuccin/catppuccin
#[derive(Clone, Copy, Debug)]
pub struct CatppuccinTheme {
    // Base colors for backgrounds and surfaces
    /// Main background color (#303446) - Used for primary backgrounds
    pub base: egui::Color32,
    /// Secondary background color (#292c3c) - Used for elevated surfaces
    pub mantle: egui::Color32,
    /// Darkest background color (#232634) - Used for deepest surfaces
    pub crust: egui::Color32,

    // Text hierarchy colors
    /// Primary text color (#c6d0f5) - Used for main text content
    pub text: egui::Color32,
    /// Secondary text color (#b5bfe2) - Used for less important text
    pub subtext1: egui::Color32,
    /// Tertiary text color (#a5adce) - Used for subtle text and hints
    pub subtext0: egui::Color32,

    // Surface colors for UI elements
    /// Lowest elevation surface (#414559) - Used for subtle elevation
    pub surface0: egui::Color32,
    /// Medium elevation surface (#51576d) - Used for moderate elevation
    pub surface1: egui::Color32,
    /// Highest elevation surface (#626880) - Used for prominent elevation
    pub surface2: egui::Color32,

    // Overlay colors
    /// Overlay color (#737994) - Used for overlays and disabled states
    pub overlay0: egui::Color32,
    /// Secondary overlay color (#838ba7) - Used for secondary overlays
    pub overlay1: egui::Color32,
    /// Tertiary overlay color (#949cbb) - Used for tertiary overlays
    pub overlay2: egui::Color32,

    // Semantic accent colors
    /// Blue accent color (#8caaee) - Used for primary actions and info
    pub blue: egui::Color32,
    /// Lavender accent color (#babbf1) - Used for special highlights
    pub lavender: egui::Color32,
    /// Sapphire accent color (#85c1dc) - Used for secondary actions
    pub sapphire: egui::Color32,
    /// Sky accent color (#99d1db) - Used for tertiary actions
    pub sky: egui::Color32,
    /// Teal accent color (#81c8be) - Used for success and safe actions
    pub teal: egui::Color32,
    /// Green accent color (#a6d189) - Used for success states
    pub green: egui::Color32,
    /// Yellow accent color (#e5c890) - Used for warnings and caution
    pub yellow: egui::Color32,
    /// Peach accent color (#ef9f76) - Used for warm accents
    pub peach: egui::Color32,
    /// Maroon accent color (#ea999c) - Used for muted red accents
    pub maroon: egui::Color32,
    /// Red accent color (#e78284) - Used for errors and danger
    pub red: egui::Color32,
    /// Mauve accent color (#ca9ee6) - Used for special accents and highlights
    pub mauve: egui::Color32,
    /// Pink accent color (#f4b8e4) - Used for decorative accents
    pub pink: egui::Color32,
    /// Flamingo accent color (#eebebe) - Used for soft accents
    pub flamingo: egui::Color32,
    /// Rosewater accent color (#f2d5cf) - Used for subtle warm accents
    pub rosewater: egui::Color32,
}

impl CatppuccinTheme {
    /// Catppuccin Frappé theme constant with all colors defined at compile-time
    /// 
    /// This constant provides immediate access to all Catppuccin Frappé colors
    /// without any runtime initialization overhead.
    pub const FRAPPE: Self = Self {
        // Base colors
        base: egui::Color32::from_rgb(0x30, 0x34, 0x46),      // #303446
        mantle: egui::Color32::from_rgb(0x29, 0x2c, 0x3c),    // #292c3c
        crust: egui::Color32::from_rgb(0x23, 0x26, 0x34),     // #232634

        // Text colors
        text: egui::Color32::from_rgb(0xc6, 0xd0, 0xf5),      // #c6d0f5
        subtext1: egui::Color32::from_rgb(0xb5, 0xbf, 0xe2),  // #b5bfe2
        subtext0: egui::Color32::from_rgb(0xa5, 0xad, 0xce),  // #a5adce

        // Surface colors
        surface0: egui::Color32::from_rgb(0x41, 0x45, 0x59),  // #414559
        surface1: egui::Color32::from_rgb(0x51, 0x57, 0x6d),  // #51576d
        surface2: egui::Color32::from_rgb(0x62, 0x68, 0x80),  // #626880

        // Overlay colors
        overlay0: egui::Color32::from_rgb(0x73, 0x79, 0x94),  // #737994
        overlay1: egui::Color32::from_rgb(0x83, 0x8b, 0xa7),  // #838ba7
        overlay2: egui::Color32::from_rgb(0x94, 0x9c, 0xbb),  // #949cbb

        // Accent colors
        blue: egui::Color32::from_rgb(0x8c, 0xaa, 0xee),      // #8caaee
        lavender: egui::Color32::from_rgb(0xba, 0xbb, 0xf1),  // #babbf1
        sapphire: egui::Color32::from_rgb(0x85, 0xc1, 0xdc),  // #85c1dc
        sky: egui::Color32::from_rgb(0x99, 0xd1, 0xdb),       // #99d1db
        teal: egui::Color32::from_rgb(0x81, 0xc8, 0xbe),      // #81c8be
        green: egui::Color32::from_rgb(0xa6, 0xd1, 0x89),     // #a6d189
        yellow: egui::Color32::from_rgb(0xe5, 0xc8, 0x90),    // #e5c890
        peach: egui::Color32::from_rgb(0xef, 0x9f, 0x76),     // #ef9f76
        maroon: egui::Color32::from_rgb(0xea, 0x99, 0x9c),    // #ea999c
        red: egui::Color32::from_rgb(0xe7, 0x82, 0x84),       // #e78284
        mauve: egui::Color32::from_rgb(0xca, 0x9e, 0xe6),     // #ca9ee6
        pink: egui::Color32::from_rgb(0xf4, 0xb8, 0xe4),      // #f4b8e4
        flamingo: egui::Color32::from_rgb(0xee, 0xbe, 0xbe),  // #eebebe
        rosewater: egui::Color32::from_rgb(0xf2, 0xd5, 0xcf), // #f2d5cf
    };
}

/// Helper function for 256-color ANSI to RGB conversion
/// 
/// Converts ANSI 256-color codes to RGB values, using Catppuccin Frappé colors
/// for the standard 16 colors (0-15) and standard color cube/grayscale for the rest.
/// 
/// # Arguments
/// 
/// * `color_index` - The ANSI color index (0-255)
/// 
/// # Returns
/// 
/// An `egui::Color32` representing the RGB color
pub fn ansi_256_to_rgb(color_index: u8) -> egui::Color32 {
    // Use Catppuccin Frappé theme for standard colors (0-15)
    const THEME: &CatppuccinTheme = &CatppuccinTheme::FRAPPE;
    
    match color_index {
        // Standard colors (0-15) mapped to Catppuccin Frappé colors
        0 => THEME.surface1,   // Black -> surface1
        1 => THEME.red,        // Dark Red -> Catppuccin red
        2 => THEME.green,      // Dark Green -> Catppuccin green
        3 => THEME.yellow,     // Dark Yellow -> Catppuccin yellow
        4 => THEME.blue,       // Dark Blue -> Catppuccin blue
        5 => THEME.mauve,      // Dark Magenta -> Catppuccin mauve
        6 => THEME.teal,       // Dark Cyan -> Catppuccin teal
        7 => THEME.subtext1,   // Light Gray -> Catppuccin subtext1
        8 => THEME.surface2,   // Dark Gray -> Catppuccin surface2
        9 => THEME.red,        // Bright Red -> Catppuccin red (same as dark red for consistency)
        10 => THEME.green,     // Bright Green -> Catppuccin green
        11 => THEME.yellow,    // Bright Yellow -> Catppuccin yellow
        12 => THEME.blue,      // Bright Blue -> Catppuccin blue
        13 => THEME.mauve,     // Bright Magenta -> Catppuccin mauve
        14 => THEME.teal,      // Bright Cyan -> Catppuccin teal
        15 => THEME.text,      // Bright White -> Catppuccin text

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