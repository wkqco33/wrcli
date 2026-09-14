use wrcli::style::{Align, Rule};

#[test]
fn rule_default_is_center_align() {
    let out = Rule::new().title("X").width(11).render(false);
    // 11 - 3 = 8 -> 4 left, 4 right
    let expected = format!("{} X {}", "─".repeat(4), "─".repeat(4));
    assert_eq!(out, expected);
}

#[test]
fn rule_align_left() {
    let out = Rule::new()
        .title("Left")
        .width(20)
        .align(Align::Left)
        .render(false);
    // " Left " has width 6. 20 - 6 = 14 remaining.
    // left gets 2 dashes, right gets 12 dashes.
    let expected = format!("{} Left {}", "─".repeat(2), "─".repeat(12));
    assert_eq!(out, expected);
}

#[test]
fn rule_align_right() {
    let out = Rule::new()
        .title("Right")
        .width(20)
        .align(Align::Right)
        .render(false);
    // " Right " has width 7. 20 - 7 = 13 remaining.
    // right gets 2 dashes, left gets 11 dashes.
    let expected = format!("{} Right {}", "─".repeat(11), "─".repeat(2));
    assert_eq!(out, expected);
}
