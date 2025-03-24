use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::entry::{Activity, Entry, HELLO_ENTRY_NAME, MIDNIGHT_SEPARATOR_PREFIX};

pub fn read_entries(data_file: &Path) -> Result<Vec<Entry>> {
    if !data_file.exists() {
        return Ok(Vec::new());
    }
    
    let file = File::open(data_file)
        .context(format!("Failed to open data file: {:?}", data_file))?;
    
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.context(format!("Failed to read line {}", line_num + 1))?;
        
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }
        
        match Entry::parse(&line) {
            Ok(entry) => {
                entries.push(entry);
            },
            Err(err) => {
                return Err(err).context(format!("Error parsing line {}: {}", line_num + 1, line));
            }
        }
    }
    
    // Ensure entries are sorted chronologically
    entries.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    
    // Validate chronological order
    for i in 1..entries.len() {
        if entries[i].datetime < entries[i-1].datetime {
            return Err(anyhow::anyhow!(
                "Entries are not in chronological order at position {}: {} is before {}",
                i + 1, entries[i], entries[i-1]
            ));
        }
    }
    
    Ok(entries)
}

pub fn append_entry(data_file: &Path, entry: &Entry) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = data_file.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create directory {:?}", parent))?;
    }
    
    // Determine if we need to add a separator line
    let add_separator = if data_file.exists() {
        let entries = read_entries(data_file)?;
        !entries.is_empty() && entries.last().unwrap().datetime.date_naive() != entry.datetime.date_naive()
    } else {
        false
    };
    
    // Check if we need to start with a newline
    let file_ends_with_newline = if data_file.exists() {
        let metadata = fs::metadata(data_file)?;
        if metadata.len() > 0 {
            let mut file = File::open(data_file)?;
            let file_len = file.metadata()?.len();
            let mut buf = [0u8; 1];
            
            if file_len > 0 {
                file.seek(SeekFrom::End(-1))?;
                file.read_exact(&mut buf)?;
                buf[0] == b'\n'
            } else {
                true // Empty file technically ends with a newline
            }
        } else {
            true // Empty file
        }
    } else {
        true // File doesn't exist yet
    };
    
    // Open file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_file)
        .context(format!("Failed to open data file for writing: {:?}", data_file))?;
    
    // Write the entry
    if !file_ends_with_newline {
        writeln!(file)?;
    }
    
    if add_separator {
        writeln!(file)?;
    }
    
    writeln!(file, "{}", entry)
        .context("Failed to write entry to file")?;
    
    Ok(())
}


pub fn entries_to_activities(
    entries: &[Entry], 
    start_date: Option<NaiveDate>, 
    end_date: Option<NaiveDate>,
    now: Option<DateTime<Local>> // Add parameter for current time
) -> Vec<Activity> {
    let mut activities = Vec::new();
    
    // We need at least one entry to create an activity
    if entries.is_empty() {
        return activities;
    }
    
    // Create activities from consecutive entries, skipping midnight separators and hello entries
    for i in 0..entries.len() - 1 {
        // Skip current entry if it's a hello entry - hello doesn't create duration
        if entries[i].name == HELLO_ENTRY_NAME {
            continue;
        }
        
        // Skip current entry if it's a midnight separator
        if entries[i].name.starts_with(MIDNIGHT_SEPARATOR_PREFIX) {
            continue;
        }
        
        // Get the start and end times
        let start_time = entries[i].datetime;
        let end_time = entries[i+1].datetime;
        
        // Create activity using the CURRENT entry's name
        // This represents what you were doing during this time span
        let activity = Activity::new(
            entries[i].name.clone(),
            start_time,
            end_time,
            false,
            entries[i].comment.clone(),
        );
        
        activities.push(activity);
    }
    
    // Handle the last entry separately - it represents what you're currently doing
    // Only add if we have the current time and it's the same day
    if let Some(now_time) = now {
        if let Some(last_entry) = entries.last() {
            // Skip if it's a hello entry or midnight separator
            if last_entry.name != HELLO_ENTRY_NAME && 
               !last_entry.name.starts_with(MIDNIGHT_SEPARATOR_PREFIX) {
                
                // Only add if it's from today
                if last_entry.datetime.date_naive() == now_time.date_naive() {
                    let activity = Activity::new(
                        last_entry.name.clone(),
                        last_entry.datetime,
                        now_time,
                        true, // Mark as current activity
                        last_entry.comment.clone(),
                    );
                    
                    activities.push(activity);
                }
            }
        }
    }
    
    // Apply date filtering if specified
    if let (Some(start), Some(end)) = (start_date, end_date) {
        activities.retain(|activity| {
            let activity_date = activity.end.date_naive();
            activity_date >= start && activity_date <= end
        });
    }
    
    activities
}

pub fn filter_entries_by_date_range(entries: &[Entry], start_date: NaiveDate, end_date: NaiveDate) -> Vec<Entry> {
    // If there are no entries, return an empty vector
    if entries.is_empty() {
        return Vec::new();
    }
    
    let mut filtered_entries = Vec::new();
    
    // Find the last entry before the start date (needed for calculating the first activity's duration)
    // This handles the case where an activity starts before our date range but ends within it
    let mut last_entry_before_range = None;
    for entry in entries.iter().rev() {
        if entry.datetime.date_naive() < start_date {
            last_entry_before_range = Some(entry.clone());
            break;
        }
    }
    
    // If we found a last entry before the range, include it
    if let Some(entry) = last_entry_before_range {
        filtered_entries.push(entry);
    }
    
    // Include all entries within the date range
    for entry in entries {
        let entry_date = entry.datetime.date_naive();
        
        if entry_date >= start_date && entry_date <= end_date {
            filtered_entries.push(entry.clone());
        }
    }
    
    // Sort entries by datetime (just in case)
    filtered_entries.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    
    filtered_entries
}

pub fn create_current_activity(
    last_entry: &Entry,
    now: DateTime<Local>,
    current_activity_name: &str,
) -> Activity {
    Activity::new(
        current_activity_name.to_string(),
        last_entry.datetime,
        now,
        true,
        None,
    )
}
