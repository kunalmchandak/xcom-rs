/// Test utilities for coordinating test execution
#[cfg(test)]
pub mod env_lock {
    use std::sync::Mutex;

    /// Global lock for environment variable tests
    /// This ensures that tests modifying XDG_* env vars don't interfere with each other
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}
