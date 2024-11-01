//! # Color System
//!
//! A comprehensive color management system providing RGBA color representation, theme management,
//! and color manipulation capabilities. The system is designed specifically for terminal user
//! interfaces with support for dark themes and elevation hierarchy.
//!
//! ## Features
//!
//! - Full RGBA color support with 8-bit components
//! - HSL and hexadecimal color space conversions
//! - Color manipulation (lighten, darken, saturate)
//! - Theme management with semantic color groupings
//! - Integration with owo-colors and ratatui
//! - Thread-safe and zero-allocation color operations
//!
//! ## Main Components
//!
//! - [`Color`]: Core struct for color representation and manipulation
//! - [`ThemeColorize`]: Trait for applying theme colors to text
//! - [`theme`]: Module containing all theme-related color constants and functions
//!
//! ## Examples
//!
//! Basic color creation and manipulation:
//!
//! ```rust
//! use oxitty::colors::{Color, theme};
//!
//! // Create colors
//! let red = Color::rgb(255, 0, 0);
//! let blue = Color::from_hex("#0000FF").unwrap();
//! let semi_transparent = Color::rgba(0, 255, 0, 128);
//!
//! // Color manipulation
//! let lighter_red = red.lighten(20.0);
//! let darker_blue = blue.darken(30.0);
//! let mixed = red.mix(&blue, 0.5);
//! ```
//!
//! Using the theme system:
//!
//! ```rust
//! use oxitty::colors::{theme, ThemeColorize};
//!
//! // Theme colors
//! let bg = theme::background::BASE;
//! let text = theme::text::PRIMARY;
//! let accent = theme::void::PURPLE;
//!
//! // Applying colors to text
//! println!("{}", "Important message".primary());
//! println!("{}", "Warning alert".warning());
//! ```

use owo_colors::OwoColorize;
use ratatui::style::Color as RatatuiColor;
use std::fmt::{self, Display};

/// Represents an RGBA color with 8-bit components for each channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0-255)
    r: u8,
    /// Green component (0-255)
    g: u8,
    /// Blue component (0-255)
    b: u8,
    /// Alpha component (0-255)
    a: u8,
}

impl Color {
    /// Creates a new RGB color with full opacity.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let red = Color::rgb(255, 0, 0);
    /// assert_eq!(red.rgb_components(), (255, 0, 0));
    /// ```
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates a new RGBA color with specified alpha.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let semi_transparent_blue = Color::rgba(0, 0, 255, 128);
    /// assert_eq!(semi_transparent_blue.rgba_components(), (0, 0, 255, 128));
    /// ```
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from HSL values.
    ///
    /// # Arguments
    ///
    /// * `h` - Hue in degrees (0-360)
    /// * `s` - Saturation percentage (0-100)
    /// * `l` - Lightness percentage (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let red = Color::from_hsl(0.0, 100.0, 50.0);
    /// let pastel_blue = Color::from_hsl(210.0, 65.0, 75.0);
    /// ```
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

    /// Converts the color to HSL values.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * Hue (0-360)
    /// * Saturation (0-100)
    /// * Lightness (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(255, 0, 0);
    /// let (h, s, l) = color.to_hsl();
    /// assert_eq!(h, 0.0); // Red hue
    /// assert_eq!(s, 100.0); // Full saturation
    /// assert_eq!(l, 50.0); // Mid lightness
    /// ```
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

    /// Creates a color from a hexadecimal string.
    ///
    /// Supports both RGB (#RRGGBB) and RGBA (#RRGGBBAA) formats.
    /// The '#' prefix is optional.
    ///
    /// # Arguments
    ///
    /// * `hex` - Hexadecimal color string
    ///
    /// # Returns
    ///
    /// `Some(Color)` if parsing succeeds, `None` if the string is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let red = Color::from_hex("#FF0000").unwrap();
    /// let transparent_blue = Color::from_hex("0000FF80").unwrap();
    /// assert!(Color::from_hex("invalid").is_none());
    /// ```
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

