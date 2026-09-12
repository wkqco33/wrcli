//! Tests for PAGER integration.

mod common;

use common::EnvGuard;
use wrcli::style::pager;

#[test]
fn page_falls_back_to_plain_output_when_not_a_tty() {
    // The test stdout is not a TTY, so output is written as-is without launching the pager.
    pager::page("pager fallback\n").unwrap();
}

#[test]
fn page_ignores_pager_env_when_not_a_tty() {
    // Even if PAGER is an invalid command, it must be ignored when not a TTY.
    let _env = EnvGuard::set("PAGER", "definitely-not-a-real-pager-xyz");
    pager::page("still fine\n").unwrap();
}
