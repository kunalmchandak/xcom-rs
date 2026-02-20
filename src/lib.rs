pub mod auth;
pub mod billing;
pub mod bookmarks;
pub mod cli;
pub mod context;
pub mod doctor;
pub mod errors;
pub mod handlers;
pub mod introspection;
pub mod logging;
pub mod media;
pub mod output;
pub mod protocol;
pub mod search;
pub mod skills;
pub mod timeline;
pub mod tweets;
pub mod x_api;

// Test utilities for unit tests only (integration tests use tests/common/test_utils.rs)
#[cfg(test)]
pub mod test_utils {
    pub mod env_lock {
        use std::sync::Mutex;

        /// Global lock for environment variable tests
        /// This ensures that tests modifying XDG_* env vars don't interfere with each other
        pub static ENV_LOCK: Mutex<()> = Mutex::new(());
    }
}
