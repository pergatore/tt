use anyhow::Result;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

use crate::entry::{Activity, ActivityType};
use crate::util;

pub struct ReportRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

pub struct ReportOptions {
    pub range: ReportRange,
    pub project_filter: Option<String>,
    pub csv_section: Option<String>,
    pub show_details: bool,
    pub show_comments: bool,
}

pub fn generate_report(activities: &[Activity], options: &ReportOptions) -> Result<String> {
    let mut output = String::new();
    
    // Generate title
    if options.range.start_date == options.range.end_date {
        // Single day report
        let title = util::format_date_title(options.range.start_date);
        output.push_str(&util::format_title(&title));
    } else {
        // Date range report
        let start_title = util::format_date_title(options.range.start_date);
        let end_title = util::format_date_title(options.range.end_date);
        let title = format!("{} to {}", start_title, end_title);
        output.push_str(&util::format_title(&title));
    }
    
    output.push_str("\n\n");
    
    // Filter activities by project if needed
    let filtered_activities: Vec<Activity> = activities.iter()
        .filter(|a| {
            if let Some(proj_filter) = &options.project_filter {
                if let Some(proj) = &a.project {
                    return proj == proj_filter;
                }
                return false;
            }
            true
        })
        .cloned()
        .collect();
    
    // Calculate summary
    let summary = calculate_summary(&filtered_activities);
    output.push_str(&format_summary(&summary));
    output.push_str("\n\n");
    
    // Generate projects section
    let projects = group_by_project(&filtered_activities);
    output.push_str(&util::format_title("Projects"));
    output.push_str("\n\n");
    output.push_str(&format_projects(&projects));
    output.push_str("\n\n");
    
    // Generate activities section
    let activity_groups = group_by_activity(&filtered_activities);
    output.push_str(&util::format_title("Activities"));
    output.push_str("\n\n");
    output.push_str(&format_activity_groups(&activity_groups));
    output.push_str("\n\n");
    
    // Generate details section if it's a single day report or details are explicitly requested
    if options.range.start_date == options.range.end_date || options.show_details {
        output.push_str(&util::format_title("Details"));
        output.push_str("\n\n");
        output.push_str(&format_details(&filtered_activities, options.show_comments));
    }
    
    Ok(output)
}

struct Summary {
    total_time: chrono::Duration,
    work_time: chrono::Duration,
    break_time: chrono::Duration,
    current_activity_time: Option<chrono::Duration>,
    current_activity_type: Option<ActivityType>,
}

fn calculate_summary(activities: &[Activity]) -> Summary {
    let mut work_time = chrono::Duration::zero();
    let mut break_time = chrono::Duration::zero();
    let mut current_activity_time = None;
    let mut current_activity_type = None;
    
    for activity in activities {
        match activity.activity_type {
            ActivityType::Work => {
                work_time = work_time + activity.duration;
            },
            ActivityType::Break => {
                break_time = break_time + activity.duration;
            },
            ActivityType::Ignored => {
                // Ignore these activities
            },
        }
        
        if activity.is_current_activity {
            current_activity_time = Some(activity.duration);
            current_activity_type = Some(activity.activity_type.clone());
        }
    }
    
    let total_time = work_time + break_time;
    
    Summary {
        total_time,
        work_time,
        break_time,
        current_activity_time,
        current_activity_type,
    }
}

fn format_summary(summary: &Summary) -> String {
    let mut output = String::new();
    
    // Format the total time line
    output.push_str("  Total: ");
    output.push_str(&util::format_duration(summary.total_time));
    
    if let Some(current_time) = summary.current_activity_time {
        let previous_time = summary.total_time - current_time;
        output.push_str(&format!(" ({} + {})", 
            util::format_duration(previous_time),
            util::format_duration(current_time)));
    }
    output.push('\n');
    
    // Format the working time line
    output.push_str("Working: ");
    output.push_str(&util::format_duration(summary.work_time));
    
    if let Some(current_time) = summary.current_activity_time {
        if let Some(ActivityType::Work) = &summary.current_activity_type {
            let previous_work_time = summary.work_time - current_time;
            output.push_str(&format!(" ({} + {})",
                util::format_duration(previous_work_time),
                util::format_duration(current_time)));
        }
    }
    output.push('\n');
    
    // Format the break time line
    output.push_str("  Break: ");
    output.push_str(&util::format_duration(summary.break_time));
    
    if let Some(current_time) = summary.current_activity_time {
        if let Some(ActivityType::Break) = &summary.current_activity_type {
            let previous_break_time = summary.break_time - current_time;
            output.push_str(&format!(" ({} + {})",
                util::format_duration(previous_break_time),
                util::format_duration(current_time)));
        }
    }
    
    output
}

