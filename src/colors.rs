//! Color system for a dark theme with elevation hierarchy.
//!
//! Provides a complete color management system with:
//! - RGBA color representation
//! - Theme management
//! - Color manipulation
//! - Integration with owo-colors and ratatui
//! - Color space conversions (RGB, HSL, HEX)

use owo_colors::OwoColorize;
use ratatui::style::Color as RatatuiColor;
use std::fmt::{self, Display};

/// Represents an RGBA color with optional alpha channel
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    /// Create a new RGB color with full opacity
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new RGBA color with specified alpha
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from HSL values
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h % 360.0;
        let s = s.clamp(0.0, 100.0) / 100.0;
        let l = l.clamp(0.0, 100.0) / 100.0;

        if s == 0.0 {
            let v = (l * 255.0) as u8;
            return Self::rgb(v, v, v);
        }

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    /// Convert to HSL values
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let mut h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };

        if h < 0.0 {
            h += 360.0;
        }

        let l = (max + min) / 2.0;
        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        (h, s * 100.0, l * 100.0)
    }

    /// Create a color from a hexadecimal string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to hexadecimal string representation
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Get color with modified alpha
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self { a: alpha, ..*self }
    }

    /// Lightens the color by a percentage (0-100)
    pub fn lighten(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).min(100.0))
    }

    /// Darkens the color by a percentage (0-100)
    pub fn darken(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l - amount).max(0.0))
    }

    /// Adjusts the saturation by a percentage (-100 to 100)
    pub fn saturate(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, (s + amount).clamp(0.0, 100.0), l)
    }

    /// Convert to owo-colors RGB type
    pub fn to_owo_rgb(&self) -> owo_colors::Rgb {
        owo_colors::Rgb(self.r, self.g, self.b)
    }

    /// Convert to ratatui Color
    pub fn to_ratatui(&self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }

    /// Get the RGB components
    pub fn rgb_components(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Get the RGBA components
    pub fn rgba_components(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Mix with another color by a specified amount (0.0-1.0)
    pub fn mix(&self, other: &Color, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        let r = ((self.r as f32 * (1.0 - amount) + other.r as f32 * amount).round()) as u8;
        let g = ((self.g as f32 * (1.0 - amount) + other.g as f32 * amount).round()) as u8;
        let b = ((self.b as f32 * (1.0 - amount) + other.b as f32 * amount).round()) as u8;
        let a = ((self.a as f32 * (1.0 - amount) + other.a as f32 * amount).round()) as u8;
        Self::rgba(r, g, b, a)
    }

    /// Inverts the color
    pub fn invert(&self) -> Self {
        Self::rgba(255 - self.r, 255 - self.g, 255 - self.b, self.a)
    }
}

// Implement conversion to owo-colors RGB
impl From<Color> for owo_colors::Rgb {
    fn from(color: Color) -> Self {
        color.to_owo_rgb()
    }
}

// Implement conversion to ratatui Color
impl From<Color> for RatatuiColor {
    fn from(color: Color) -> Self {
        color.to_ratatui()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "rgba({}, {}, {}, {})",
                self.r,
                self.g,
                self.b,
                self.a as f32 / 255.0
            )
        }
    }
}

