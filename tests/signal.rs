//! Tests for the clig.dev Signals rules: notify immediately on Ctrl-C (SIGINT), then exit.
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
    // SAFETY: child.pid() is a live child process and SIGINT is a default signal.
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(130));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("interrupted by test"), "stderr: {stderr}");
}
