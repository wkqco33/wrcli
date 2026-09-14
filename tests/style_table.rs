use wrcli::style::{BoxStyle, Color, Style, Table};

#[test]
fn table_custom_box_style_rounded() {
    let out = Table::new()
        .box_style(BoxStyle::Rounded)
        .headers(["Name", "Age"])
        .row(["Alice", "30"])
        .render(false);
    assert!(out.contains('╭'));
    assert!(out.contains('╮'));
    assert!(out.contains('╰'));
    assert!(out.contains('╯'));
}

#[test]
fn table_custom_box_style_double() {
    let out = Table::new()
        .box_style(BoxStyle::Double)
        .headers(["Name"])
        .row(["Bob"])
        .render(false);
    assert!(out.contains('╔'));
    assert!(out.contains('═'));
    assert!(out.contains('║'));
    assert!(out.contains('╝'));
}

#[test]
fn table_row_separator_disabled() {
    let out = Table::new()
        .row_separator(false)
        .headers(["ID", "Val"])
        .row(["1", "A"])
        .row(["2", "B"])
        .render(false);
    let lines: Vec<&str> = out.lines().collect();
    let hsep_count = lines
        .iter()
        .filter(|l| l.contains('├') || l.contains('┼'))
        .count();
    assert_eq!(
        hsep_count, 1,
        "expected only header separator, but got {hsep_count}"
    );
}

#[test]
fn table_border_style_applied() {
    let out = Table::new()
        .border_style(Style::new().fg(Color::Yellow))
        .headers(["A"])
        .row(["1"])
        .render(true);
    assert!(out.contains("\x1b[33m"));
}

#[test]
fn table_korean_and_english_mixed_alignment() {
    use wrcli::style::Align;

    let out = Table::new()
        .headers(["도구 이름 (Tool)", "버전 (Ver)", "설명 (Description)"])
        .row(["wrcli 도구", "0.4.0", "Cobra/Viper 스타일 CLI 프레임워크"])
        .row(["tokio 런타임", "1.38", "비동기 async runtime 엔진"])
        .row(["serde 직렬화", "1.0", "Fast serialization 라이브러리"])
        .row(["clap", "4.5", "Argument Parser 라이브러리"])
        .row(["🚀 wrcli 툴", "v0.4.0", "✨ 놀라운 CLI 프레임워크"])
        .align(vec![Align::Left, Align::Center, Align::Right])
        .render(false);

    let border_positions: Vec<Vec<usize>> = out
        .lines()
        .filter(|line| line.contains('│'))
        .map(|line| {
            let mut pos = 0usize;
            let mut col_positions = Vec::new();
            for c in line.chars() {
                if c == '│' {
                    col_positions.push(pos);
                }
                pos += wrcli::style::display_width(&c.to_string());
            }
            col_positions
        })
        .collect();

    assert!(
        border_positions.len() >= 4,
        "expected at least 4 border lines"
    );
    let first = &border_positions[0];
    for (idx, bp) in border_positions.iter().enumerate() {
        assert_eq!(
            bp, first,
            "line {idx} border positions {bp:?} do not match header line {first:?}"
        );
    }
}
