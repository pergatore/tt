use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::entry::{Activity, Entry};

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

pub fn entries_to_activities(entries: &[Entry]) -> Vec<Activity> {
    let mut activities = Vec::new();
    
    // We need at least two entries to create an activity
    if entries.len() < 2 {
        return activities;
    }
    
    // Create activities from consecutive entries
    for i in 0..entries.len() - 1 {
        let activity = Activity::new(
            entries[i+1].name.clone(),
            entries[i].datetime,
            entries[i+1].datetime,
            false,
            entries[i+1].comment.clone(),
        );
        
        activities.push(activity);
    }
    
    activities
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
