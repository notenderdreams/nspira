use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default)]
    pub clean: CleanConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Maximum depth for filesystem scanning
    #[serde(default = "default_scan_depth")]
    pub max_depth: usize,
    
    /// Directories to skip during scanning
    #[serde(default = "default_skip_dirs")]
    pub skip_directories: Vec<String>,
    
    /// Number of parallel threads for scanning
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanConfig {
    /// Ask for confirmation before cleaning
    #[serde(default = "default_true")]
    pub confirm_before_clean: bool,
    
    /// Track cleaning history in database
    #[serde(default = "default_true")]
    pub enable_history: bool,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_scan_depth() -> usize {
    4
}

fn default_skip_dirs() -> Vec<String> {
    vec![
        "Library".to_string(),
        "System".to_string(),
        "Applications".to_string(),
        ".Trash".to_string(),
        "node_modules".to_string(),
    ]
}

fn default_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: default_scan_depth(),
            skip_directories: default_skip_dirs(),
            parallelism: default_parallelism(),
        }
    }
}

impl Default for CleanConfig {
    fn default() -> Self {
        Self {
            confirm_before_clean: true,
            enable_history: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            clean: CleanConfig::default(),
        }
    }
}

impl Config {
    /// Get the configuration file path
    pub fn path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        let nspira_config = config_dir.join("nspira");
        if !nspira_config.exists() {
            fs::create_dir_all(&nspira_config)?;
        }
        Ok(nspira_config.join("config.toml"))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::path()?;
        
        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset() -> Result<()> {
        let config = Self::default();
        config.save()?;
        Ok(())
    }
}
