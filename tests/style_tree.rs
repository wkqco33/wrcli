use wrcli::style::{Color, Style, Tree};

#[test]
fn render_contains_labels() {
    let tree = Tree::new("root")
        .child(Tree::new("child1"))
        .child(Tree::new("child2"));
    let out = tree.render(false);
    assert!(out.contains("root"));
    assert!(out.contains("child1"));
    assert!(out.contains("child2"));
}

#[test]
fn render_nested_structure() {
    let tree = Tree::new("root").child(Tree::new("mid").child(Tree::new("leaf")));
    let out = tree.render(false);
    assert!(out.contains("mid"));
    assert!(out.contains("leaf"));
}

#[test]
fn tree_uses_branch_chars() {
    let tree = Tree::new("root")
        .child(Tree::new("kid1"))
        .child(Tree::new("kid2"));
    let out = tree.render(false);
    assert!(out.contains('├'));
    assert!(out.contains('└'));
}

#[test]
fn ansi_codes_when_styled() {
    let tree = Tree::new("root").child(Tree::new("kid"));
    let out = tree.render(true);
    assert!(out.contains("\x1b["));
}

#[test]
fn default_style_applied_when_styled() {
    let tree = Tree::new("root")
        .style(Style::new().fg(Color::Green))
        .child(Tree::new("kid"));
    let out = tree.render(true);
    assert!(out.contains("\x1b["));
    assert!(out.contains("kid"));
}

#[test]
fn guide_style_applied() {
    let tree = Tree::new("root")
        .style(Style::new().fg(Color::Green))
        .guide_style(Style::new().fg(Color::Yellow))
        .child(Tree::new("kid"));
    let out = tree.render(true);
    let yellow_branch = Style::new().fg(Color::Yellow).apply("└── ", true);
    assert!(out.contains(&yellow_branch));
}

#[test]
fn multiline_label_indented() {
    let tree = Tree::new("root")
        .child(Tree::new("line1\nline2"))
        .child(Tree::new("kid2"));
    let out = tree.render(false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "root");
    assert_eq!(lines[1], "├── line1");
    assert_eq!(lines[2], "│   line2");
    assert_eq!(lines[3], "└── kid2");
}