    /// Converts the color to a hexadecimal string.
    ///
    /// Returns a 6-digit hex string for opaque colors (#RRGGBB)
    /// or an 8-digit hex string for transparent colors (#RRGGBBAA).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(255, 0, 0);
    /// assert_eq!(color.to_hex(), "#ff0000");
    ///
    /// let transparent = Color::rgba(0, 255, 0, 128);
    /// assert_eq!(transparent.to_hex(), "#00ff0080");
    /// ```
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Returns a new color with modified alpha value.
    ///
    /// # Arguments
    ///
    /// * `alpha` - New alpha value (0-255)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let red = Color::rgb(255, 0, 0);
    /// let transparent_red = red.with_alpha(128);
    /// assert_eq!(transparent_red.rgba_components().3, 128);
    /// ```
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self { a: alpha, ..*self }
    }

    /// Lightens the color by a percentage.
    ///
    /// # Arguments
    ///
    /// * `amount` - Percentage to lighten (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(100, 100, 100);
    /// let lighter = color.lighten(20.0);
    /// let (_, _, l1) = color.to_hsl();
    /// let (_, _, l2) = lighter.to_hsl();
    /// assert!(l2 > l1);
    /// ```
    pub fn lighten(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).min(100.0))
    }

    /// Darkens the color by a percentage.
    ///
    /// # Arguments
    ///
    /// * `amount` - Percentage to darken (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(200, 200, 200);
    /// let darker = color.darken(30.0);
    /// let (_, _, l1) = color.to_hsl();
    /// let (_, _, l2) = darker.to_hsl();
    /// assert!(l2 < l1);
    /// ```
    pub fn darken(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l - amount).max(0.0))
    }

    /// Adjusts the saturation by a percentage.
    ///
    /// # Arguments
    ///
    /// * `amount` - Percentage to adjust (-100 to 100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(200, 100, 100);
    /// let more_saturated = color.saturate(20.0);
    /// let less_saturated = color.saturate(-20.0);
    /// ```
    pub fn saturate(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, (s + amount).clamp(0.0, 100.0), l)
    }

    /// Converts to owo-colors RGB type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(255, 0, 0);
    /// let owo_color = color.to_owo_rgb();
    /// ```
    pub fn to_owo_rgb(&self) -> owo_colors::Rgb {
        owo_colors::Rgb(self.r, self.g, self.b)
    }

    /// Converts to ratatui Color type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(255, 0, 0);
    /// let ratatui_color = color.to_ratatui();
    /// ```
    pub fn to_ratatui(&self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }

    /// Returns the RGB components as a tuple.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgb(255, 128, 0);
    /// assert_eq!(color.rgb_components(), (255, 128, 0));
    /// ```
    pub fn rgb_components(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Returns the RGBA components as a tuple.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let color = Color::rgba(255, 128, 0, 128);
    /// assert_eq!(color.rgba_components(), (255, 128, 0, 128));
    /// ```
    pub fn rgba_components(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Mixes with another color by a specified amount.
    ///
    /// # Arguments
    ///
    /// * `other` - The color to mix with
    /// * `amount` - Mix ratio (0.0-1.0), where 0.0 is this color and 1.0 is the other color
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::Color;
    ///
    /// let red = Color::rgb(255, 0, 0);
    /// let blue = Color::rgb(0, 0, 255);
    /// let purple = red.mix(&blue, 0.5);
    /// assert_eq!(purple.rgb_components(), (127, 0, 127));
    /// ```
    pub fn mix(&self, other: &Color, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        // Use floor instead of round for consistent results at 0.5
        let r = ((self.r as f32 * (1.0 - amount) + other.r as f32 * amount).floor()) as u8;
        let g = ((self.g as f32 * (1.0 - amount) + other.g as f32 * amount).floor()) as u8;
        let b = ((self.b as f32 * (1.0 - amount) + other.b as f32 * amount).floor()) as u8;
        let a = ((self.a as f32 * (1.0 - amount) + other.a as f32 * amount).floor()) as u8;
        Self::rgba(r, g, b, a)
    }

    /// Returns the inverse of the color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::Color;
    ///
    /// let white = Color::rgb(255, 255, 255);
    /// let black = white.invert();
    /// assert_eq!(black.rgb_components(), (0, 0, 0));
    /// ```
    pub fn invert(&self) -> Self {
        Self::rgba(255 - self.r, 255 - self.g, 255 - self.b, self.a)
    }
}

// Implement conversion to owo-colors RGB
impl From<Color> for owo_colors::Rgb {
    /// Converts the color to owo-colors RGB format.
    fn from(color: Color) -> Self {
        color.to_owo_rgb()
    }
}

// Implement conversion to ratatui Color
impl From<Color> for RatatuiColor {
    /// Converts the color to ratatui Color format.
    fn from(color: Color) -> Self {
        color.to_ratatui()
    }
}

impl fmt::Display for Color {
    /// Formats the color as a string representation.
    ///
    /// Returns either "rgb(r, g, b)" or "rgba(r, g, b, a)" format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            // Format alpha to exactly one decimal place
            let alpha = (self.a as f32 / 255.0 * 10.0).round() / 10.0;
            write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
        }
    }
}

