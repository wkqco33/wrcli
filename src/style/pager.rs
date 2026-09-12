//! `PAGER`를 통한 페이지 출력.
//!
//! clig.dev: "Use a pager (e.g. `less`) if you are outputting a lot of text...
//! Use a pager only if `stdin` or `stdout` is an interactive terminal."
//!
//! 비TTY(파이프·CI)에서는 페이저를 띄우지 않고 텍스트를 그대로 출력한다.

use crate::error::Result;
use std::io::Write;

/// stdout이 TTY면 `PAGER`(기본 `less -FIRX`)로, 아니면 그대로 출력한다.
///
/// 페이저 실행에 실패하면 조용히 일반 출력으로 폴백한다.
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

/// 페이저 프로세스를 띄우고 텍스트를 넘긴다. 성공적으로 끝나면 `true`.
///
/// `sh -c`로 실행하므로 `PAGER="less -R"`처럼 인자를 포함해도 동작한다.
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
