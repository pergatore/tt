use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};

use crate::config::Config;
use crate::report::{self, ReportOptions, ReportRange};
use crate::storage;
use crate::util;

/// Execute the report command
pub fn execute(
    cli: &crate::Cli,
    config: &Config,
    date: Option<&str>,
    current_activity: &str,
    no_current_activity: bool,
    from_date: Option<&str>,
    to_date: Option<&str>,
    project: Option<&str>,
    _per_day: bool, // Unused parameter, renamed with underscore
    csv_section: Option<&str>,
    month: Option<&str>,
    week: Option<&str>,
    details: bool,
    comments: bool,
) -> Result<()> {
    // Determine current time
    let now = util::parse_now_arg(cli.now.as_deref())?;
    
    // Parse report date range
    let range = parse_date_range(date, from_date, to_date, month, week, &now)?;
    
    // Get data file path
    let data_file = cli.data.as_ref().map(std::path::PathBuf::from).unwrap_or_else(|| config.data_file.clone());
    
    // Read all entries from the log file
    let all_entries = storage::read_entries(&data_file)?;
    
    // Check if we're displaying just today's report (the default case)
    let is_today_only = range.start_date == now.date_naive() && 
                         range.end_date == now.date_naive() && 
                         date.is_none() && from_date.is_none() && 
                         to_date.is_none() && month.is_none() && 
                         week.is_none();
    
    // Filter entries by date range considering midnight separators
    let filtered_entries = storage::filter_entries_by_date_range(
        &all_entries, 
        range.start_date, 
        range.end_date
    );
    
    // Convert entries to activities
    let mut activities = storage::entries_to_activities(&filtered_entries, Some(range.start_date), Some(range.end_date), Some(now));    
    
    // For today-only reports, ensure we show all activities that have a start or end time today
    if is_today_only {
        activities.retain(|activity| {
            let today = now.date_naive();
            let activity_date = activity.end.date_naive();
            
            // For current day reports, show activities that end today
            activity_date == today
        });
    } else {
        // For date range reports, show all activities that occur within the range
        activities.retain(|activity| {
            let activity_date = activity.end.date_naive();
            
            // Include activities where the end date falls within the range
            activity_date >= range.start_date && activity_date <= range.end_date
        });
    }
    
    // Add current activity if requested
    if !no_current_activity && !filtered_entries.is_empty() {
        let last_entry = filtered_entries.last().unwrap();
        
        // Only add current activity if the last entry is from today
        if last_entry.datetime.date_naive() == now.date_naive() {
            let current_activity_name = if current_activity.is_empty() {
                "-- Current Activity --"
            } else {
                current_activity
            };
            
            let current = storage::create_current_activity(
                last_entry,
                now,
                current_activity_name
            );
            
            activities.push(current);
        }
    }
    
    // Create report options
    let options = ReportOptions {
        range,
        project_filter: project.map(|s| s.to_string()),
        csv_section: csv_section.map(|s| s.to_string()),
        show_details: details,
        show_comments: comments,
    };
    
    // Generate the report
    let report = if csv_section.is_some() {
        report::generate_csv_report(&activities, &options)?
    } else {
        report::generate_report(&activities, &options)?
    };
    
    // Print the report
    println!("{}", report);
    
    Ok(())
}

/// Parse date range from various command line arguments
fn parse_date_range(
    date: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    month: Option<&str>,
    week: Option<&str>,
    now: &DateTime<Local>,
) -> Result<ReportRange> {
    let today = now.date_naive();
    
    // First, determine the initial range based on date, month, or week
    let (mut start_date, mut end_date) = if let Some(month_str) = month {
        // Month range
        util::parse_month(month_str, today)?
    } else if let Some(week_str) = week {
        // Week range
        util::parse_week(week_str, today)?
    } else if let Some(date_str) = date {
        // Single day
        let report_date = util::parse_date_string(date_str, now, true)?;
        (report_date, report_date)
    } else {
        // Default to today
        (today, today)
    };
    
    // Override start date if specified
    if let Some(from_str) = from_date {
        start_date = util::parse_date_string(from_str, now, true)?;
    }
    
    // Override end date if specified
    if let Some(to_str) = to_date {
        end_date = util::parse_date_string(to_str, now, false)?;
    }
    
    // Make sure start date is not after end date
    if start_date > end_date {
        return Err(anyhow!("Start date cannot be after end date"));
    }
    
    Ok(ReportRange {
        start_date,
        end_date,
    })
}