/// Extension trait for applying our theme colors to strings with owo-colors
pub trait ThemeColorize: OwoColorize {
    /// Apply primary text color
    #[inline]
    fn primary(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::text::PRIMARY.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply secondary text color
    #[inline]
    fn secondary(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::text::SECONDARY.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply info status color
    #[inline]
    fn info(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::INFO.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply warning status color
    #[inline]
    fn warning(self) -> String
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::WARNING.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply error status color
    #[inline]
    fn error(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::ERROR.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply v01d green color
    #[inline]
    fn void_green(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::void::GREEN.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply v01d purple color
    #[inline]
    fn void_purple(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::void::PURPLE.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }
}

// Implement ThemeColorize for all types that implement OwoColorize
impl<T: OwoColorize + Display> ThemeColorize for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use theme::*;

    // Helper function to compare floats with epsilon
    fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    /// Helper function to round HSL values for testing
    fn round_hsl(hsl: (f32, f32, f32)) -> (f32, f32, f32) {
        (
            (hsl.0 * 10.0).round() / 10.0,
            (hsl.1 * 10.0).round() / 10.0,
            (hsl.2 * 10.0).round() / 10.0,
        )
    }

    #[test]
    fn test_color_conversions() {
        let color = Color::rgb(255, 128, 0);
        let owo = color.to_owo_rgb();
        let ratatui = color.to_ratatui();

        assert_eq!(owo, owo_colors::Rgb(255, 128, 0));
        assert_eq!(ratatui, RatatuiColor::Rgb(255, 128, 0));
    }

    #[test]
    fn test_hex_conversion() {
        let color = Color::rgb(255, 128, 0);
        assert_eq!(color.to_hex(), "#ff8000");

        let parsed = Color::from_hex("#ff8000").unwrap();
        assert_eq!(color, parsed);
    }

    #[test]
    fn test_color_manipulation() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!((0.0, 100.0, 50.0), round_hsl(red.to_hsl()));

        let blue = Color::rgb(0, 0, 255);
        assert_eq!((240.0, 100.0, 50.0), round_hsl(blue.to_hsl()));
    }

    #[test]
    fn test_void_variants() {
        const EPSILON: f32 = 0.2;

        let dark_purple = void::purple_variant(20.0);
        let (_h, _s, l) = dark_purple.to_hsl();
        assert!(
            approx_eq(l, 20.0, EPSILON),
            "Lightness {} not within {} of target 20.0",
            l,
            EPSILON
        );
    }

    #[test]
    fn test_status_variants() {
        const EPSILON: f32 = 0.2;

        let warning = status::WARNING;
        let intense = status::variant(warning, 70.0);

        let (h1, s1, _) = warning.to_hsl();
        let (h2, _s2, l2) = intense.to_hsl();

        assert!(
            approx_eq(h1, h2, EPSILON),
            "Hue changed from {} to {}, diff > {}",
            h1,
            h2,
            EPSILON
        );
        assert!(s1 > 0.0, "Source saturation should be positive");
        assert!(
            approx_eq(l2, 70.0, EPSILON),
            "Lightness {} not within {} of target 70.0",
            l2,
            EPSILON
        );
    }

    #[test]
    fn test_hsl_conversion() {
        let color = Color::rgb(255, 0, 0);
        let (h, s, l) = color.to_hsl();
        assert!((h - 0.0).abs() < 0.1);
        assert!((s - 100.0).abs() < 0.1);
        assert!((l - 50.0).abs() < 0.1);

        let converted = Color::from_hsl(h, s, l);
        assert_eq!(color, converted);
    }

    #[test]
    fn test_theme_colors() {
        assert_eq!(background::BASE.to_hex(), "#0f1214");
        assert_eq!(background::ELEVATION_1.to_hex(), "#161b1e");
        assert_eq!(background::ELEVATION_2.to_hex(), "#1d2428");
        assert_eq!(background::ELEVATION_3.to_hex(), "#242b30");

        assert_eq!(void::GREEN.to_hex(), "#00e49a");
        assert_eq!(void::PURPLE.to_hex(), "#b86eff");

        assert_eq!(text::PRIMARY.to_hex(), "#e6edf3");
        assert_eq!(text::SECONDARY.to_hex(), "#8b949e");

        assert_eq!(status::INFO.to_hex(), "#29bbff");
        assert_eq!(status::WARNING.to_hex(), "#ffbf00");
        assert_eq!(status::ERROR.to_hex(), "#ff2e5f");
    }

    #[test]
    fn test_alpha_handling() {
        let color = Color::rgba(255, 0, 0, 128);
        assert_eq!(color.a, 128);

        let modified = color.with_alpha(64);
        assert_eq!(modified.a, 64);
        assert_eq!(modified.rgb_components(), (255, 0, 0));
    }

    #[test]
    fn test_theme_colorize() {
        let test_str = "Test";

        // Test that colored output contains ANSI codes
        let colored = format!("{}", test_str.primary());
        assert!(colored.contains("\x1b["));

        let error = format!("{}", test_str.error());
        assert!(error.contains("\x1b["));

        let warning = test_str.warning().to_string();
        assert!(warning.contains("\x1b["));
    }
}

/// Theme color constants and semantic color groupings
pub mod theme {
    use super::Color;

    /// Background elevation hierarchy
    pub mod background {
        use super::Color;

        pub const BASE: Color = Color::rgb(15, 18, 20); // #0F1214
        pub const ELEVATION_1: Color = Color::rgb(22, 27, 30); // #161B1E
        pub const ELEVATION_2: Color = Color::rgb(29, 36, 40); // #1D2428
        pub const ELEVATION_3: Color = Color::rgb(36, 43, 48); // #242B30

        /// Creates a custom elevation level by interpolating between existing levels
        pub fn custom_elevation(level: f32) -> Color {
            let level = level.clamp(0.0, 3.0);
            let floor = level.floor() as usize;
            let fract = level.fract();

            match floor {
                0 => BASE.mix(&ELEVATION_1, fract),
                1 => ELEVATION_1.mix(&ELEVATION_2, fract),
                2 => ELEVATION_2.mix(&ELEVATION_3, fract),
                _ => ELEVATION_3,
            }
        }
    }

    /// v01d colors and variants
    pub mod void {
        use super::Color;

        pub const GREEN: Color = Color::rgb(0, 228, 154); // #00E49A
        pub const GREEN_SUBTLE: Color = Color::rgba(0, 228, 154, 38); // 15% opacity
        pub const PURPLE: Color = Color::rgb(184, 110, 255); // #B86EFF
        pub const PURPLE_SUBTLE: Color = Color::rgba(184, 110, 255, 38); // 15% opacity

        /// Creates a custom variant of the v01d green color
        pub fn green_variant(lightness: f32) -> Color {
            let (h, s, _) = GREEN.to_hsl();
            Color::from_hsl(h, s, lightness)
        }

        /// Creates a custom variant of the v01d purple color
        pub fn purple_variant(lightness: f32) -> Color {
            let (h, s, _) = PURPLE.to_hsl();
            Color::from_hsl(h, s, lightness)
        }
    }

    /// Text hierarchy and variations
    pub mod text {
        use super::Color;

        pub const PRIMARY: Color = Color::rgb(230, 237, 243); // #E6EDF3
        pub const SECONDARY: Color = Color::rgb(139, 148, 158); // #8B949E
        pub const DISABLED: Color = Color::rgb(106, 115, 125); // #6A737D
        pub const PLACEHOLDER: Color = Color::rgb(88, 96, 105); // #586069

        /// Creates a custom text color with specified opacity
        pub fn with_opacity(base: Color, opacity: u8) -> Color {
            base.with_alpha(opacity)
        }
    }

    /// System status colors and variants
    pub mod status {
        use super::Color;

        pub const INFO: Color = Color::rgb(41, 187, 255); // #29BBFF
        pub const SUCCESS: Color = Color::rgb(35, 209, 139); // #23D18B
        pub const WARNING: Color = Color::rgb(255, 191, 0); // #FFBF00
        pub const ERROR: Color = Color::rgb(255, 46, 95); // #FF2E5F

        // Subtle variants for backgrounds
        pub const INFO_SUBTLE: Color = Color::rgba(41, 187, 255, 38); // 15% opacity
        pub const SUCCESS_SUBTLE: Color = Color::rgba(35, 209, 139, 38); // 15% opacity
        pub const WARNING_SUBTLE: Color = Color::rgba(255, 191, 0, 38); // 15% opacity
        pub const ERROR_SUBTLE: Color = Color::rgba(255, 46, 95, 38); // 15% opacity

        /// Creates a status color variant with custom intensity
        pub fn variant(base: Color, intensity: f32) -> Color {
            let (h, s, _) = base.to_hsl();
            Color::from_hsl(h, s, intensity)
        }
    }

    /// Base16 theme implementation for terminal compatibility
    pub mod base16 {
        use super::Color;

        pub const BASE00: Color = super::background::BASE; // Base Background
        pub const BASE01: Color = super::background::ELEVATION_1; // Elevation 1
        pub const BASE02: Color = super::background::ELEVATION_2; // Elevation 2
        pub const BASE03: Color = super::background::ELEVATION_3; // Elevation 3
        pub const BASE04: Color = super::text::SECONDARY; // Secondary Text
        pub const BASE05: Color = super::text::PRIMARY; // Primary Text
        pub const BASE06: Color = super::void::GREEN; // Green Primary
        pub const BASE07: Color = super::void::PURPLE; // Purple Secondary
        pub const BASE08: Color = super::status::ERROR; // Red
        pub const BASE09: Color = super::status::WARNING; // Yellow
        pub const BASE0A: Color = super::status::INFO; // Blue
        pub const BASE0B: Color = super::status::SUCCESS; // Green
        pub const BASE0C: Color = super::void::GREEN; // Cyan
        pub const BASE0D: Color = super::void::PURPLE; // Purple
        pub const BASE0E: Color = super::text::DISABLED; // Muted
        pub const BASE0F: Color = super::text::PLACEHOLDER; // Subtle
    }

    /// Semantic color mapping for common UI elements
    pub mod semantic {
        use super::Color;

        // Interactive elements - now using computed values instead of const
        pub const LINK: Color = super::void::PURPLE;
        pub fn link_hover() -> Color {
            super::void::PURPLE.lighten(10.0)
        }

        pub const BUTTON: Color = super::void::GREEN;
        pub fn button_hover() -> Color {
            super::void::GREEN.lighten(10.0)
        }

        // Borders and separators
        pub const BORDER: Color = super::background::ELEVATION_2;
        pub const SEPARATOR: Color = super::background::ELEVATION_1;

        // Selection and focus
        pub const SELECTION: Color = super::void::PURPLE_SUBTLE;
        pub const FOCUS_RING: Color = super::void::PURPLE;

        // Overlays
        pub const MODAL_OVERLAY: Color = Color::rgba(0, 0, 0, 178); // 70% opacity
        pub const DROPDOWN_SHADOW: Color = Color::rgba(0, 0, 0, 204); // 80% opacity
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;
    use theme::{background, semantic, status, void};

    #[test]
    fn test_custom_elevation() {
        let mid_elevation = background::custom_elevation(1.5);
        let (_, _, l1) = background::ELEVATION_1.to_hsl();
        let (_, _, l2) = background::ELEVATION_2.to_hsl();
        let (_, _, lm) = mid_elevation.to_hsl();

        assert!(lm > l1 && lm < l2);
    }

    #[test]
    fn test_void_variants() {
        const EPSILON: f32 = 0.5; // Increased tolerance

        let dark_purple = void::purple_variant(20.0);
        let (_h, _s, l) = dark_purple.to_hsl();

        assert!(
            (l - 20.0).abs() < EPSILON,
            "Lightness too far from target: {} vs 20.0 (diff: {})",
            l,
            (l - 20.0).abs()
        );
    }

    #[test]
    fn test_status_variants() {
        const EPSILON: f32 = 0.5;

        let warning = status::WARNING;
        let intense = status::variant(warning, 70.0);

        let (h1, s1, _) = warning.to_hsl();
        let (h2, _s2, l2) = intense.to_hsl();

        assert!(
            (h1 - h2).abs() < EPSILON,
            "Hue deviated too much: {} vs {} (diff: {})",
            h1,
            h2,
            (h1 - h2).abs()
        );
        assert!(s1 > 0.0, "Source saturation should be positive");
        assert!(
            (l2 - 70.0).abs() < EPSILON,
            "Lightness too far from target: got {}, expected 70.0 ±{}",
            l2,
            EPSILON
        );
    }

    #[test]
    fn test_semantic_colors() {
        assert!(semantic::link_hover().to_hsl().2 > semantic::LINK.to_hsl().2);
        assert!(semantic::button_hover().to_hsl().2 > semantic::BUTTON.to_hsl().2);
        assert_eq!(semantic::MODAL_OVERLAY.a, 178);
        assert_eq!(semantic::DROPDOWN_SHADOW.a, 204);
    }
}
