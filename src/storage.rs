use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::entry::{Activity, Entry, MIDNIGHT_SEPARATOR_PREFIX};

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

pub fn entries_to_activities(entries: &[Entry], start_date: Option<NaiveDate>, end_date: Option<NaiveDate>) -> Vec<Activity> {
    let mut activities = Vec::new();
    
    // We need at least two entries to create an activity
    if entries.len() < 2 {
        return activities;
    }
    
    // Create activities from consecutive entries, skipping midnight separators
    for i in 0..entries.len() - 1 {
        // Skip if this is a midnight separator entry
        if entries[i+1].name.starts_with(MIDNIGHT_SEPARATOR_PREFIX) {
            continue;
        }
        
        // Skip if previous entry was a midnight separator
        if entries[i].name.starts_with(MIDNIGHT_SEPARATOR_PREFIX) {
            continue;
        }
        
        // Get the start and end times
        let start_time = entries[i].datetime;
        let end_time = entries[i+1].datetime;
        
        // Apply date filtering if specified
        // Only include activities that occur within the date range
        if let Some(start_date) = start_date {
            if end_time.date_naive() < start_date {
                continue;  // Skip activities that end before the start date
            }
        }
        
        if let Some(end_date) = end_date {
            if start_time.date_naive() > end_date {
                continue;  // Skip activities that start after the end date
            }
        }
        
        // Only include activities where the end time is within the date range
        // This ensures only activities that complete within the requested range are shown
        if let (Some(start_date), Some(end_date)) = (start_date, end_date) {
            let activity_date = end_time.date_naive();
            if activity_date < start_date || activity_date > end_date {
                continue;
            }
        }
        
        let activity = Activity::new(
            entries[i+1].name.clone(),
            start_time,
            end_time,
            false,
            entries[i+1].comment.clone(),
        );
        
        activities.push(activity);
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
