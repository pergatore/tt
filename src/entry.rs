use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

// Constants for activity types
pub const HELLO_ENTRY_NAME: &str = "hello";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Work,
    Break,
    Ignored,
}

// Implement Hash for ActivityType (required for HashMap keys)
impl Hash for ActivityType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ActivityType::Work => 0.hash(state),
            ActivityType::Break => 1.hash(state),
            ActivityType::Ignored => 2.hash(state),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub datetime: DateTime<Local>,
    pub name: String,
    pub is_current_entry: bool,
    pub comment: Option<String>,
}

impl Entry {
    pub fn new(datetime: DateTime<Local>, name: String, is_current_entry: bool, comment: Option<String>) -> Self {
        Entry {
            datetime,
            name,
            is_current_entry,
            comment,
        }
    }
    
    pub fn parse(line: &str) -> Result<Self> {
        // Parse entry with regular expressions
        // Format: "YYYY-MM-DD HH:MM[+/-HHMM] name [# comment]"
        let re = Regex::new(r"^(\d{4}-\d{1,2}-\d{1,2}\s+\d{1,2}:\d{1,2}(?:[+-]\d{2}:?\d{2})?)\s+([^\s].*?)(?:\s+#\s+(.+))?$")?;
        
        match re.captures(line) {
            Some(caps) => {
                let datetime_str = caps.get(1).unwrap().as_str();
                let name = caps.get(2).unwrap().as_str().trim().to_string();
                let comment = caps.get(3).map(|m| m.as_str().trim().to_string());

                // Try to parse the datetime with timezone
                let datetime = if datetime_str.contains('+') || datetime_str.contains('-') {
                    // Contains timezone information
                    let dt = DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M%z")
                        .or_else(|_| DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M%:z"))?;
                    dt.with_timezone(&Local)
                } else {
                    // No timezone, assume local
                    let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M")?;
                    Local.from_local_datetime(&naive).single().ok_or_else(|| {
                        anyhow::anyhow!("Failed to convert naive datetime to local timezone")
                    })?
                };

                Ok(Entry {
                    datetime,
                    name,
                    is_current_entry: false,
                    comment,
                })
            },
            None => bail!("Invalid entry format: {}", line),
        }
    }
    

    // This method is used in Activity::new()
    pub fn project(&self) -> Option<String> {
        let re = Regex::new(r"^([^\s:]+):\s+(.+)$").unwrap();
        
        if let Some(caps) = re.captures(&self.name) {
            Some(caps.get(1).unwrap().as_str().to_string())
        } else {
            None
        }
    }
    
    // This method is used in Activity::new()
    pub fn task(&self) -> String {
        let re = Regex::new(r"^([^\s:]+):\s+(.+)$").unwrap();
        
        if let Some(caps) = re.captures(&self.name) {
            caps.get(2).unwrap().as_str().to_string()
        } else {
            self.name.clone()
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.datetime.format("%Y-%m-%d %H:%M%z"), self.name)?;
        
        if let Some(comment) = &self.comment {
            write!(f, " # {}", comment)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub name: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub duration: Duration,
    pub activity_type: ActivityType,
    pub is_current_activity: bool,
    pub comment: Option<String>,
    pub project: Option<String>,
    pub task: String,
}

impl Activity {
    pub fn new(name: String, start: DateTime<Local>, end: DateTime<Local>, 
               is_current_activity: bool, comment: Option<String>) -> Self {
        let duration = end.signed_duration_since(start);
        let project = Entry::new(start, name.clone(), false, None).project();
        let task = Entry::new(start, name.clone(), false, None).task();
        let activity_type = if name.ends_with("***") {
            ActivityType::Ignored
        } else if name.ends_with("**") {
            ActivityType::Break
        } else {
            ActivityType::Work
        };
        
        Activity {
            name,
            start,
            end,
            duration,
            activity_type,
            is_current_activity,
            comment,
            project,
            task,
        }
    }
}
