use super::{Color, Style, stdout_is_styled};

/// Renders a hierarchical tree structure with Unicode box characters.
///
/// Inspired by the rich library's `Tree`.
///
/// # Example
///
/// ```
/// use wrcli::style::Tree;
///
/// let tree = Tree::new("root")
///     .child(Tree::new("child1"))
///     .child(Tree::new("child2").child(Tree::new("grandchild")));
///
/// let out = tree.render(false);
/// assert!(out.contains("root"));
/// assert!(out.contains("grandchild"));
/// ```
pub struct Tree {
    label: String,
    style: Style,
    children: Vec<Tree>,
}

impl Tree {
    pub fn new(label: &str) -> Self {
        Tree {
            label: label.to_owned(),
            style: Style::new().fg(Color::Cyan),
            children: Vec::new(),
        }
    }

    /// Style applied to this node's label (default: cyan).
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Adds a child node.
    pub fn child(mut self, child: Tree) -> Self {
        self.children.push(child);
        self
    }

    /// Prints, auto-detecting whether stdout is a TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the tree as a `String`.
    ///
    /// When `styled = true`, includes ANSI escape sequences.
    pub fn render(&self, styled: bool) -> String {
        let mut buf = String::new();
        buf.push_str(&self.style.apply(&self.label, styled));
        buf.push('\n');
        self.render_children(&self.children, &mut buf, styled, "");
        buf
    }

    /// `prefix` is the indentation marker inherited from ancestor nodes (including the vertical lines).
    fn render_children(&self, children: &[Tree], buf: &mut String, styled: bool, prefix: &str) {
        let child_count = children.len();
        for (i, child) in children.iter().enumerate() {
            let last = i == child_count - 1;
            let branch = if last { "└── " } else { "├── " };
            buf.push_str(&self.style.apply(prefix, styled));
            buf.push_str(&self.style.apply(branch, styled));
            buf.push_str(&child.style.apply(&child.label, styled));
            buf.push('\n');
            let child_prefix = format!("{}{}", prefix, if last { "    " } else { "│   " });
            self.render_children(&child.children, buf, styled, &child_prefix);
        }
    }
}
