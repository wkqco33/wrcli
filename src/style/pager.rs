//! Paged output via `PAGER`.
//!
//! clig.dev: "Use a pager (e.g. `less`) if you are outputting a lot of text...
//! Use a pager only if `stdin` or `stdout` is an interactive terminal."
//!
//! On a non-TTY (pipe/CI), it does not launch a pager and prints the text as is.

use crate::error::Result;
use std::io::Write;

/// If stdout is a TTY, uses `PAGER` (default `less -FIRX`); otherwise prints as is.
///
/// If launching the pager fails, silently falls back to plain output.
pub fn page(text: &str) -> Result<()> {
    if !super::stdout_is_terminal() {
        print!("{}", text);
        return Ok(());
    }
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less -FIRX".to_owned());
    if !run_pager(&pager, text) {
        print!("{}", text);
    }
    Ok(())
}

/// Spawns a pager process and feeds it the text. Returns `true` if it exits successfully.
///
/// Runs via `sh -c`, so values containing arguments like `PAGER="less -R"` also work.
fn run_pager(pager: &str, text: &str) -> bool {
    let spawned = std::process::Command::new("sh")
        .arg("-c")
        .arg(pager)
        .stdin(std::process::Stdio::piped())
        .spawn();
    let Ok(mut child) = spawned else {
        return false;
    };
    if let Some(stdin) = child.stdin.as_mut()
        && stdin.write_all(text.as_bytes()).is_err()
    {
        let _ = child.wait();
        return false;
    }
    drop(child.stdin.take());
    child.wait().map(|s| s.success()).unwrap_or(false)
}
