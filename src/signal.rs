//! Ctrl-C (`SIGINT`) handling (`signal` feature).
//!
//! clig.dev: "If a user hits Ctrl-C (the INT signal), exit as soon as possible.
//! Say something immediately, before you start clean-up."
//!
//! The handler performs only async-signal-safe operations (`write`, `_exit`), so it
//! does not run any clean-up code. If clean-up is needed, the caller must handle it
//! separately (crash-only design).
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

/// Sets the message to print when Ctrl-C is received.
pub fn set_message(message: &'static str) {
    let _ = MESSAGE.set(message);
}

/// Installs the `SIGINT` handler. Repeated calls are ignored.
///
/// The handler `write`s the message to stderr, then exits the process immediately
/// with exit code `130` (128 + SIGINT).
pub fn install(message: &'static str) {
    set_message(message);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    #[cfg(unix)]
    unsafe {
        // SAFETY: registers an async-signal-safe handler.
        libc::signal(
            libc::SIGINT,
            handle_sigint as *const () as libc::sighandler_t,
        );
    }
}

#[cfg(unix)]
unsafe extern "C" fn handle_sigint(_sig: libc::c_int) {
    let msg = MESSAGE.get().copied().unwrap_or("Interrupted.\n");
    // SAFETY: an async-signal-safe path using only write and _exit.
    unsafe {
        libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
        libc::_exit(130);
    }
}
