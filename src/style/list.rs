use super::{Color, Style, stdout_is_styled};

/// Marker style for [`List`] items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListMarker {
    /// Bullet point: `• `
    #[default]
    Bullet,
    /// Dash: `- `
    Dash,
    /// Arrow: `→ `
    Arrow,
    /// 1-based decimal numbering: `1. `, `2. `
    Numbered,
}

enum ListItem {
    Text(String),
    Sublist(List),
}

/// Renders bulleted or numbered lists with optional nesting.
///
/// # Example
///
/// ```
/// use wrcli::style::{List, ListMarker};
///
/// let out = List::new()
///     .item("Apples")
///     .item("Oranges")
///     .render(false);
///
/// assert!(out.contains("• Apples"));
/// ```
pub struct List {
    items: Vec<ListItem>,
    marker: ListMarker,
    marker_style: Style,
    item_style: Style,
}

impl List {
    pub fn new() -> Self {
        List {
            items: Vec::new(),
            marker: ListMarker::Bullet,
            marker_style: Style::new().fg(Color::Cyan),
            item_style: Style::new(),
        }
    }

    /// Appends a text item to the list.
    pub fn item<S: Into<String>>(mut self, s: S) -> Self {
        self.items.push(ListItem::Text(s.into()));
        self
    }

    /// Appends a nested sublist.
    pub fn sublist(mut self, sub: List) -> Self {
        self.items.push(ListItem::Sublist(sub));
        self
    }

    /// Sets the list marker style.
    pub fn marker(mut self, m: ListMarker) -> Self {
        self.marker = m;
        self
    }

    /// Sets the style for the list markers.
    pub fn marker_style(mut self, s: Style) -> Self {
        self.marker_style = s;
        self
    }

    /// Sets the style for item texts.
    pub fn item_style(mut self, s: Style) -> Self {
        self.item_style = s;
        self
    }

    /// Prints the list to stdout, auto-detecting TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the list as a `String`.
    pub fn render(&self, styled: bool) -> String {
        let mut buf = String::new();
        self.render_depth(&mut buf, styled, 0);
        buf
    }

    fn render_depth(&self, buf: &mut String, styled: bool, depth: usize) {
        let indent = "  ".repeat(depth);
        let mut item_index = 1;
        for item in &self.items {
            match item {
                ListItem::Text(text) => {
                    let marker_str = match self.marker {
                        ListMarker::Bullet => "• ".to_owned(),
                        ListMarker::Dash => "- ".to_owned(),
                        ListMarker::Arrow => "→ ".to_owned(),
                        ListMarker::Numbered => format!("{}. ", item_index),
                    };
                    buf.push_str(&indent);
                    buf.push_str(&self.marker_style.apply(&marker_str, styled));
                    buf.push_str(&self.item_style.apply(text, styled));
                    buf.push('\n');
                    item_index += 1;
                }
                ListItem::Sublist(sub) => {
                    sub.render_depth(buf, styled, depth + 1);
                }
            }
        }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}
