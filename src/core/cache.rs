use crate::utils::{clean_dir, get_dir_size};
use anyhow::Result;

/// Cache operations
pub struct CacheManager;

impl CacheManager {
    /// Get the size of a directory
    pub fn get_size(path: &str) -> u64 {
        get_dir_size(path)
    }

    /// Clean a cache directory
    pub fn clean(path: &str) -> Result<()> {
        clean_dir(path)
    }

    /// Clean multiple cache directories and return total size freed
    pub fn clean_multiple(paths: &[String]) -> Result<u64> {
        let mut total_freed = 0u64;

        for path in paths {
            let size = Self::get_size(path);
            Self::clean(path)?;
            total_freed += size;
        }

        Ok(total_freed)
    }
}
