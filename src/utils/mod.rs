//! Utility functions and helpers

use std::path::Path;
use crate::error::Result;

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Check if a directory exists
pub fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Create a directory if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| crate::error::Error::FileSystem(format!("Failed to create directory: {}", e)))?;
    }
    Ok(())
}

/// Get a temporary directory
pub fn temp_dir() -> Result<tempfile::TempDir> {
    tempfile::tempdir()
        .map_err(|e| crate::error::Error::FileSystem(format!("Failed to create temporary directory: {}", e)))
}

/// Get a random string
pub fn random_string(len: usize) -> String {
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;
    
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub mod manifest;
pub mod logging;
