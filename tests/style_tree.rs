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
