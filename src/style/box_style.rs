/// Characters used to draw box borders in [`Table`](crate::style::Table) and [`Panel`](crate::style::Panel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxStyle {
    /// Rectangular borders with sharp corners: `┌─┐│└─┘`
    #[default]
    Square,
    /// Smooth rounded corners: `╭─╮│╰─╯`
    Rounded,
    /// Double-line borders: `╔═╗║╚═╝`
    Double,
    /// Heavy (bold) single-line borders: `┏━┓┃┗━┛`
    Heavy,
    /// Plain ASCII characters: `+-+|+-+`
    Ascii,
    /// Markdown-compatible table format (uses `|` and `-`).
    Markdown,
}

impl BoxStyle {
    pub fn top_left(self) -> char {
        match self {
            BoxStyle::Square => '┌',
            BoxStyle::Rounded => '╭',
            BoxStyle::Double => '╔',
            BoxStyle::Heavy => '┏',
            BoxStyle::Ascii => '+',
            BoxStyle::Markdown => '|',
        }
    }

    pub fn top_right(self) -> char {
        match self {
            BoxStyle::Square => '┐',
            BoxStyle::Rounded => '╮',
            BoxStyle::Double => '╗',
            BoxStyle::Heavy => '┓',
            BoxStyle::Ascii => '+',
            BoxStyle::Markdown => '|',
        }
    }

    pub fn bottom_left(self) -> char {
        match self {
            BoxStyle::Square => '└',
            BoxStyle::Rounded => '╰',
            BoxStyle::Double => '╚',
            BoxStyle::Heavy => '┗',
            BoxStyle::Ascii => '+',
            BoxStyle::Markdown => '|',
        }
    }

    pub fn bottom_right(self) -> char {
        match self {
            BoxStyle::Square => '┘',
            BoxStyle::Rounded => '╯',
            BoxStyle::Double => '╝',
            BoxStyle::Heavy => '┛',
            BoxStyle::Ascii => '+',
            BoxStyle::Markdown => '|',
        }
    }

    pub fn horizontal(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '─',
            BoxStyle::Double => '═',
            BoxStyle::Heavy => '━',
            BoxStyle::Ascii | BoxStyle::Markdown => '-',
        }
    }

    pub fn vertical(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '│',
            BoxStyle::Double => '║',
            BoxStyle::Heavy => '┃',
            BoxStyle::Ascii | BoxStyle::Markdown => '|',
        }
    }

    pub fn t_top(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '┬',
            BoxStyle::Double => '╦',
            BoxStyle::Heavy => '┳',
            BoxStyle::Ascii | BoxStyle::Markdown => '+',
        }
    }

    pub fn t_bottom(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '┴',
            BoxStyle::Double => '╩',
            BoxStyle::Heavy => '┻',
            BoxStyle::Ascii | BoxStyle::Markdown => '+',
        }
    }

    pub fn t_left(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '├',
            BoxStyle::Double => '╠',
            BoxStyle::Heavy => '┣',
            BoxStyle::Ascii | BoxStyle::Markdown => '+',
        }
    }

    pub fn t_right(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '┤',
            BoxStyle::Double => '╣',
            BoxStyle::Heavy => '┫',
            BoxStyle::Ascii | BoxStyle::Markdown => '+',
        }
    }

    pub fn cross(self) -> char {
        match self {
            BoxStyle::Square | BoxStyle::Rounded => '┼',
            BoxStyle::Double => '╬',
            BoxStyle::Heavy => '╋',
            BoxStyle::Ascii | BoxStyle::Markdown => '+',
        }
    }
}
