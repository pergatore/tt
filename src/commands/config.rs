use anyhow::Result;

use crate::config::{self, Config};

/// Execute the config command
pub fn execute(_cli: &crate::Cli, config: &Config, default: bool, filename: bool) -> Result<()> {
    if filename {
        // Print the config file path
        println!("{}", config::get_config_path().display());
        return Ok(());
    }
    
    if default {
        // Print the default configuration
        let default_config = Config::default();
        println!("{}", serde_json::to_string_pretty(&default_config)?);
        return Ok(());
    }
    
    // Print the current configuration
    println!("{}", serde_json::to_string_pretty(config)?);
    
    Ok(())
}
