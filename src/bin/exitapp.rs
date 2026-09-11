//! Minimal binary used by `tests/binary.rs` to verify `Command::execute_or_exit` exit codes.
//!
//! ```text
//! exitapp ok     -> prints "ok", exits 0
//! exitapp fail   -> user error, exits 1
//! exitapp <bad>  -> usage error, exits 2
//! ```

use wrcli::{Command, WrCliError};

fn main() {
    Command::new("exitapp")
        .short("Exit-code fixture for wrcli assert_cmd tests")
        .subcommand(
            Command::new("ok")
                .short("Succeed")
                .on_run(|_| println!("ok")),
        )
        .subcommand(
            Command::new("fail")
                .short("Fail with a user error")
                .on_run_e(|_| Err(WrCliError::user(std::io::Error::other("boom")))),
        )
        .execute_or_exit();
}
