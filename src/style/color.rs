/// A terminal foreground or background color.
///
/// Supports the 16 standard ANSI colors, the 8-bit (256-color) fixed palette, and 24-bit RGB truecolor.
use std::fmt::Write as _;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// 8-bit (256-color) terminal color index.
    Fixed(u8),
    /// 24-bit RGB truecolor.
    Rgb(u8, u8, u8),
}

impl Color {
    pub(crate) fn fg_code(&self) -> &'static str {
        match self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::BrightBlack => "90",
            Color::BrightRed => "91",
            Color::BrightGreen => "92",
            Color::BrightYellow => "93",
            Color::BrightBlue => "94",
            Color::BrightMagenta => "95",
            Color::BrightCyan => "96",
            Color::BrightWhite => "97",
            Color::Fixed(_) | Color::Rgb(..) => "",
        }
    }

    pub(crate) fn write_fg_code(self, out: &mut String) {
        match self {
            Color::Fixed(n) => {
                let _ = write!(out, "38;5;{}", n);
            }
            Color::Rgb(r, g, b) => {
                let _ = write!(out, "38;2;{};{};{}", r, g, b);
            }
            _ => out.push_str(self.fg_code()),
        }
    }

    pub(crate) fn bg_code(&self) -> &'static str {
        match self {
            Color::Black => "40",
            Color::Red => "41",
            Color::Green => "42",
            Color::Yellow => "43",
            Color::Blue => "44",
            Color::Magenta => "45",
            Color::Cyan => "46",
            Color::White => "47",
            Color::BrightBlack => "100",
            Color::BrightRed => "101",
            Color::BrightGreen => "102",
            Color::BrightYellow => "103",
            Color::BrightBlue => "104",
            Color::BrightMagenta => "105",
            Color::BrightCyan => "106",
            Color::BrightWhite => "107",
            Color::Fixed(_) | Color::Rgb(..) => "",
        }
    }

    pub(crate) fn write_bg_code(self, out: &mut String) {
        match self {
            Color::Fixed(n) => {
                let _ = write!(out, "48;5;{}", n);
            }
            Color::Rgb(r, g, b) => {
                let _ = write!(out, "48;2;{};{};{}", r, g, b);
            }
            _ => out.push_str(self.bg_code()),
        }
    }

    /// Parses a color from a string.
    ///
    /// - Standard color names: `"red"`, `"bright_cyan"` (or `"bright cyan"`)
    /// - 8-bit index: `"42"`
    /// - 24-bit hex: `"#ff0000"`
    /// - 24-bit rgb: `"rgb(0,128,255)"`
    ///
    /// Returns `None` if parsing fails.
    pub fn from_name(s: &str) -> Option<Color> {
        let trimmed = s.trim();
        let normalized = trimmed.to_ascii_lowercase();
        let color = match normalized.as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "bright_black" | "bright black" => Color::BrightBlack,
            "bright_red" | "bright red" => Color::BrightRed,
            "bright_green" | "bright green" => Color::BrightGreen,
            "bright_yellow" | "bright yellow" => Color::BrightYellow,
            "bright_blue" | "bright blue" => Color::BrightBlue,
            "bright_magenta" | "bright magenta" => Color::BrightMagenta,
            "bright_cyan" | "bright cyan" => Color::BrightCyan,
            "bright_white" | "bright white" => Color::BrightWhite,
            _ => return parse_numeric(trimmed),
        };
        Some(color)
    }
}

/// Parses a color string in 8-bit index, hex, or rgb() form.
fn parse_numeric(s: &str) -> Option<Color> {
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    if let Some(inner) = s.strip_prefix("rgb(").and_then(|x| x.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            let r = parts[0].parse().ok()?;
            let g = parts[1].parse().ok()?;
            let b = parts[2].parse().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    s.parse().ok().map(Color::Fixed)
}

fn parse_hex(hex: &str) -> Option<Color> {
    let clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fg_codes_are_distinct() {
        assert_ne!(Color::Red.fg_code(), Color::Green.fg_code());
        assert_ne!(Color::Blue.fg_code(), Color::Cyan.fg_code());
    }

    #[test]
    fn rgb_code_format() {
        let mut fg = String::new();
        Color::Rgb(10, 20, 30).write_fg_code(&mut fg);
        assert_eq!(fg, "38;2;10;20;30");
        let mut fixed = String::new();
        Color::Fixed(42).write_fg_code(&mut fixed);
        assert_eq!(fixed, "38;5;42");
    }

    #[test]
    fn bg_codes_differ_from_fg() {
        assert_ne!(Color::Red.fg_code(), Color::Red.bg_code());
        assert_ne!(Color::BrightGreen.fg_code(), Color::BrightGreen.bg_code());
    }

    #[test]
    fn from_name_parses_standard_colors() {
        assert_eq!(Color::from_name("red"), Some(Color::Red));
        assert_eq!(Color::from_name("bright_cyan"), Some(Color::BrightCyan));
        assert_eq!(Color::from_name("bright cyan"), Some(Color::BrightCyan));
        assert_eq!(Color::from_name("nope"), None);
    }

    #[test]
    fn from_name_parses_fixed_and_rgb() {
        assert_eq!(Color::from_name("42"), Some(Color::Fixed(42)));
        assert_eq!(Color::from_name("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(
            Color::from_name("rgb(0,128,255)"),
            Some(Color::Rgb(0, 128, 255))
        );
        assert_eq!(Color::from_name("notacolor"), None);
    }
}
