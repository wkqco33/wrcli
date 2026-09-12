#![allow(dead_code)]

use std::sync::Mutex;

/// Serializes access to env variables with a global mutex (safe for parallel test execution).
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard: sets an env variable on creation and restores the original value on Drop.
pub struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    entries: Vec<(String, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    /// Returns a Guard that sets an env variable and restores it automatically on Drop.
    ///
    /// # Example
    /// ```ignore
    /// let _g = EnvGuard::set("MYAPP_PORT", "3000");
    /// // ... test code ...
    /// // the env variable is removed automatically when _g drops
    /// ```
    pub fn set(key: &str, val: &str) -> Self {
        Self::set_many(&[(key, val)])
    }

    /// Returns a Guard that sets multiple env variables under a single lock and restores them
    /// automatically on Drop.
    ///
    /// Tests that depend on env variables must set them together in a single Guard,
    /// even when they use different variables, to be parallel-safe without deadlocks.
    pub fn set_many(entries: &[(&str, &str)]) -> Self {
        let lock = ENV_LOCK.lock().unwrap();
        let mut restored = Vec::with_capacity(entries.len());
        for (key, val) in entries {
            let prev = std::env::var_os(key);
            // SAFETY: serialized by ENV_LOCK, so there is no concurrent access
            unsafe {
                std::env::set_var(key, val);
            }
            restored.push((key.to_string(), prev));
        }
        EnvGuard {
            _lock: lock,
            entries: restored,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: serialized by ENV_LOCK, so there is no concurrent access
        for (key, prev) in &self.entries {
            match prev {
                Some(v) => unsafe {
                    std::env::set_var(key, v);
                },
                None => unsafe {
                    std::env::remove_var(key);
                },
            }
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

/// Temporary directory — automatically deleted on Drop.
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
