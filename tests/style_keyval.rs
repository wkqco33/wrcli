use wrcli::style::{Color, KeyVal, Style};

#[test]
fn keyval_plain_alignment() {
    let out = KeyVal::new()
        .entry("Host", "127.0.0.1")
        .entry("Port", "8080")
        .render(false);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "Host : 127.0.0.1");
    assert_eq!(lines[1], "Port : 8080");
}

#[test]
fn keyval_custom_separator() {
    let out = KeyVal::new()
        .separator(" => ")
        .entry("k1", "v1")
        .render(false);
    assert_eq!(out.trim_end(), "k1 => v1");
}

#[test]
fn keyval_cjk_alignment() {
    let out = KeyVal::new()
        .entry("이름", "wrcli")
        .entry("id", "1")
        .render(false);
    let lines: Vec<&str> = out.lines().collect();
    // "이름" has width 4. "id" has width 2, so padded with 2 spaces -> "id  "
    assert_eq!(lines[0], "이름 : wrcli");
    assert_eq!(lines[1], "id   : 1");
}

#[test]
fn keyval_styled() {
    let out = KeyVal::new()
        .key_style(Style::new().bold().fg(Color::Cyan))
        .entry("Key", "Value")
        .render(true);
    assert!(out.contains("\x1b["));
    assert!(out.contains("Key"));
    assert!(out.contains("Value"));
}
