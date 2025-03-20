# Timetracker (TT)

A simple command-line time tracking application written in Rust.

## Features

- Track your time spent on activities throughout the day
- Categorize activities by projects
- Record time when you start and finish tasks
- Generate detailed reports of your time usage
- Define different activity types (work, break, ignored)
- Export reports to CSV format
- Automatic handling of date ranges

## Quick Start

### Installation

1. Make sure you have Rust installed (https://rustup.rs/)
2. Clone this repository
3. Build and install the project:
   ```
   cargo install --path .
   ```
4. This will install the `tt` command to your PATH

### Basic Usage

1. Start your day:
   ```
   tt hello
   ```

2. Add tasks as you complete them:
   ```
   tt add "project-1: task-1"
   tt add "lunch **"  # break activity
   tt add "project-1: task-2"
   tt add "commuting ***"  # ignored activity
   ```

3. Generate a report for today:
   ```
   tt report
   ```

4. Edit your entries:
   ```
   tt edit
   ```

## Activity Types

- **Work activities**: Regular activities that count toward working time.
  Example: `tt add "coding"`

- **Break activities**: Activities that count toward break time, marked with `**` at the end.
  Example: `tt add "lunch **"`

- **Ignored activities**: Activities that are not counted in reports, marked with `***` at the end.
  Example: `tt add "commuting ***"`

## Project Notation

You can group activities by projects using the `project: task` notation:

```
tt add "project-1: implementing feature"
tt add "project-2: bug fix"
```

## Commands

### `hello`

Marks the beginning of your day:
```
tt hello
```

### `add`

Adds a completed task:
```
tt add [task description]
```

Add with a comment:
```
tt add "project: task" --comment "details about the task"
```

### `report`

Generate a report of your activities:
```
tt report
```

Report for a specific date:
```
tt report 2023-10-15
```

Report for a date range:
```
tt report --from 2023-10-10 --to 2023-10-15
```

Filter by project:
```
tt report --project "project-1"
```

Export as CSV:
```
tt report --csv-section per_day
tt report --csv-section per_task
```

Additional options:
- `--details`: Show detailed breakdown even for multi-day reports
- `--comments`: Include comments in the report
- `--per-day`: Group activities by day
- `--no-current-activity`: Don't include current activity in the report
- `--month [this|prev|YYYY-MM]`: Show report for a specific month
- `--week [this|prev|number]`: Show report for a specific week

### `stretch`

Extend the previous task to the current time:
```
tt stretch
```

### `edit`

Edit your time log file in your default text editor:
```
tt edit
```

### `config`

View or modify configuration:
```
tt config
tt config --filename
tt config --default
```

## Configuration

The configuration file is stored at:
- Linux/macOS: `~/.config/timetracker/tt.json`
- Windows: `%APPDATA%\timetracker\tt.json`

Settings include:
- Data file location
- Default editor
- Timezone settings

## Data File Format

Each entry follows this format:
```
YYYY-MM-DD HH:MM[+/-HHMM] activity name [# comment]
```

Example:
```
2023-10-15 09:00 hello
2023-10-15 10:30 project-1: implementing feature # added initial structure
2023-10-15 12:00 lunch **
2023-10-15 13:00 project-1: implementing feature
```

## License

This software is released under the GPL-3.0 License.
