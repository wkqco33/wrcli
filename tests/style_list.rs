use wrcli::style::{Color, List, ListMarker, Style};

#[test]
fn list_default_bullet() {
    let out = List::new().item("First").item("Second").render(false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "• First");
    assert_eq!(lines[1], "• Second");
}

#[test]
fn list_numbered() {
    let out = List::new()
        .marker(ListMarker::Numbered)
        .item("Step 1")
        .item("Step 2")
        .render(false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "1. Step 1");
    assert_eq!(lines[1], "2. Step 2");
}

#[test]
fn list_nested_sublist() {
    let out = List::new()
        .item("Parent")
        .sublist(List::new().item("Child"))
        .render(false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "• Parent");
    assert_eq!(lines[1], "  • Child");
}

#[test]
fn list_styled() {
    let out = List::new()
        .marker_style(Style::new().fg(Color::Cyan))
        .item("Styled item")
        .render(true);
    assert!(out.contains("\x1b[36m"));
    assert!(out.contains("Styled item"));
}
