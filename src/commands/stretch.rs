use anyhow::{anyhow, Result};

use crate::config::Config;
use crate::entry::Entry;
use crate::storage;
use crate::util;

/// Execute the stretch command
pub fn execute(cli: &crate::Cli, config: &Config) -> Result<()> {
    // Determine current time, either from command line or system
    let now = util::parse_now_arg(cli.now.as_deref())?;
    
    // Get data file path
    let data_file = cli.data.as_ref().map(std::path::PathBuf::from).unwrap_or_else(|| config.data_file.clone());
    
    // Read entries from the log file
    let entries = storage::read_entries(&data_file)?;
    
    // Make sure there's at least one entry
    if entries.is_empty() {
        return Err(anyhow!("No entries found to stretch"));
    }
    
    // Get the latest entry
    let latest_entry = entries.last().unwrap();
    
    // Create a new entry with the same name but current time
    let new_entry = Entry::new(
        now,
        latest_entry.name.clone(),
        false,
        latest_entry.comment.clone(),
    );
    
    // Write the new entry to the log file
    storage::append_entry(&data_file, &new_entry)?;
    
    // Output the information
    println!("stretched {} {}", latest_entry.datetime.format("%Y-%m-%d %H:%M"), latest_entry.name);
    println!("        → {} {}", new_entry.datetime.format("%Y-%m-%d %H:%M"), new_entry.name);
    
    Ok(())
}
