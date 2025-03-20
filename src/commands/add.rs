use anyhow::Result;

use crate::config::Config;
use crate::entry::Entry;
use crate::storage;
use crate::util;

/// Execute the add command
pub fn execute(cli: &crate::Cli, config: &Config, name: &str, comment: Option<&str>) -> Result<()> {
    // Determine current time, either from command line or system
    let now = util::parse_now_arg(cli.now.as_deref())?;
    
    // Create an entry
    let entry = Entry::new(now, name.to_string(), false, comment.map(|s| s.to_string()));
    
    // Get data file path
    let data_file = cli.data.as_ref().map(std::path::PathBuf::from).unwrap_or_else(|| config.data_file.clone());
    
    // Create directory structure if needed
    crate::config::ensure_data_dir(&data_file)?;
    
    // Write the entry to the log file
    storage::append_entry(&data_file, &entry)?;
    
    Ok(())
}
