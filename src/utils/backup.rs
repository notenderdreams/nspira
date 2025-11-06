use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub created_at: String,
    pub project_id: i32,
    pub project_name: String,
    pub cache_directories: Vec<String>,
    pub total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: String,
    pub created_at: String,
    pub backups: Vec<BackupMetadata>,
}

pub struct BackupManager;

impl BackupManager {
    /// Get the backup directory path
    pub fn backup_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        let backup_dir = data_dir.join("nspira").join("backups");
        
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .context("Failed to create backup directory")?;
        }
        
        Ok(backup_dir)
    }

    /// Create a backup before cleaning
    pub fn create_backup(project_id: i32, project_name: &str, cache_dirs: &[String]) -> Result<PathBuf> {
        let backup_dir = Self::backup_dir()?;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("{}_{}_backup", project_id, timestamp);
        let project_backup_dir = backup_dir.join(&backup_name);

        fs::create_dir_all(&project_backup_dir)
            .context("Failed to create project backup directory")?;

        let mut total_size = 0u64;
        let mut backed_up_dirs = Vec::new();

        for (idx, cache_dir) in cache_dirs.iter().enumerate() {
            let cache_path = Path::new(cache_dir);
            
            if cache_path.exists() && cache_path.is_dir() {
                let backup_cache_dir = project_backup_dir.join(format!("cache_{}", idx));
                
                // Copy directory contents
                match copy_dir_recursive(cache_path, &backup_cache_dir) {
                    Ok(size) => {
                        total_size += size;
                        backed_up_dirs.push(cache_dir.clone());
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to backup {}: {}", cache_dir, e);
                    }
                }
            }
        }

        // Create metadata file
        let metadata = BackupMetadata {
            created_at: Utc::now().to_rfc3339(),
            project_id,
            project_name: project_name.to_string(),
            cache_directories: backed_up_dirs,
            total_size,
        };

        let metadata_path = project_backup_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .context("Failed to serialize backup metadata")?;
        
        fs::write(&metadata_path, metadata_json)
            .context("Failed to write backup metadata")?;

        // Update manifest
        Self::update_manifest(&backup_name, &metadata)?;

        Ok(project_backup_dir)
    }

    /// List available backups
    pub fn list_backups() -> Result<Vec<BackupMetadata>> {
        let manifest = Self::load_manifest()?;
        Ok(manifest.backups)
    }

    /// Restore a backup
    pub fn restore_backup(backup_name: &str) -> Result<()> {
        let backup_dir = Self::backup_dir()?;
        let backup_path = backup_dir.join(backup_name);
        
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup '{}' not found", backup_name));
        }

        // Load metadata
        let metadata_path = backup_path.join("metadata.json");
        let metadata_content = fs::read_to_string(&metadata_path)
            .context("Failed to read backup metadata")?;
        let metadata: BackupMetadata = serde_json::from_str(&metadata_content)
            .context("Failed to parse backup metadata")?;

        // Restore each cache directory
        for (idx, original_cache_dir) in metadata.cache_directories.iter().enumerate() {
            let backup_cache_dir = backup_path.join(format!("cache_{}", idx));
            let original_path = Path::new(original_cache_dir);

            if backup_cache_dir.exists() {
                // Remove existing cache directory if it exists
                if original_path.exists() {
                    fs::remove_dir_all(original_path)
                        .with_context(|| format!("Failed to remove existing cache directory: {}", original_cache_dir))?;
                }

                // Create parent directory if needed
                if let Some(parent) = original_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory for: {}", original_cache_dir))?;
                }

                // Restore from backup
                copy_dir_recursive(&backup_cache_dir, original_path)
                    .with_context(|| format!("Failed to restore cache directory: {}", original_cache_dir))?;
            }
        }

        Ok(())
    }

    /// Delete old backups (keep only the most recent N backups per project)
    pub fn cleanup_old_backups(keep_count: usize) -> Result<()> {
        let mut manifest = Self::load_manifest()?;
        
        // Group backups by project_id
        let mut project_backups: std::collections::HashMap<i32, Vec<BackupMetadata>> = 
            std::collections::HashMap::new();
        
        for backup in manifest.backups {
            project_backups.entry(backup.project_id)
                .or_insert_with(Vec::new)
                .push(backup);
        }

        let mut kept_backups = Vec::new();
        let backup_dir = Self::backup_dir()?;

        for (project_id, mut backups) in project_backups {
            // Sort by creation time (newest first)
            backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            // Keep the most recent backups
            for (idx, backup) in backups.into_iter().enumerate() {
                if idx < keep_count {
                    kept_backups.push(backup);
                } else {
                    // Delete old backup
                    let backup_name = format!("{}_{}_backup", 
                        project_id, 
                        backup.created_at.replace([':', '-'], "").replace('T', "_").split('.').next().unwrap_or("")
                    );
                    let backup_path = backup_dir.join(backup_name);
                    
                    if backup_path.exists() {
                        fs::remove_dir_all(&backup_path)
                            .with_context(|| format!("Failed to remove old backup: {}", backup_path.display()))?;
                    }
                }
            }
        }

        // Update manifest with kept backups
        manifest.backups = kept_backups;
        Self::save_manifest(&manifest)?;

        Ok(())
    }

    /// Load backup manifest
    fn load_manifest() -> Result<BackupManifest> {
        let backup_dir = Self::backup_dir()?;
        let manifest_path = backup_dir.join("manifest.json");

        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .context("Failed to read backup manifest")?;
            serde_json::from_str(&content)
                .context("Failed to parse backup manifest")
        } else {
            Ok(BackupManifest {
                version: "1.0".to_string(),
                created_at: Utc::now().to_rfc3339(),
                backups: Vec::new(),
            })
        }
    }

    /// Save backup manifest
    fn save_manifest(manifest: &BackupManifest) -> Result<()> {
        let backup_dir = Self::backup_dir()?;
        let manifest_path = backup_dir.join("manifest.json");
        
        let content = serde_json::to_string_pretty(manifest)
            .context("Failed to serialize backup manifest")?;
        
        fs::write(&manifest_path, content)
            .context("Failed to write backup manifest")?;

        Ok(())
    }

    /// Update manifest with new backup
    fn update_manifest(backup_name: &str, metadata: &BackupMetadata) -> Result<()> {
        let mut manifest = Self::load_manifest()?;
        manifest.backups.push(metadata.clone());
        Self::save_manifest(&manifest)
    }
}

/// Recursively copy a directory and return total size copied
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<u64> {
    if !src.exists() {
        return Ok(0);
    }

    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create destination directory: {}", dst.display()))?;

    let mut total_size = 0u64;

    for entry in fs::read_dir(src)
        .with_context(|| format!("Failed to read source directory: {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            total_size += copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy file: {} -> {}", src_path.display(), dst_path.display()))?;
            
            if let Ok(metadata) = fs::metadata(&src_path) {
                total_size += metadata.len();
            }
        }
    }

    Ok(total_size)
}