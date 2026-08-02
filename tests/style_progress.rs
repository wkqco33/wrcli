use wrcli::style::{Color, Progress, Style};

#[test]
fn zero_progress() {
    let p = Progress::new(100).width(10).render(false);
    assert_eq!(p, "[----------]   0%");
}

#[test]
fn full_progress() {
    let p = Progress::new(100).progress(100).width(10).render(false);
    assert_eq!(p, "[##########] 100%");
}

#[test]
fn partial_progress() {
    let p = Progress::new(4).progress(2).width(8).render(false);
    assert!(p.contains("[####----]"));
    assert!(p.contains("50%"));
}

#[test]
fn clamp_above_total() {
    let p = Progress::new(10).progress(100).width(5).render(false);
    assert!(p.contains("100%"));
}

#[test]
fn with_label() {
    let p = Progress::new(10)
        .progress(5)
        .label("Downloading")
        .render(false);
    assert!(p.starts_with("Downloading"));
    assert!(p.contains("50%"));
}

#[test]
fn ansi_codes_when_styled() {
    let p = Progress::new(10)
        .progress(5)
        .bar_style(Style::new().fg(Color::Green))
        .render(true);
    assert!(p.contains("\x1b["));
}
