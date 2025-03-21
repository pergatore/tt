use anyhow::Result;
use chrono::{Local, NaiveTime, TimeZone};

use crate::config::Config;
use crate::entry::{Entry, HELLO_ENTRY_NAME, MIDNIGHT_SEPARATOR_PREFIX};
use crate::storage;
use crate::util;

/// Execute the hello command
pub fn execute(cli: &crate::Cli, config: &Config) -> Result<()> {
    // Determine current time, either from command line or system
    let now = util::parse_now_arg(cli.now.as_deref())?;
    
    // Get data file path
    let data_file = cli.data.as_ref().map(std::path::PathBuf::from).unwrap_or_else(|| config.data_file.clone());
    
    // Create directory structure if needed
    crate::config::ensure_data_dir(&data_file)?;
    
    // Check if there are existing entries
    let existing_entries = storage::read_entries(&data_file)?;
    
    // Check if we need to create a midnight separator
    let create_midnight_separator = if !existing_entries.is_empty() {
        let last_entry = existing_entries.last().unwrap();
        let last_entry_date = last_entry.datetime.date_naive();
        let current_date = now.date_naive();
        
        // If the last entry is from a previous day, we should add a midnight separator
        last_entry_date < current_date
    } else {
        false
    };
    
    // If needed, add a midnight separator entry
    if create_midnight_separator {
        // Create a midnight entry at 00:00 of the current day
        let midnight_naive = now.date_naive().and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let midnight = Local.from_local_datetime(&midnight_naive).single().unwrap();
        
        // Create a midnight separator entry
        let midnight_entry = Entry::new(
            midnight, 
            format!("{}", MIDNIGHT_SEPARATOR_PREFIX), 
            false, 
            None
        );
        
        // Write the midnight separator to the log file
        storage::append_entry(&data_file, &midnight_entry)?;
    }
    
    // Create a hello entry
    let entry = Entry::new(now, HELLO_ENTRY_NAME.to_string(), false, None);
    
    // Write the entry to the log file
    storage::append_entry(&data_file, &entry)?;
    
    Ok(())
}
