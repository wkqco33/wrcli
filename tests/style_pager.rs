//! PAGER 연동 테스트.

mod common;

use common::EnvGuard;
use wrcli::style::pager;

#[test]
fn page_falls_back_to_plain_output_when_not_a_tty() {
    // 테스트 stdout은 TTY가 아니므로 페이저를 띄우지 않고 그대로 출력한다.
    pager::page("pager fallback\n").unwrap();
}

#[test]
fn page_ignores_pager_env_when_not_a_tty() {
    // PAGER가 잘못된 명령이어도 비TTY에서는 무시되어야 한다.
    let _env = EnvGuard::set("PAGER", "definitely-not-a-real-pager-xyz");
    pager::page("still fine\n").unwrap();
}
