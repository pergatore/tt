use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Weekday};

pub fn parse_date_string(date_str: &str, today: &DateTime<Local>, is_past: bool) -> Result<NaiveDate> {
    // First try to parse as a day name
    if let Some(date) = parse_day_name(date_str, today.date_naive(), is_past) {
        return Ok(date);
    }
    
    // Try to parse as an absolute date
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date);
    }
    
    // Try to parse as a relative reference like "today" or "yesterday"
    if "today".starts_with(&date_str.to_lowercase()) {
        return Ok(today.date_naive());
    }
    
    if "yesterday".starts_with(&date_str.to_lowercase()) {
        return Ok(today.date_naive().pred_opt().unwrap());
    }
    
    Err(anyhow!("Invalid date format: {}", date_str))
}

pub fn parse_day_name(day_name: &str, today: NaiveDate, is_past: bool) -> Option<NaiveDate> {
    let day = match day_name.to_lowercase().as_str() {
        d if "monday".starts_with(d) => Weekday::Mon,
        d if "tuesday".starts_with(d) => Weekday::Tue,
        d if "wednesday".starts_with(d) => Weekday::Wed,
        d if "thursday".starts_with(d) => Weekday::Thu,
        d if "friday".starts_with(d) => Weekday::Fri,
        d if "saturday".starts_with(d) => Weekday::Sat,
        d if "sunday".starts_with(d) => Weekday::Sun,
        _ => return None,
    };
    
    let today_weekday = today.weekday();
    
    if is_past {
        // Find the most recent occurrence of the specified weekday
        let days_back = (7 + today_weekday.num_days_from_monday() - day.num_days_from_monday()) % 7;
        Some(today - chrono::Duration::days(days_back as i64))
    } else {
        // Find the next occurrence of the specified weekday
        let days_forward = (7 + day.num_days_from_monday() - today_weekday.num_days_from_monday()) % 7;
        Some(today + chrono::Duration::days(days_forward as i64))
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let total_minutes = total_seconds / 60;
    let (hours, minutes) = (total_minutes / 60, total_minutes % 60);
    
    format!("{}h{:02}", hours, minutes)
}

pub fn beginning_of_day(date: NaiveDate) -> DateTime<Local> {
    let naive = date.and_hms_opt(0, 0, 0).unwrap();
    Local.from_utc_datetime(&naive)
}

pub fn end_of_day(date: NaiveDate) -> DateTime<Local> {
    let naive = date.and_hms_opt(23, 59, 59).unwrap();
    Local.from_utc_datetime(&naive)
}

pub fn parse_week(week_str: &str, today: NaiveDate) -> Result<(NaiveDate, NaiveDate)> {
    let first_day = match week_str.to_lowercase().as_str() {
        "this" => {
            // Get the Monday of current week
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days(days_since_monday as i64)
        },
        "prev" | "previous" => {
            // Get the Monday of previous week
            let days_since_monday = today.weekday().num_days_from_monday();
            today - chrono::Duration::days((days_since_monday + 7) as i64)
        },
        _ => {
            // Try to parse as week number
            match week_str.parse::<i32>() {
                Ok(week_num) => {
                    if week_num <= 0 || week_num > 53 {
                        return Err(anyhow!("Week number must be between 1 and 53"));
                    }
                    
                    let year = today.year();
                    // This is a simplification - proper ISO week calculation is more complex
                    let first_day_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                    let days_to_monday = (first_day_of_year.weekday().num_days_from_monday() + 7 - 1) % 7;
                    let first_monday = first_day_of_year + chrono::Duration::days(days_to_monday as i64);
                    
                    first_monday + chrono::Duration::days((week_num - 1) as i64 * 7)
                },
                Err(_) => return Err(anyhow!("Invalid week format: {}", week_str)),
            }
        }
    };
    
    let last_day = first_day + chrono::Duration::days(6); // Sunday
    
    Ok((first_day, last_day))
}

pub fn parse_month(month_str: &str, today: NaiveDate) -> Result<(NaiveDate, NaiveDate)> {
    let (year, month) = match month_str.to_lowercase().as_str() {
        "this" => (today.year(), today.month()),
        "prev" | "previous" => {
            let mut year = today.year();
            let mut month = today.month() - 1;
            
            if month == 0 {
                month = 12;
                year -= 1;
            }
            
            (year, month)
        },
        _ if month_str.contains('-') => {
            // Parse as YYYY-MM
            let parts: Vec<&str> = month_str.split('-').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid month format: {}", month_str));
            }
            
            let year = parts[0].parse::<i32>()?;
            let month = parts[1].parse::<u32>()?;
            
            if month < 1 || month > 12 {
                return Err(anyhow!("Month must be between 1 and 12"));
            }
            
            (year, month)
        },
        _ => {
            // Try to parse as month name
            let month = match month_str.to_lowercase().as_str() {
                m if "january".starts_with(m) => 1,
                m if "february".starts_with(m) => 2,
                m if "march".starts_with(m) => 3,
                m if "april".starts_with(m) => 4,
                m if "may".starts_with(m) => 5,
                m if "june".starts_with(m) => 6,
                m if "july".starts_with(m) => 7,
                m if "august".starts_with(m) => 8,
                m if "september".starts_with(m) => 9,
                m if "october".starts_with(m) => 10,
                m if "november".starts_with(m) => 11,
                m if "december".starts_with(m) => 12,
                _ => return Err(anyhow!("Invalid month name: {}", month_str)),
            };
            
            let year = if month > today.month() {
                today.year() - 1
            } else {
                today.year()
            };
            
            (year, month)
        }
    };
    
    let first_day = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| anyhow!("Invalid date: {}-{}-01", year, month))?;
    
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    };
    
    Ok((first_day, last_day))
}

pub fn parse_now_arg(now_arg: Option<&str>) -> Result<DateTime<Local>> {
    match now_arg {
        Some(time_str) => {
            let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M")?;
            Ok(Local.from_utc_datetime(&naive))
        },
        None => Ok(Local::now()),
    }
}

pub fn format_date_title(date: NaiveDate) -> String {
    let weekday = match date.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    };
    
    let month = match date.month() {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => unreachable!(),
    };
    
    let iso_week = date.iso_week();
    let week_num = iso_week.week();
    
    format!("{}, {} {:02}, {} (week {})", weekday, month, date.day(), date.year(), week_num)
}

pub fn format_title(text: &str) -> String {
    format!("{:-^80}", format!(" {} ", text))
}
