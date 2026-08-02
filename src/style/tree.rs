use super::{Color, Style, stdout_is_styled};

/// 계층적 트리 구조를 Unicode 박스 문자로 렌더링.
///
/// rich 라이브러리의 `Tree`에서 영감을 받음.
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

    /// 이 노드의 레이블에 적용할 스타일 (기본값: 청록색).
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// 자식 노드 추가.
    pub fn child(mut self, child: Tree) -> Self {
        self.children.push(child);
        self
    }

    /// stdout 이 TTY인지 자동 감지해서 출력.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// 트리를 `String`으로 렌더링.
    ///
    /// `styled = true`이면 ANSI 이스케이프 시퀀스 포함.
    pub fn render(&self, styled: bool) -> String {
        let mut buf = String::new();
        buf.push_str(&self.style.apply(&self.label, styled));
        buf.push('\n');
        self.render_children(&self.children, &mut buf, styled, "");
        buf
    }

    /// `prefix`는 조상 노드에서 내려온 들여쓰기 표시(수직선 포함).
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