/// Extension trait for applying theme colors to strings with owo-colors.
///
/// This trait provides convenient methods for applying semantic theme colors to text.
/// It is automatically implemented for all types that implement `OwoColorize`.
///
/// # Examples
///
/// ```rust
/// use oxitty::colors::ThemeColorize;
///
/// println!("{}", "Important message".primary());
/// println!("{}", "Warning alert".warning());
/// println!("{}", "Error occurred".error());
/// ```
pub trait ThemeColorize: OwoColorize {
    /// Apply primary text color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Main content".primary());
    /// ```
    #[inline]
    fn primary(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::text::PRIMARY.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply secondary text color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Additional info".secondary());
    /// ```
    #[inline]
    fn secondary(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::text::SECONDARY.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply info status color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Information".info());
    /// ```
    #[inline]
    fn info(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::INFO.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply warning status color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Warning message".warning());
    /// ```
    #[inline]
    fn warning(self) -> String
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::WARNING.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply error status color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Error message".error());
    /// ```
    #[inline]
    fn error(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::status::ERROR.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply v01d green color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Success".void_green());
    /// ```
    #[inline]
    fn void_green(self) -> impl fmt::Display
    where
        Self: Sized + Display,
    {
        let (r, g, b) = theme::void::GREEN.rgb_components();
        format!("{}", self.truecolor(r, g, b))
    }

    /// Apply v01d purple color.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxitty::colors::ThemeColorize;
    ///
    /// println!("{}", "Accent".void_purple());
    /// ```
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

/// Theme color constants and semantic color groupings.
pub mod theme {
    use super::Color;

    /// Background elevation hierarchy.
    pub mod background {
        use super::Color;

        /// Base background color.
        pub const BASE: Color = Color::rgb(15, 18, 20);
        /// First level elevation color.
        pub const ELEVATION_1: Color = Color::rgb(22, 27, 30);
        /// Second level elevation color.
        pub const ELEVATION_2: Color = Color::rgb(29, 36, 40);
        /// Third level elevation color.
        pub const ELEVATION_3: Color = Color::rgb(36, 43, 48);

        /// Creates a custom elevation level by interpolating between existing levels.
        ///
        /// # Arguments
        ///
        /// * `level` - Elevation level (0.0-3.0)
        ///
        /// # Examples
        ///
        /// ```rust
        /// use oxitty::colors::theme::background;
        ///
        /// let custom = background::custom_elevation(1.5);
        /// ```
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

    /// v01d colors and variants.
    pub mod void {
        use super::Color;

        /// Primary green color.
        pub const GREEN: Color = Color::rgb(0, 228, 154);
        /// Subtle green color (15% opacity).
        pub const GREEN_SUBTLE: Color = Color::rgba(0, 228, 154, 38);
        /// Primary purple color.
        pub const PURPLE: Color = Color::rgb(184, 110, 255);
        /// Subtle purple color (15% opacity).
        pub const PURPLE_SUBTLE: Color = Color::rgba(184, 110, 255, 38);

        /// Creates a custom variant of the v01d green color.
        ///
        /// # Arguments
        ///
        /// * `lightness` - Target lightness value (0-100)
        pub fn green_variant(lightness: f32) -> Color {
            let (h, s, _) = GREEN.to_hsl();
            Color::from_hsl(h, s, lightness)
        }

        /// Creates a custom variant of the v01d purple color.
        ///
        /// # Arguments
        ///
        /// * `lightness` - Target lightness value (0-100)
        pub fn purple_variant(lightness: f32) -> Color {
            let (h, s, _) = PURPLE.to_hsl();
            Color::from_hsl(h, s, lightness)
        }
    }

    /// Text hierarchy and variations.
    pub mod text {
        use super::Color;

        /// Primary text color.
        pub const PRIMARY: Color = Color::rgb(230, 237, 243);
        /// Secondary text color.
        pub const SECONDARY: Color = Color::rgb(139, 148, 158);
        /// Disabled text color.
        pub const DISABLED: Color = Color::rgb(106, 115, 125);
        /// Placeholder text color.
        pub const PLACEHOLDER: Color = Color::rgb(88, 96, 105);

        /// Creates a custom text color with specified opacity.
        ///
        /// # Arguments
        ///
        /// * `base` - Base color to modify
        /// * `opacity` - Target opacity (0-255)
        pub fn with_opacity(base: Color, opacity: u8) -> Color {
            base.with_alpha(opacity)
        }
    }

    /// System status colors and variants.
    pub mod status {
        use super::Color;

        /// Info status color.
        pub const INFO: Color = Color::rgb(41, 187, 255);
        /// Success status color.
        pub const SUCCESS: Color = Color::rgb(35, 209, 139);
        /// Warning status color.
        pub const WARNING: Color = Color::rgb(255, 191, 0);
        /// Error status color.
        pub const ERROR: Color = Color::rgb(255, 46, 95);

        /// Subtle variants for backgrounds
        pub const INFO_SUBTLE: Color = Color::rgba(41, 187, 255, 38);
        pub const SUCCESS_SUBTLE: Color = Color::rgba(35, 209, 139, 38);
        pub const WARNING_SUBTLE: Color = Color::rgba(255, 191, 0, 38);
        pub const ERROR_SUBTLE: Color = Color::rgba(255, 46, 95, 38);

        /// Creates a status color variant with custom intensity.
        ///
        /// # Arguments
        ///
        /// * `base` - Base status color
        /// * `intensity` - Target intensity (0-100)
        pub fn variant(base: Color, intensity: f32) -> Color {
            let (h, s, _) = base.to_hsl();
            Color::from_hsl(h, s, intensity)
        }
    }

    /// Base16 theme implementation for terminal compatibility.
    pub mod base16 {
        use super::Color;

        pub const BASE00: Color = super::background::BASE;
        pub const BASE01: Color = super::background::ELEVATION_1;
        pub const BASE02: Color = super::background::ELEVATION_2;
        pub const BASE03: Color = super::background::ELEVATION_3;
        pub const BASE04: Color = super::text::SECONDARY;
        pub const BASE05: Color = super::text::PRIMARY;
        pub const BASE06: Color = super::void::GREEN;
        pub const BASE07: Color = super::void::PURPLE;
        pub const BASE08: Color = super::status::ERROR;
        pub const BASE09: Color = super::status::WARNING;
        pub const BASE0A: Color = super::status::INFO;
        pub const BASE0B: Color = super::status::SUCCESS;
        pub const BASE0C: Color = super::void::GREEN;
        pub const BASE0D: Color = super::void::PURPLE;
        pub const BASE0E: Color = super::text::DISABLED;
        pub const BASE0F: Color = super::text::PLACEHOLDER;
    }

    /// Semantic color mapping for common UI elements.
    pub mod semantic {
        use super::Color;

        // Interactive elements
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
        /// Modal overlay with 70% opacity.
        pub const MODAL_OVERLAY: Color = Color::rgba(0, 0, 0, 178);
        /// Dropdown shadow with 80% opacity.
        pub const DROPDOWN_SHADOW: Color = Color::rgba(0, 0, 0, 204);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use theme::{background, semantic, status, void};

    /// Helper function to round HSL values for testing.
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

        // Test invalid hex strings
        assert!(Color::from_hex("invalid").is_none());
        assert!(Color::from_hex("#12345").is_none());
    }

    #[test]
    fn test_color_manipulation() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!((0.0, 100.0, 50.0), round_hsl(red.to_hsl()));

        let blue = Color::rgb(0, 0, 255);
        assert_eq!((240.0, 100.0, 50.0), round_hsl(blue.to_hsl()));

        let lightened = red.lighten(20.0);
        let (_, _, l) = lightened.to_hsl();
        assert!(l > 50.0);

        let darkened = red.darken(20.0);
        let (_, _, l) = darkened.to_hsl();
        assert!(l < 50.0);
    }

    #[test]
    fn test_color_mixing() {
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        let purple = red.mix(&blue, 0.5);

        let (r, g, b) = purple.rgb_components();
        assert_eq!(r, 127);
        assert_eq!(g, 0);
        assert_eq!(b, 127);

        // Test alpha mixing
        let transparent = Color::rgba(255, 0, 0, 128);
        let opaque = Color::rgb(0, 0, 255);
        let mixed = transparent.mix(&opaque, 0.5);
        let (_, _, _, a) = mixed.rgba_components();
        assert_eq!(a, 191); // (128 + 255) / 2 floored = 191
    }

    #[test]
    fn test_theme_colors() {
        assert_eq!(background::BASE.to_hex(), "#0f1214");
        assert_eq!(background::ELEVATION_1.to_hex(), "#161b1e");
        assert_eq!(background::ELEVATION_2.to_hex(), "#1d2428");
        assert_eq!(background::ELEVATION_3.to_hex(), "#242b30");

        assert_eq!(void::GREEN.to_hex(), "#00e49a");
        assert_eq!(void::PURPLE.to_hex(), "#b86eff");

        assert_eq!(status::INFO.to_hex(), "#29bbff");
        assert_eq!(status::WARNING.to_hex(), "#ffbf00");
        assert_eq!(status::ERROR.to_hex(), "#ff2e5f");
    }

    #[test]
    fn test_custom_elevation() {
        let custom = background::custom_elevation(1.5);
        let (_, _, l) = custom.to_hsl();
        let (_, _, l1) = background::ELEVATION_1.to_hsl();
        let (_, _, l2) = background::ELEVATION_2.to_hsl();
        assert!(l > l1 && l < l2);
    }

    #[test]
    fn test_semantic_colors() {
        let normal = semantic::BUTTON;
        let hover = semantic::button_hover();
        let (_, _, l1) = normal.to_hsl();
        let (_, _, l2) = hover.to_hsl();
        assert!(l2 > l1);

        assert_eq!(semantic::MODAL_OVERLAY.a, 178);
        assert_eq!(semantic::DROPDOWN_SHADOW.a, 204);
    }

    #[test]
    fn test_theme_colorize() {
        let text = "Test";
        let primary = text.primary().to_string();
        let error = text.error().to_string();
        let warning = text.warning();

        assert!(primary.contains("\x1b["));
        assert!(error.contains("\x1b["));
        assert!(warning.contains("\x1b["));
    }

    #[test]
    fn test_color_display() {
        let rgb = Color::rgb(255, 128, 64);
        assert_eq!(rgb.to_string(), "rgb(255, 128, 64)");

        let rgba = Color::rgba(255, 128, 64, 128);
        assert_eq!(rgba.to_string(), "rgba(255, 128, 64, 0.5)");
    }
}
