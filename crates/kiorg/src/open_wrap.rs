#[cfg(not(any(test, feature = "testing")))]
pub use open::{that as open_that, with as open_with};

#[cfg(any(test, feature = "testing"))]
pub mod mock_open {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static OPEN_WITH_CALLS: OnceLock<Mutex<Vec<OpenCall>>> = OnceLock::new();
    static OPEN_THAT_CALLS: OnceLock<Mutex<Vec<OpenCall>>> = OnceLock::new();
    static TEST_SERIALIZATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn get_open_with_calls_storage() -> &'static Mutex<Vec<OpenCall>> {
        OPEN_WITH_CALLS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn get_open_that_calls_storage() -> &'static Mutex<Vec<OpenCall>> {
        OPEN_THAT_CALLS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn get_test_serialization_lock() -> &'static Mutex<()> {
        TEST_SERIALIZATION_LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Acquires the test serialization lock to ensure tests don't interfere with each other.
    /// Returns a guard that holds the lock until dropped.
    pub fn acquire_open_test_lock() -> MutexGuard<'static, ()> {
        get_test_serialization_lock().lock().unwrap()
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OpenCall {
        pub path: std::ffi::OsString,
        pub app: Option<String>,
    }

    pub fn get_open_with_calls() -> Vec<OpenCall> {
        let calls = get_open_with_calls_storage().lock().unwrap();
        calls.clone()
    }

    pub fn get_open_that_calls() -> Vec<OpenCall> {
        let calls = get_open_that_calls_storage().lock().unwrap();
        calls.clone()
    }

    pub fn clear_open_calls() {
        let mut with_calls = get_open_with_calls_storage().lock().unwrap();
        with_calls.clear();

        let mut that_calls = get_open_that_calls_storage().lock().unwrap();
        that_calls.clear();
    }

    pub fn open_with(
        path: impl AsRef<std::ffi::OsStr>,
        app: impl Into<String>,
    ) -> std::io::Result<()> {
        let mut calls = get_open_with_calls_storage().lock().unwrap();
        calls.push(OpenCall {
            path: path.as_ref().to_owned(),
            app: Some(app.into()),
        });

        Ok(())
    }

    pub fn open_that(path: impl AsRef<std::ffi::OsStr>) -> std::io::Result<()> {
        let mut calls = get_open_that_calls_storage().lock().unwrap();
        calls.push(OpenCall {
            path: path.as_ref().to_owned(),
            app: None,
        });

        Ok(())
    }
}

#[cfg(any(test, feature = "testing"))]
pub use mock_open::{
    acquire_open_test_lock, clear_open_calls, get_open_that_calls, get_open_with_calls, open_that,
    open_with,
};
