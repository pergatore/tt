use anyhow::{Context, Result};
use std::process::Command;

use crate::config::Config;

/// Execute the edit command
pub fn execute(cli: &crate::Cli, config: &Config) -> Result<()> {
    // Get data file path
    let data_file = cli.data.as_ref().map(std::path::PathBuf::from).unwrap_or_else(|| config.data_file.clone());
    
    // Create directory structure if needed
    crate::config::ensure_data_dir(&data_file)?;
    
    // Get editor from config
    let editor = &config.editor;
    
    // Launch the editor with the data file
    let status = Command::new(editor)
        .arg(&data_file)
        .status()
        .context(format!("Failed to run editor: {}", editor))?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Editor process exited with error: {}", status));
    }
    
    Ok(())
}
