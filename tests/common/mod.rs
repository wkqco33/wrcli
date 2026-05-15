#![allow(dead_code)]

pub fn args(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        vec![]
    } else {
        s.split_whitespace().map(str::to_owned).collect()
    }
}

/// 임시 디렉토리 — Drop 시 자동 삭제.
pub struct TempDir(std::path::PathBuf);

impl TempDir {
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn tempdir() -> TempDir {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let path = std::env::temp_dir().join(format!("wrcli_test_{}", ts));
    std::fs::create_dir_all(&path).unwrap();
    TempDir(path)
}
