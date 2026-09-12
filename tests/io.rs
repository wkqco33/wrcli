//! clig.dev `-`(stdin/stdout) 관례 헬퍼 테스트.

mod common;

use common::tempdir;
use std::io::{Read, Write};
use wrcli::io::{open_reader, open_writer, read_to_string};

#[test]
fn writer_and_reader_roundtrip_file() {
    let dir = tempdir();
    let path = dir.path().join("out.txt");
    let path_str = path.to_str().unwrap().to_owned();

    let mut w = open_writer(&path_str).unwrap();
    w.write_all(b"hello\nworld\n").unwrap();
    w.flush().unwrap();
    drop(w);

    let mut r = open_reader(&path_str).unwrap();
    let mut buf = String::new();
    r.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "hello\nworld\n");

    assert_eq!(read_to_string(&path_str).unwrap(), "hello\nworld\n");
}

#[test]
fn dash_reader_uses_stdin() {
    assert!(open_reader("-").is_ok());
}

#[test]
fn dash_writer_uses_stdout() {
    assert!(open_writer("-").is_ok());
}

#[test]
fn missing_file_is_an_io_error() {
    let err = open_reader("/definitely/not/here.txt").err().unwrap();
    assert!(matches!(err, wrcli::WrCliError::Io(_)));
}
