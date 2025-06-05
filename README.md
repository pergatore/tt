# Time Tracker TUI ⏱️

A beautiful terminal-based time tracker built with Go and the Charmbracelet ecosystem. Track your completed tasks with automatic duration calculation and enjoy a stunning interface.

![Time Tracker TUI Demo](https://img.shields.io/badge/TUI-Charmbracelet-purple)
![Go Version](https://img.shields.io/badge/Go-1.19+-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- 🎯 **Retrospective time tracking** - Log completed tasks with automatic duration calculation
- 🎨 **Beautiful TUI interface** - Styled with Charmbracelet's Lipgloss
- ⚡ **Simple workflow** - Start → Work → Complete tasks → View reports
- 📊 **Real-time summaries** - Work/break/total time with project breakdowns
- 🔄 **Task extension** - Extend the last task if you're still working on it
- 💾 **Persistent storage** - JSON-based data storage with automatic backups
- 🎨 **Color-coded activities** - Visual distinction between work, breaks, and ignored time

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

### Key Commands

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

### Workflow Example

1. **Start your day**
   ```
   Press 's' → Creates "Start" entry at current time
   ```

2. **Work on tasks** (time passes naturally)

3. **Complete tasks as you finish them**
   ```
   Press 'a' → "Meeting: Standup" → Optional comment
   Duration automatically calculated from last entry
   ```

4. **View your progress**
   ```
   Press 'r' → Beautiful report with time breakdown
   ```

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

### Main Dashboard
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

### Task Completion Flow
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
- [Conventional Commits](https://conventionalcommits.org/) for inspiration on task categorization
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
