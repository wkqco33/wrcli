/// 터미널 전경색 또는 배경색.
///
/// 16개 표준 ANSI 색상, 8비트(256색) 고정 팔레트, 24비트 RGB 트루컬러를 지원.
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
    /// 8비트(256색) 터미널 색상 인덱스.
    Fixed(u8),
    /// 24비트 RGB 트루컬러.
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

    pub(crate) fn fg_code_owned(self) -> String {
        match self {
            Color::Fixed(n) => format!("38;5;{}", n),
            Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
            _ => self.fg_code().to_owned(),
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

    pub(crate) fn bg_code_owned(self) -> String {
        match self {
            Color::Fixed(n) => format!("48;5;{}", n),
            Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
            _ => self.bg_code().to_owned(),
        }
    }

    /// 문자열에서 색상을 파싱.
    ///
    /// - 표준 색상 이름: `"red"`, `"bright_cyan"`(또는 `"bright cyan"`)
    /// - 8비트 인덱스: `"42"`
    /// - 24비트 hex: `"#ff0000"`
    /// - 24비트 rgb: `"rgb(0,128,255)"`
    ///
    /// 파싱 실패 시 `None` 반환.
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

/// 8비트 인덱스, hex, 또는 rgb() 형식의 색상 문자열을 파싱.
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
        assert_eq!(Color::Rgb(10, 20, 30).fg_code_owned(), "38;2;10;20;30");
        assert_eq!(Color::Fixed(42).fg_code_owned(), "38;5;42");
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
