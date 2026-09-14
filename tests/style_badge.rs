use wrcli::style::{Badge, Color, Style};

#[test]
fn badge_success_plain() {
    let b = Badge::success("DONE");
    assert_eq!(b.render(false), "[DONE]");
}

#[test]
fn badge_error_styled() {
    let b = Badge::error("FAIL");
    let out = b.render(true);
    assert!(out.contains("\x1b["));
    assert!(out.contains("FAIL"));
}

#[test]
fn badge_custom_brackets() {
    let b = Badge::info("v1.0").brackets('(', ')');
    assert_eq!(b.render(false), "(v1.0)");
}

#[test]
fn badge_no_brackets() {
    let b = Badge::new("ACTIVE")
        .style(Style::new().fg(Color::Black).bg(Color::Green))
        .no_brackets();
    assert_eq!(b.render(false), " ACTIVE ");
}
