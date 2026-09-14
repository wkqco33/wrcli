use wrcli::style::{Align, BoxStyle, Color, Panel, Style};

#[test]
fn panel_custom_box_style_square() {
    let out = Panel::new("hello")
        .box_style(BoxStyle::Square)
        .render(false);
    assert!(out.contains('┌'));
    assert!(out.contains('┐'));
    assert!(out.contains('└'));
    assert!(out.contains('┘'));
}

#[test]
fn panel_custom_box_style_double() {
    let out = Panel::new("hello")
        .box_style(BoxStyle::Double)
        .render(false);
    assert!(out.contains('╔'));
    assert!(out.contains('═'));
    assert!(out.contains('║'));
    assert!(out.contains('╝'));
}

#[test]
fn panel_content_align_center() {
    let out = Panel::new("hi")
        .width(10)
        .padding(0)
        .content_align(Align::Center)
        .render(false);
    // inner_width is 10. "hi" is 2. 8 spaces left.
    // 4 on left, 4 on right.
    assert!(out.contains("│    hi    │"));
}

#[test]
fn panel_subtitle() {
    let out = Panel::new("body")
        .title("Title")
        .subtitle("v1.0")
        .subtitle_style(Style::new().fg(Color::Green))
        .render(false);
    assert!(out.contains("Title"));
    assert!(out.contains("v1.0"));
    assert!(out.contains('╭'));
    assert!(out.contains('╰'));
}

#[test]
fn panel_korean_and_english_mixed_alignment() {
    let out = Panel::new(
        "wrcli 라이브러리 v0.4.0\nCLI 프레임워크 툴킷\nShort line\n🚀 배포 시작: ✨ 성공",
    )
    .title("배포 알림 (Deployment Notice)")
    .subtitle("status: 정상")
    .render(false);

    let right_borders: Vec<usize> = out
        .lines()
        .map(|line| {
            let mut pos = 0usize;
            let mut right_border = 0;
            for c in line.chars() {
                if c == '│' || c == '╮' || c == '╯' {
                    right_border = pos;
                }
                pos += wrcli::style::display_width(&c.to_string());
            }
            right_border
        })
        .collect();

    assert!(right_borders.len() >= 5);
    let first = right_borders[0];
    for (i, &r) in right_borders.iter().enumerate() {
        assert_eq!(r, first, "line {i} right border at {r}, expected {first}");
    }
}
