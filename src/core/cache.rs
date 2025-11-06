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
        if paths.len() > 1 {
            // Use parallel cleaning for multiple directories
            crate::utils::parallel::clean_caches_parallel(paths)
        } else if let Some(path) = paths.first() {
            // Single directory - use regular cleaning
            let size = Self::get_size(path);
            Self::clean(path)?;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    /// Calculate sizes for multiple directories in parallel
    pub fn get_sizes_parallel(paths: &[String]) -> Vec<(String, u64)> {
        if paths.len() > 1 {
            crate::utils::parallel::calculate_sizes_parallel(paths)
        } else if let Some(path) = paths.first() {
            vec![(path.clone(), Self::get_size(path))]
        } else {
            Vec::new()
        }
    }
}