fn group_by_project(activities: &[Activity]) -> HashMap<String, (chrono::Duration, Vec<String>)> {
    let mut projects: HashMap<String, (chrono::Duration, Vec<String>)> = HashMap::new();
    
    // Only consider work activities
    for activity in activities {
        if activity.activity_type != ActivityType::Work {
            continue;
        }
        
        let project_name = activity.project.clone().unwrap_or_default();
        
        if !projects.contains_key(&project_name) {
            projects.insert(project_name.clone(), (chrono::Duration::zero(), Vec::new()));
        }
        
        if let Some((duration, tasks)) = projects.get_mut(&project_name) {
            // Add duration
            *duration = *duration + activity.duration;
            
            // Add task name if not already present
            if !tasks.contains(&activity.task) {
                tasks.push(activity.task.clone());
            }
        }
    }
    
    projects
}

fn format_projects(projects: &HashMap<String, (chrono::Duration, Vec<String>)>) -> String {
    let mut output = String::new();
    
    // Sort projects by name
    let mut project_names: Vec<&String> = projects.keys().collect();
    project_names.sort();
    
    for project_name in project_names {
        let (duration, tasks) = &projects[project_name];
        let tasks_str = tasks.join(", ");
        
        output.push_str(&format!("({}) {}: {}\n", 
            util::format_duration(*duration),
            project_name,
            tasks_str));
    }
    
    output
}

fn group_by_activity(activities: &[Activity]) -> HashMap<ActivityType, Vec<(String, String, chrono::Duration, NaiveDate)>> {
    let mut grouped: HashMap<ActivityType, Vec<(String, String, chrono::Duration, NaiveDate)>> = HashMap::new();
    let mut seen_tasks: HashMap<(ActivityType, String, String, NaiveDate), chrono::Duration> = HashMap::new();
    
    // First, calculate total durations for each unique activity
    for activity in activities {
        if activity.activity_type == ActivityType::Ignored {
            continue;
        }
        
        let project = activity.project.clone().unwrap_or_default();
        let activity_date = activity.end.date_naive();
        let key = (activity.activity_type.clone(), project.clone(), activity.task.clone(), activity_date);
        
        let duration = seen_tasks.entry(key).or_insert(chrono::Duration::zero());
        *duration = *duration + activity.duration;
    }
    
    // Then, populate the groups
    for ((activity_type, project, task, date), duration) in seen_tasks {
        if !grouped.contains_key(&activity_type) {
            grouped.insert(activity_type.clone(), Vec::new());
        }
        
        if let Some(group) = grouped.get_mut(&activity_type) {
            group.push((project, task, duration, date));
        }
    }
    
    // Sort each group by date, then by project and task
    for group in grouped.values_mut() {
        group.sort_by(|a, b| {
            // First sort by date
            let date_cmp = a.3.cmp(&b.3);
            if date_cmp != std::cmp::Ordering::Equal {
                return date_cmp;
            }
            
            // Then sort by project and task
            let a_key = format!("{}{}", a.0.to_lowercase(), a.1.to_lowercase());
            let b_key = format!("{}{}", b.0.to_lowercase(), b.1.to_lowercase());
            a_key.cmp(&b_key)
        });
    }
    
    grouped
}

fn format_activity_groups(groups: &HashMap<ActivityType, Vec<(String, String, chrono::Duration, NaiveDate)>>) -> String {
    let mut output = String::new();
    let mut saw_multi_days = false;
    
    // Check if we have activities from multiple days
    let mut unique_dates = HashSet::new();
    for activities in groups.values() {
        for (_, _, _, date) in activities {
            unique_dates.insert(*date);
        }
    }
    
    saw_multi_days = unique_dates.len() > 1;
    
    // Format work activities
    if let Some(work_activities) = groups.get(&ActivityType::Work) {
        let mut current_date: Option<NaiveDate> = None;
        
        for (project, task, duration, date) in work_activities {
            // Add date header if date changes and we have multiple days
            if saw_multi_days && current_date.map_or(true, |d| d != *date) {
                if current_date.is_some() {
                    output.push('\n');
                }
                
                current_date = Some(*date);
                output.push_str(&format!("{}:\n", date.format("%Y-%m-%d")));
            }
            
            let project_str = if project.is_empty() {
                String::new()
            } else {
                format!("{}: ", project)
            };
            
            output.push_str(&format!("({}) {}{}\n", 
                util::format_duration(*duration),
                project_str,
                task));
        }
    }
    
    output.push('\n');
    
    // Format break activities
    if let Some(break_activities) = groups.get(&ActivityType::Break) {
        let mut current_date: Option<NaiveDate> = None;
        
        for (project, task, duration, date) in break_activities {
            // Add date header if date changes and we have multiple days
            if saw_multi_days && current_date.map_or(true, |d| d != *date) {
                if current_date.is_some() {
                    output.push('\n');
                }
                
                current_date = Some(*date);
                output.push_str(&format!("{}:\n", date.format("%Y-%m-%d")));
            }
            
            let project_str = if project.is_empty() {
                String::new()
            } else {
                format!("{}: ", project)
            };
            
            output.push_str(&format!("({}) {}{}\n", 
                util::format_duration(*duration),
                project_str,
                task));
        }
    }
    
    output
}

