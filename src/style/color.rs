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
    pub(crate) fn fg_code(self) -> String {
        match self {
            Color::Black => "30".to_owned(),
            Color::Red => "31".to_owned(),
            Color::Green => "32".to_owned(),
            Color::Yellow => "33".to_owned(),
            Color::Blue => "34".to_owned(),
            Color::Magenta => "35".to_owned(),
            Color::Cyan => "36".to_owned(),
            Color::White => "37".to_owned(),
            Color::BrightBlack => "90".to_owned(),
            Color::BrightRed => "91".to_owned(),
            Color::BrightGreen => "92".to_owned(),
            Color::BrightYellow => "93".to_owned(),
            Color::BrightBlue => "94".to_owned(),
            Color::BrightMagenta => "95".to_owned(),
            Color::BrightCyan => "96".to_owned(),
            Color::BrightWhite => "97".to_owned(),
            Color::Fixed(n) => format!("38;5;{}", n),
            Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
        }
    }

    pub(crate) fn bg_code(self) -> String {
        match self {
            Color::Black => "40".to_owned(),
            Color::Red => "41".to_owned(),
            Color::Green => "42".to_owned(),
            Color::Yellow => "43".to_owned(),
            Color::Blue => "44".to_owned(),
            Color::Magenta => "45".to_owned(),
            Color::Cyan => "46".to_owned(),
            Color::White => "47".to_owned(),
            Color::BrightBlack => "100".to_owned(),
            Color::BrightRed => "101".to_owned(),
            Color::BrightGreen => "102".to_owned(),
            Color::BrightYellow => "103".to_owned(),
            Color::BrightBlue => "104".to_owned(),
            Color::BrightMagenta => "105".to_owned(),
            Color::BrightCyan => "106".to_owned(),
            Color::BrightWhite => "107".to_owned(),
            Color::Fixed(n) => format!("48;5;{}", n),
            Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
        }
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
        assert_eq!(Color::Rgb(10, 20, 30).fg_code(), "38;2;10;20;30");
        assert_eq!(Color::Fixed(42).fg_code(), "38;5;42");
    }

    #[test]
    fn bg_codes_differ_from_fg() {
        assert_ne!(Color::Red.fg_code(), Color::Red.bg_code());
    }
}
