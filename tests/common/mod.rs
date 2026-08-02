#![allow(dead_code)]

use std::sync::Mutex;

/// 전역 뮤텍스로 env 변수 접근을 직렬화 (테스트 병렬 실행 안전).
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard: 생성 시 env 변수를 설정하고, Drop 시 원래 값으로 복원.
pub struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    key: String,
    prev: Option<std::ffi::OsString>,
}

impl EnvGuard {
    /// env 변수를 설정하고 Drop 시 자동 복원되는 Guard 반환.
    ///
    /// # Example
    /// ```ignore
    /// let _g = EnvGuard::set("MYAPP_PORT", "3000");
    /// // ... test code ...
    /// // _g drop 시 env 변수 자동 제거
    /// ```
    pub fn set(key: &str, val: &str) -> Self {
        let lock = ENV_LOCK.lock().unwrap();
        let prev = std::env::var_os(key);
        // SAFETY: ENV_LOCK으로 직렬화되어 동시 접근 없음
        unsafe {
            std::env::set_var(key, val);
        }
        EnvGuard {
            _lock: lock,
            key: key.to_owned(),
            prev,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: ENV_LOCK으로 직렬화되어 동시 접근 없음
        match &self.prev {
            Some(v) => unsafe {
                std::env::set_var(&self.key, v);
            },
            None => unsafe {
                std::env::remove_var(&self.key);
            },
        }
    }
}

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