fn format_details(activities: &[Activity], show_comments: bool) -> String {
    let mut output = String::new();
    let mut current_date: Option<NaiveDate> = None;
    
    // Sort activities by start time
    let mut sorted_activities = activities.to_vec();
    sorted_activities.sort_by(|a, b| a.start.cmp(&b.start));
    
    // Do we need to show dates (if activities span multiple days)?
    let show_dates = activities.len() > 1 && 
        activities.first().unwrap().start.date_naive() != activities.last().unwrap().start.date_naive();
    
    for activity in sorted_activities {
        // Check if we need to print a date header
        if show_dates {
            let activity_date = activity.start.date_naive();
            
            if current_date.map_or(true, |d| d != activity_date) {
                if current_date.is_some() {
                    output.push_str("\n");
                }
                
                current_date = Some(activity_date);
                output.push_str(&format!("{}:\n\n", activity_date.format("%Y-%m-%d")));
            }
        }
        
        // Format the activity
        let start_time = activity.start.format("%H:%M").to_string();
        let end_time = activity.end.format("%H:%M").to_string();
        
        let mut line = format!("({}) {}-{} {}", 
            util::format_duration(activity.duration),
            start_time,
            end_time,
            activity.name);
        
        // Add comment if requested
        if show_comments && activity.comment.is_some() {
            line.push_str(&format!(" # {}", activity.comment.as_ref().unwrap()));
        }
        
        output.push_str(&line);
        output.push('\n');
    }
    
    output
}

pub fn generate_csv_report(activities: &[Activity], options: &ReportOptions) -> Result<String> {
    let csv_section = match options.csv_section.as_deref() {
        Some("per-day") | Some("per_day") => "per_day",
        Some("per-task") | Some("per_task") => "per_task",
        _ => return Err(anyhow::anyhow!("Invalid CSV section: {:?}", options.csv_section)),
    };
    
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    
    if csv_section == "per_day" {
        // Group activities by day
        let mut days: HashMap<NaiveDate, (chrono::Duration, Vec<String>, Vec<String>)> = HashMap::new();
        
        for activity in activities {
            if activity.activity_type != ActivityType::Work {
                continue;
            }
            
            let date = activity.start.date_naive();
            
            if !days.contains_key(&date) {
                days.insert(date, (chrono::Duration::zero(), Vec::new(), Vec::new()));
            }
            
            if let Some((duration, projects, tasks)) = days.get_mut(&date) {
                // Add duration
                *duration = *duration + activity.duration;
                
                // Add project if not already present
                if let Some(project) = &activity.project {
                    if !projects.contains(project) {
                        projects.push(project.clone());
                    }
                }
                
                // Add task if not already present
                if !tasks.contains(&activity.task) {
                    tasks.push(activity.task.clone());
                }
            }
        }
        
        // Write CSV header
        wtr.write_record(&["Date", "Hours", "Duration", "Projects", "Tasks"])?;
        
        // Sort days
        let mut dates: Vec<NaiveDate> = days.keys().cloned().collect();
        dates.sort();
        
        // Write each day
        for date in dates {
            let (duration, projects, tasks) = &days[&date];
            let hours = duration.num_seconds() as f64 / 3600.0;
            
            wtr.write_record(&[
                date.format("%Y-%m-%d").to_string(),
                format!("{:.1}", hours),
                util::format_duration(*duration),
                projects.join(", "),
                tasks.join(", "),
            ])?;
        }
    } else { // per_task
        // Write CSV header
        wtr.write_record(&["Date", "Projects", "Tasks", "Duration", "Type", "Comment"])?;
        
        // Sort activities by date and time
        let mut sorted_activities = activities.to_vec();
        sorted_activities.sort_by(|a, b| a.start.cmp(&b.start));
        
        // Write each activity
        for activity in sorted_activities {
            let date = activity.start.date_naive().format("%Y-%m-%d").to_string();
            let project = activity.project.clone().unwrap_or_default();
            let task = activity.task.clone();
            let duration_hours = activity.duration.num_seconds() as f64 / 3600.0;
            let activity_type = match activity.activity_type {
                ActivityType::Work => "WORK",
                ActivityType::Break => "BREAK",
                ActivityType::Ignored => "IGNORED",
            };
            let comment = activity.comment.clone().unwrap_or_default();
            
            wtr.write_record(&[
                date,
                project,
                task,
                format!("{:.1}", duration_hours),
                activity_type.to_string(),
                comment,
            ])?;
        }
    }
    
    // Get the CSV data as a string
    let data = String::from_utf8(wtr.into_inner()?)?;
    
    Ok(data)
}
