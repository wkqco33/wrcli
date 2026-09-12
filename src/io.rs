//! `-`(stdin/stdout) 관례를 지원하는 입출력 헬퍼.
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

/// `path`가 `-`이면 stdin, 아니면 해당 파일을 여는 reader를 반환한다.
pub fn open_reader(path: &str) -> Result<Box<dyn Read>> {
    if path == "-" {
        Ok(Box::new(std::io::stdin()))
    } else {
        Ok(Box::new(std::fs::File::open(path)?))
    }
}

/// `path`가 `-`이면 stdout, 아니면 해당 파일을 생성/절단하는 writer를 반환한다.
pub fn open_writer(path: &str) -> Result<Box<dyn Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout()))
    } else {
        Ok(Box::new(std::fs::File::create(path)?))
    }
}

/// `-`면 stdin에서, 아니면 파일에서 전체 내용을 UTF-8 문자열로 읽는다.
pub fn read_to_string(path: &str) -> Result<String> {
    let mut buf = String::new();
    open_reader(path)?.read_to_string(&mut buf)?;
    Ok(buf)
}
