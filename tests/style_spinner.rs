use wrcli::style::{Color, Spinner, Style};

#[test]
fn spinner_render() {
    let s = Spinner::new("Loading...");
    let out = s.render(false);
    assert!(out.contains("⠋"));
    assert!(out.contains("Loading..."));
}

#[test]
fn spinner_tick_advances_frame() {
    let mut s = Spinner::new("Processing");
    assert!(s.render(false).contains("⠋"));
    s.step();
    assert!(s.render(false).contains("⠙"));
    s.step();
    assert!(s.render(false).contains("⠹"));
}

#[test]
fn spinner_styled() {
    let s = Spinner::new("Building").style(Style::new().fg(Color::Green));
    let out = s.render(true);
    assert!(out.contains("\x1b[32m"));
}

#[test]
fn spinner_safe_without_tty() {
    let mut s = Spinner::new("Working");
    s.tick();
    s.finish();
    s.finish_with_message("Done!");
}
