# Time Tracker TUI ⏱️

A beautiful terminal-based time tracker built with Go and the Charmbracelet ecosystem. Track your completed tasks with automatic duration calculation and enjoy a stunning interface.

![Time Tracker TUI Demo](https://img.shields.io/badge/TUI-Charmbracelet-purple)
![Go Version](https://img.shields.io/badge/Go-1.19+-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- 🎯 **Retrospective time tracking** - Log completed tasks with automatic duration calculation
- 🎨 **Beautiful TUI interface** - Styled with Charmbracelet's Lipgloss
- ⚡ **CLI and TUI modes** - Quick command-line operations or interactive interface
- 📊 **Real-time summaries** - Work/break/total time with project breakdowns
- 🔄 **Task extension** - Extend the last task if you're still working on it
- 💾 **Persistent storage** - JSON-based data storage with automatic backups
- 🎨 **Color-coded activities** - Visual distinction between work, breaks, and ignored time
- 📋 **Project categorization** - Automatic parsing of "Project: Task" format

## 🚀 Installation

### Prerequisites
- Go 1.19 or later

### Quick Install
```bash
# Clone the repository
git clone https://github.com/pergatore/tt
cd tt

# Install dependencies
go mod tidy

# Run the application
go run main.go
```

### Build Binary
```bash
# Build for your platform
go build -o tt main.go

# Move to PATH (optional)
sudo mv tt /usr/local/bin/

# Or install with go install
go install
```

## 🎮 Usage

### Command Line Interface (CLI)

For quick operations, use CLI commands:

```bash
# Start your day
tt -s

# Add completed tasks
tt -a "Meeting: Standup"
tt -a "Education: CKA Labs" -c "Studied networking concepts"
tt -a "Lunch **"                    # Break task
tt -a "Commuting ***"               # Ignored task

# View today's report
tt -r

# Extend last task to current time
tt -x

# Show help
tt -h
```

### Terminal UI (TUI)

For interactive sessions, run without arguments:

```bash
# Launch beautiful TUI interface
tt
```

### TUI Key Commands

#### Navigation
- `↑/k, ↓/j` - Move up/down
- `←/h, →/l` - Move left/right  
- `Enter` - Select/confirm
- `Esc` - Go back
- `q` - Quit application

#### Actions
- `s` - **Start day** (creates initial timestamp)
- `a` - **Complete task** (log what you just finished)
- `r` - **View report** (detailed today's summary)
- `x` - **Extend last task** (continue working on previous task)
- `?` - **Toggle help** (show all commands)

### CLI Commands

```bash
tt                              # Launch TUI interface
tt -s                           # Start your day
tt -a "task name"               # Add completed task
tt -a "task name" -c "comment"  # Add task with comment
tt -r                           # Show today's report
tt -x                           # Extend last task
tt -h                           # Show CLI help
```

### Workflow Example

#### CLI Workflow (Quick & Scriptable)
```bash
# Start your day
tt -s

# Work on tasks throughout the day...
tt -a "Meeting: Daily standup"
tt -a "Development: User auth" -c "Implemented JWT tokens"  
tt -a "Lunch **"
tt -a "Code review" -c "Reviewed PR #123"

# Check your progress
tt -r
```

#### TUI Workflow (Interactive & Visual)
1. **Launch interface**
   ```bash
   tt
   ```

2. **Start your day** - Press `s`

3. **Work naturally** - Time passes as you focus on tasks

4. **Log completed work** - Press `a` when you finish something
   - Guided prompts for task name and optional comments
   - Automatic duration calculation from last entry

5. **Monitor progress** - Press `r` for beautiful reports

6. **Extend if needed** - Press `x` to continue previous task

### Task Types

The application supports three types of activities:

- **Work tasks**: `"Meeting: Standup"`, `"Development: Bug fixes"`
- **Break activities**: `"Lunch **"`, `"Coffee break **"` (end with `**`)
- **Ignored time**: `"Commuting ***"`, `"Personal call ***"` (end with `***`)

### Project Format

Use the `Project: Task` format to categorize your work:

```
Education: CKA Labs
Sprint-2: Bug fix #123
Admin: Email cleanup
Meeting: Daily standup
```

## 📊 Interface Overview

### CLI Report Output
```
📊 Today's Report
================

Work:  3h15
Break: 0h45
Total: 4h00

Projects:
  Meeting: 0h30
  Education: 1h45
  Development: 1h00

Activities:
  09:00-09:30  0h30  Meeting: Standup
  09:30-11:15  1h45  Education: CKA Labs
  11:15-12:00  0h45  Lunch ** [BREAK]
  12:00-13:00  1h00  Development: Bug fixes
```

### TUI Main Dashboard
```
⏱️  Time Tracker

Latest: Education: CKA Labs (45min ago)

Recent Activities:
  09:00-09:30  0h30  Meeting: Standup
  09:30-10:15  0h45  Education: CKA Labs
  10:15-12:00  1h45  Development: Bug fixes

Today's Summary:
  Work:  2h30
  Break: 0h30
  Total: 3h00

• Task completed: Education: CKA Labs (45min)

Press ? for help, q to quit
```

### TUI Task Completion Flow
```
✅ Task Completed

What task did you just finish?
Examples: 'Meeting: Standup', 'Lunch **', 'Commuting ***'

Duration: 1h15 (since 09:30)

[Meeting: Daily standup____________]

Enter to continue • Esc to cancel
```

## 📁 Data Storage

The application stores data in your system's standard configuration directory:

- **Linux/macOS**: `~/.config/timetracker/`
- **Windows**: `%APPDATA%\timetracker\`

### Files Created
- `config.json` - Application configuration
- `entries.json` - Your time tracking data

### Data Format
```json
[
  {
    "timestamp": "2025-01-15T09:00:00Z",
    "name": "Start",
    "comment": ""
  },
  {
    "timestamp": "2025-01-15T09:30:00Z", 
    "name": "Meeting: Standup",
    "comment": "Sprint planning discussion"
  }
]
```

## 🎨 Color Coding

- 🔵 **Work activities** - Blue text
- 🟠 **Break activities** - Orange text  
- ⚪ **Ignored activities** - Gray text
- 🟢 **Current status** - Green text
- 🔴 **Error messages** - Red text
- 🟢 **Success messages** - Green text

## 🏗️ Built With

- [Bubble Tea](https://github.com/charmbracelet/bubbletea) - Terminal UI framework
- [Bubbles](https://github.com/charmbracelet/bubbles) - Common TUI components
- [Lipgloss](https://github.com/charmbracelet/lipgloss) - Styling and layout

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Charmbracelet](https://charm.sh/) for the amazing TUI toolkit
- The Go community for excellent tooling and libraries

## 🐛 Issues & Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/pergatore/tt/issues) page
2. Create a new issue with:
   - Your operating system
   - Go version (`go version`)
   - Steps to reproduce the problem
   - Expected vs actual behavior

---

**Happy time tracking! ⏰**
