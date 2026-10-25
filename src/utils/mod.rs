//! Utility functions and helpers

use crate::error::Result;
use std::path::Path;

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
        std::fs::create_dir_all(path).map_err(|e| crate::error::Error::Filesystem {
            operation: "create_directory".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
    }
    Ok(())
}

/// Get a temporary directory
pub fn temp_dir() -> Result<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| crate::error::Error::Filesystem {
        operation: "create_temp_directory".to_string(),
        path: std::env::temp_dir(),
        reason: e.to_string(),
    })
}

/// Get a random string
pub fn random_string(len: usize) -> String {
    use rand::distr::Alphanumeric;
    use rand::{RngExt, rng};

    rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub mod logging;
pub mod manifest;
