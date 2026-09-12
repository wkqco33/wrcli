//! Ctrl-C(`SIGINT`) 처리 (`signal` 피처).
//!
//! clig.dev: "If a user hits Ctrl-C (the INT signal), exit as soon as possible.
//! Say something immediately, before you start clean-up."
//!
//! 핸들러 안에서는 async-signal-safe 연산(`write`, `_exit`)만 수행하므로
//! 정리(clean-up) 코드를 실행하지 않는다. 정리가 필요하면 호출자가 별도로
//! 처리해야 한다 (crash-only 설계).
//!
//! ```no_run
//! # #[cfg(feature = "signal")] {
//! wrcli::signal::install("^C interrupted\n");
//! # }
//! ```

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static MESSAGE: OnceLock<&'static str> = OnceLock::new();
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Ctrl-C 수신 시 출력할 메시지를 설정한다.
pub fn set_message(message: &'static str) {
    let _ = MESSAGE.set(message);
}

/// `SIGINT` 핸들러를 설치한다. 반복 호출은 무시된다.
///
/// 핸들러는 메시지를 stderr에 `write`한 뒤 종료 코드 `130`(128 + SIGINT)으로
/// 즉시 프로세스를 끝낸다.
pub fn install(message: &'static str) {
    set_message(message);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    #[cfg(unix)]
    unsafe {
        // SAFETY: async-signal-safe 핸들러를 등록한다.
        libc::signal(
            libc::SIGINT,
            handle_sigint as *const () as libc::sighandler_t,
        );
    }
}

#[cfg(unix)]
unsafe extern "C" fn handle_sigint(_sig: libc::c_int) {
    let msg = MESSAGE.get().copied().unwrap_or("Interrupted.\n");
    // SAFETY: write와 _exit만 사용하는 async-signal-safe 경로.
    unsafe {
        libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
        libc::_exit(130);
    }
}
