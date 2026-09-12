//! clig.dev Signals 규칙 테스트: Ctrl-C(SIGINT) 시 즉시 안내 후 종료.
#![cfg(all(unix, feature = "signal"))]

use std::process::{Command, Stdio};
use std::time::Duration;

fn testapp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_testapp"))
}

#[test]
fn sigint_prints_message_and_exits_130() {
    let child = testapp()
        .arg("sleep")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    // SAFETY: child.pid()는 살아 있는 자식 프로세스이며 SIGINT는 기본 신호다.
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(130));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("interrupted by test"), "stderr: {stderr}");
}
