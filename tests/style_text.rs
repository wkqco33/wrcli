use wrcli::style::{Color, Style, Text};

#[test]
fn render_concatenates_plain_spans() {
    let text = Text::new().plain("Hello ").plain("World");
    assert_eq!(text.render(false), "Hello World");
}

#[test]
fn render_empty() {
    let text = Text::new();
    assert_eq!(text.render(false), "");
}

#[test]
fn styled_span_emits_ansi_when_styled() {
    let text = Text::new().span("warning", Style::new().fg(Color::Yellow).bold());
    let out = text.render(true);
    assert!(out.contains("\x1b["));
    assert!(out.contains("warning"));
}

#[test]
fn styled_span_passthrough_when_unstyled() {
    let text = Text::new().span("warning", Style::new().fg(Color::Yellow));
    assert_eq!(text.render(false), "warning");
}

#[test]
fn mixed_plain_and_styled() {
    let text = Text::new()
        .plain("Error: ")
        .span("boom", Style::new().fg(Color::Red));
    let out = text.render(false);
    assert_eq!(out, "Error: boom");
}

#[test]
fn plain_with_style() {
    let text = Text::new().plain_styled("bold text", Style::new().bold());
    let out = text.render(true);
    assert!(out.contains("\x1b["));
    assert!(out.contains("bold text"));
}
