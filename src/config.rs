use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "tt.json";
const DATA_FILENAME: &str = "entries.log";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Path to the entries data file
    pub data_file: PathBuf,
    
    /// Whether timezone support is enabled
    pub timezone_enabled: bool,
    
    /// Default editor for editing entries
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("timetracker");
        
        Config {
            data_file: data_dir.join(DATA_FILENAME),
            timezone_enabled: false,
            editor: std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| String::from("vi")),
        }
    }
}

/// Returns the path to the config file
pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("timetracker");
    
    config_dir.join(CONFIG_FILENAME)
}

/// Load configuration from the default location
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }
    
    let config_str = fs::read_to_string(&config_path)?;
    
    let config: Config = serde_json::from_str(&config_str)?;
    
    Ok(config)
}

/// Save configuration to the default location
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path();
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Serialize and save the config
    let config_str = serde_json::to_string_pretty(config)?;
    
    fs::write(&config_path, config_str)?;
    
    Ok(())
}

/// Ensure the data directory exists
pub fn ensure_data_dir(data_file: &Path) -> Result<()> {
    if let Some(parent) = data_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    Ok(())
}
