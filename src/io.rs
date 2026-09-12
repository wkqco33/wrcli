//! I/O helpers that support the `-` (stdin/stdout) convention.
//!
//! clig.dev: "If input or output is a file, support `-` to read from `stdin`
//! or write to `stdout`."
//!
//! ```no_run
//! use wrcli::io::{open_reader, open_writer};
//! use std::io::{Read, Write};
//!
//! # fn main() -> wrcli::Result<()> {
//! let mut input = open_reader("-")?; // stdin
//! let mut buf = String::new();
//! input.read_to_string(&mut buf)?;
//!
//! let mut output = open_writer("-")?; // stdout
//! output.write_all(buf.as_bytes())?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use std::io::{Read, Write};

/// Returns a reader that reads from stdin if `path` is `-`, otherwise opens the given file.
pub fn open_reader(path: &str) -> Result<Box<dyn Read>> {
    if path == "-" {
        Ok(Box::new(std::io::stdin()))
    } else {
        Ok(Box::new(std::fs::File::open(path)?))
    }
}

/// Returns a writer that writes to stdout if `path` is `-`, otherwise creates/truncates the given file.
pub fn open_writer(path: &str) -> Result<Box<dyn Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout()))
    } else {
        Ok(Box::new(std::fs::File::create(path)?))
    }
}

/// Reads the entire contents as a UTF-8 string from stdin if `path` is `-`, otherwise from the file.
pub fn read_to_string(path: &str) -> Result<String> {
    let mut buf = String::new();
    open_reader(path)?.read_to_string(&mut buf)?;
    Ok(buf)
}
