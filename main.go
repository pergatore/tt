package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// Styles
var (
	titleStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FAFAFA")).
			Background(lipgloss.Color("#7D56F4")).
			Padding(0, 1).
			Bold(true)

	subtitleStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#7D56F4")).
			Bold(true)

	infoStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#626262")).
			Italic(true)

	errorStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FF5F87")).
			Bold(true)

	successStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#50FA7B")).
			Bold(true)

	workStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#8BE9FD"))

	breakStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#FFB86C"))

	ignoredStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#6272A4"))

	currentActivityStyle = lipgloss.NewStyle().
				Foreground(lipgloss.Color("#50FA7B")).
				Bold(true)

	helpStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#626262"))

	docStyle = lipgloss.NewStyle().Padding(1, 2, 1, 2)
)

// Data structures
type ActivityType int

const (
	Work ActivityType = iota
	Break
	Ignored
)

func (a ActivityType) String() string {
	switch a {
	case Work:
		return "WORK"
	case Break:
		return "BREAK"
	case Ignored:
		return "IGNORED"
	default:
		return "UNKNOWN"
	}
}

type Entry struct {
	Timestamp time.Time `json:"timestamp"`
	Name      string    `json:"name"`
	Comment   string    `json:"comment,omitempty"`
}

type Activity struct {
	Name     string
	Start    time.Time
	End      time.Time
	Duration time.Duration
	Type     ActivityType
	Project  string
	Task     string
	Comment  string
	IsCurrent bool
}

type Config struct {
	DataFile string `json:"data_file"`
	Editor   string `json:"editor"`
}

type TimeTracker struct {
	entries []Entry
	config  Config
}

// Views
type viewType int

const (
	mainView viewType = iota
	addTaskView
	reportView
	helpView
)

// Key mappings
type keyMap struct {
	Up       key.Binding
	Down     key.Binding
	Left     key.Binding
	Right    key.Binding
	Help     key.Binding
	Quit     key.Binding
	Enter    key.Binding
	Back     key.Binding
	AddTask  key.Binding
	Report   key.Binding
	Hello    key.Binding
	Stretch  key.Binding
}

func (k keyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Help, k.Quit}
}

func (k keyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Up, k.Down, k.Left, k.Right},
		{k.AddTask, k.Report, k.Hello, k.Stretch},
		{k.Enter, k.Back, k.Help, k.Quit},
	}
}

var keys = keyMap{
	Up: key.NewBinding(
		key.WithKeys("up", "k"),
		key.WithHelp("↑/k", "move up"),
	),
	Down: key.NewBinding(
		key.WithKeys("down", "j"),
		key.WithHelp("↓/j", "move down"),
	),
	Left: key.NewBinding(
		key.WithKeys("left", "h"),
		key.WithHelp("←/h", "move left"),
	),
	Right: key.NewBinding(
		key.WithKeys("right", "l"),
		key.WithHelp("→/l", "move right"),
	),
	Help: key.NewBinding(
		key.WithKeys("?"),
		key.WithHelp("?", "toggle help"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "esc", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
	Enter: key.NewBinding(
		key.WithKeys("enter"),
		key.WithHelp("enter", "select"),
	),
	Back: key.NewBinding(
		key.WithKeys("esc"),
		key.WithHelp("esc", "back"),
	),
	AddTask: key.NewBinding(
		key.WithKeys("a"),
		key.WithHelp("a", "complete task"),
	),
	Report: key.NewBinding(
		key.WithKeys("r"),
		key.WithHelp("r", "view report"),
	),
	Hello: key.NewBinding(
		key.WithKeys("s"),
		key.WithHelp("s", "start day"),
	),
	Stretch: key.NewBinding(
		key.WithKeys("x"),
		key.WithHelp("x", "extend last task"),
	),
}

// Model
type model struct {
	tracker       *TimeTracker
	currentView   viewType
	width         int
	height        int
	ready         bool
	
	// Components
	help       help.Model
	taskInput  textinput.Model
	viewport   viewport.Model
	table      table.Model
	
	// State
	message    string
	messageType string // "error", "success", "info"
	
	// Add task form
	taskName    string
	taskComment string
	inputMode   int // 0 = name, 1 = comment
}

func initialModel() model {
	tracker := &TimeTracker{}
	tracker.loadConfig()
	tracker.loadEntries()

	// Initialize task input
	ti := textinput.New()
	ti.Placeholder = "Enter task name (e.g., 'Education: CKA Labs' or 'Lunch **')"
	ti.Focus()
	ti.CharLimit = 156
	ti.Width = 50

	// Initialize help
	h := help.New()
	h.Width = 50

	// Initialize viewport
	vp := viewport.New(78, 20)
	vp.Style = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("62")).
		PaddingRight(2)

	// Initialize table
	columns := []table.Column{
		{Title: "Time", Width: 10},
		{Title: "Duration", Width: 12},
		{Title: "Activity", Width: 40},
		{Title: "Type", Width: 8},
	}

	t := table.New(
		table.WithColumns(columns),
		table.WithFocused(true),
		table.WithHeight(15),
	)

	s := table.DefaultStyles()
	s.Header = s.Header.
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("240")).
		BorderBottom(true).
		Bold(false)
	s.Selected = s.Selected.
		Foreground(lipgloss.Color("229")).
		Background(lipgloss.Color("57")).
		Bold(false)
	t.SetStyles(s)

	return model{
		tracker:     tracker,
		currentView: mainView,
		help:        h,
		taskInput:   ti,
		viewport:    vp,
		table:       t,
		inputMode:   0,
	}
}

func (m model) Init() tea.Cmd {
	return tea.EnterAltScreen
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		m.viewport.Width = msg.Width - 4
		m.viewport.Height = msg.Height - 10
		m.help.Width = msg.Width
		m.ready = true

	case tea.KeyMsg:
		switch m.currentView {
		case mainView:
			return m.updateMainView(msg)
		case addTaskView:
			return m.updateAddTaskView(msg)
		case reportView:
			return m.updateReportView(msg)
		case helpView:
			return m.updateHelpView(msg)
		}
	}

	// Only update components that aren't being actively used for input
	if m.currentView != addTaskView {
		m.taskInput, cmd = m.taskInput.Update(msg)
		cmds = append(cmds, cmd)
	}

	m.viewport, cmd = m.viewport.Update(msg)
	cmds = append(cmds, cmd)

	m.table, cmd = m.table.Update(msg)
	cmds = append(cmds, cmd)

	return m, tea.Batch(cmds...)
}

func (m model) updateMainView(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch {
	case key.Matches(msg, keys.Quit):
		return m, tea.Quit
	case key.Matches(msg, keys.AddTask):
		m.currentView = addTaskView
		m.taskInput.SetValue("")
		m.taskInput.Focus()
		m.inputMode = 0
		m.message = ""
		m.messageType = ""
	case key.Matches(msg, keys.Report):
		m.currentView = reportView
		m.updateReportData()
	case key.Matches(msg, keys.Hello):
		m.tracker.addStart()
		m.message = "Day started!"
		m.messageType = "success"
	case key.Matches(msg, keys.Stretch):
		err := m.tracker.extend()
		if err != nil {
			m.message = fmt.Sprintf("Error: %v", err)
			m.messageType = "error"
		} else {
			m.message = "Task extended to current time!"
			m.messageType = "success"
		}
	case key.Matches(msg, keys.Help):
		m.currentView = helpView
	}
	return m, nil
}

func (m model) updateAddTaskView(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	
	switch {
	case key.Matches(msg, keys.Back):
		m.currentView = mainView
		m.taskInput.Blur()
		m.message = ""
		return m, nil
	case key.Matches(msg, keys.Enter):
		if m.inputMode == 0 {
			// Save task name and move to comment
			m.taskName = m.taskInput.Value()
			if m.taskName == "" {
				m.message = "Task name cannot be empty"
				m.messageType = "error"
				return m, nil
			}
			m.inputMode = 1
			m.taskInput.SetValue("")
			m.taskInput.Placeholder = "Optional comment (press Enter to skip)"
			m.taskInput.Focus()
		} else {
			// Save comment and add task
			m.taskComment = m.taskInput.Value()
			
			entry := Entry{
				Timestamp: time.Now(),
				Name:      m.taskName,
				Comment:   m.taskComment,
			}
			
			err := m.tracker.addEntry(entry)
			if err != nil {
				m.message = fmt.Sprintf("Error adding task: %v", err)
				m.messageType = "error"
			} else {
				// Calculate duration from last entry
				var durationMsg string
				if len(m.tracker.entries) > 1 {
					lastEntry := m.tracker.entries[len(m.tracker.entries)-2] // Previous entry before the one we just added
					duration := entry.Timestamp.Sub(lastEntry.Timestamp)
					durationMsg = fmt.Sprintf(" (%s)", formatDuration(duration))
				}
				m.message = fmt.Sprintf("Task completed: %s%s", m.taskName, durationMsg)
				m.messageType = "success"
				m.currentView = mainView
				m.taskInput.Blur()
			}
			
			// Reset form
			m.taskName = ""
			m.taskComment = ""
			m.inputMode = 0
			m.taskInput.Placeholder = "Enter task name (e.g., 'Education: CKA Labs' or 'Lunch **')"
		}
		return m, nil
	default:
		// Let the text input handle other keys
		m.taskInput, cmd = m.taskInput.Update(msg)
		return m, cmd
	}
}

func (m model) updateReportView(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch {
	case key.Matches(msg, keys.Back):
		m.currentView = mainView
	case key.Matches(msg, keys.Quit):
		return m, tea.Quit
	}
	return m, nil
}

func (m model) updateHelpView(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch {
	case key.Matches(msg, keys.Back), key.Matches(msg, keys.Help):
		m.currentView = mainView
	case key.Matches(msg, keys.Quit):
		return m, tea.Quit
	}
	return m, nil
}

func (m *model) updateReportData() {
	activities := m.tracker.getTodaysActivities()
	
	rows := []table.Row{}
	for _, activity := range activities {
		timeStr := activity.Start.Format("15:04") + "-" + activity.End.Format("15:04")
		durationStr := formatDuration(activity.Duration)
		activityName := activity.Name
		
		rows = append(rows, table.Row{
			timeStr,
			durationStr,
			activityName,
			activity.Type.String(),
		})
	}
	
	m.table.SetRows(rows)
	
	// Generate summary for viewport
	summary := m.tracker.generateTodaysSummary()
	m.viewport.SetContent(summary)
}

func (m model) View() string {
	if !m.ready {
		return "\n  Initializing..."
	}

	switch m.currentView {
	case mainView:
		return m.mainViewRender()
	case addTaskView:
		return m.addTaskViewRender()
	case reportView:
		return m.reportViewRender()
	case helpView:
		return m.helpViewRender()
	default:
		return "Unknown view"
	}
}

func (m model) mainViewRender() string {
	title := titleStyle.Render("⏱️  Time Tracker")
	
	// Current status
	status := m.tracker.getCurrentStatus()
	
	// Recent activities (last 5)
	recentActivities := m.tracker.getRecentActivities(5)
	var recent strings.Builder
	recent.WriteString(subtitleStyle.Render("Recent Activities:") + "\n\n")
	
	if len(recentActivities) == 0 {
		recent.WriteString(infoStyle.Render("No activities yet. Press 's' to start your day or 'a' to complete a task."))
	} else {
		for _, activity := range recentActivities {
			timeStr := activity.Start.Format("15:04") + "-" + activity.End.Format("15:04")
			durationStr := formatDuration(activity.Duration)
			
			var style lipgloss.Style
			switch activity.Type {
			case Work:
				style = workStyle
			case Break:
				style = breakStyle
			case Ignored:
				style = ignoredStyle
			}
			
			// Use a simple, consistent format
			line := fmt.Sprintf("  %s  %s  %s", timeStr, durationStr, activity.Name)
			recent.WriteString(style.Render(line) + "\n")
		}
	}
	
	// Quick stats
	stats := m.tracker.getTodaysStats()
	quickStats := fmt.Sprintf("\n%s\n%s\n%s\n%s",
		subtitleStyle.Render("Today's Summary:"),
		workStyle.Render(fmt.Sprintf("  Work:  %s", formatDuration(stats.WorkTime))),
		breakStyle.Render(fmt.Sprintf("  Break: %s", formatDuration(stats.BreakTime))),
		subtitleStyle.Render(fmt.Sprintf("  Total: %s", formatDuration(stats.TotalTime))))
	
	// Project breakdown for main view
	projects := m.tracker.getTodaysProjects()
	// Debug: Always show the projects section to see what's in it
	quickStats += "\n\n" + subtitleStyle.Render("Projects:")
	if len(projects) == 0 {
		quickStats += "\n" + infoStyle.Render("  No projects found")
	} else {
		for project, duration := range projects {
			if project == "" {
				project = "General"
			}
			quickStats += "\n" + workStyle.Render(fmt.Sprintf("  %s: %s", project, formatDuration(duration)))
		}
	}
	
	// Message
	var message string
	if m.message != "" {
		switch m.messageType {
		case "error":
			message = "\n" + errorStyle.Render("• "+m.message)
		case "success":
			message = "\n" + successStyle.Render("• "+m.message)
		default:
			message = "\n" + infoStyle.Render("• "+m.message)
		}
	}
	
	// Help
	helpView := "\n" + helpStyle.Render("Press ? for help, q to quit")
	
	content := lipgloss.JoinVertical(lipgloss.Left,
		title,
		"",
		status,
		"",
		recent.String(),
		quickStats,
		message,
		helpView,
	)
	
	return docStyle.Render(content)
}

func (m model) addTaskViewRender() string {
	title := titleStyle.Render("✅ Task Completed")
	
	var prompt string
	if m.inputMode == 0 {
		prompt = subtitleStyle.Render("What task did you just finish?")
		prompt += "\n" + infoStyle.Render("Examples: 'Meeting: Standup', 'Lunch **', 'Commuting ***'")
		
		// Show duration since last activity
		if len(m.tracker.entries) > 0 {
			lastEntry := m.tracker.entries[len(m.tracker.entries)-1]
			duration := time.Since(lastEntry.Timestamp)
			prompt += "\n" + workStyle.Render(fmt.Sprintf("Duration: %s (since %s)", 
				formatDuration(duration), lastEntry.Timestamp.Format("15:04")))
		}
	} else {
		prompt = subtitleStyle.Render("Comment (optional):")
		prompt += "\n" + infoStyle.Render("Task: ") + workStyle.Render(m.taskName)
		
		// Show the duration this task will have
		if len(m.tracker.entries) > 0 {
			lastEntry := m.tracker.entries[len(m.tracker.entries)-1]
			duration := time.Since(lastEntry.Timestamp)
			prompt += "\n" + workStyle.Render(fmt.Sprintf("This task took: %s", formatDuration(duration)))
		}
	}
	
	input := m.taskInput.View()
	
	var message string
	if m.message != "" {
		switch m.messageType {
		case "error":
			message = errorStyle.Render("• " + m.message)
		default:
			message = infoStyle.Render("• " + m.message)
		}
	}
	
	help := helpStyle.Render("Enter to continue • Esc to cancel")
	
	content := lipgloss.JoinVertical(lipgloss.Left,
		title,
		"",
		prompt,
		"",
		input,
		"",
		message,
		"",
		help,
	)
	
	return docStyle.Render(content)
}

func (m model) reportViewRender() string {
	title := titleStyle.Render("📊 Today's Report")
	
	// Summary in viewport
	summary := m.viewport.View()
	
	// Activities table
	table := m.table.View()
	
	help := helpStyle.Render("Esc to go back • q to quit")
	
	content := lipgloss.JoinVertical(lipgloss.Left,
		title,
		"",
		summary,
		"",
		subtitleStyle.Render("Activities:"),
		"",
		table,
		"",
		help,
	)
	
	return docStyle.Render(content)
}

func (m model) helpViewRender() string {
	title := titleStyle.Render("❓ Help")
	
	helpContent := `
` + subtitleStyle.Render("Navigation:") + `
  ↑/k, ↓/j     Move up/down
  ←/h, →/l     Move left/right
  Enter        Select/confirm
  Esc          Go back
  q            Quit

` + subtitleStyle.Render("Actions:") + `
  s            Start day
  a            Complete task (add finished task)
  r            View today's report
  x            Extend last task to now
  ?            Toggle this help

` + subtitleStyle.Render("Task Types:") + `
  Regular task        "Meeting: Standup"
  Break task (**)     "Lunch **"
  Ignored task (***)  "Commuting ***"

` + subtitleStyle.Render("Project Format:") + `
  Use "Project: Task" to categorize activities
  Examples: "Education: CKA Labs", "Sprint-2: Bug fix"
`
	
	back := helpStyle.Render("Press ? or Esc to go back")
	
	content := lipgloss.JoinVertical(lipgloss.Left,
		title,
		helpContent,
		"",
		back,
	)
	
	return docStyle.Render(content)
}

// TimeTracker methods
func (tt *TimeTracker) loadConfig() {
	homeDir, _ := os.UserHomeDir()
	configDir := filepath.Join(homeDir, ".config", "timetracker")
	configFile := filepath.Join(configDir, "config.json")
	
	// Default config
	tt.config = Config{
		DataFile: filepath.Join(configDir, "entries.json"),
		Editor:   "vi",
	}
	
	// Try to load existing config
	if data, err := os.ReadFile(configFile); err == nil {
		json.Unmarshal(data, &tt.config)
	} else {
		// Create config directory and save default config
		os.MkdirAll(configDir, 0755)
		data, _ := json.MarshalIndent(tt.config, "", "  ")
		os.WriteFile(configFile, data, 0644)
	}
}

func (tt *TimeTracker) loadEntries() {
	if data, err := os.ReadFile(tt.config.DataFile); err == nil {
		json.Unmarshal(data, &tt.entries)
	}
	
	// Sort entries by timestamp
	sort.Slice(tt.entries, func(i, j int) bool {
		return tt.entries[i].Timestamp.Before(tt.entries[j].Timestamp)
	})
}

func (tt *TimeTracker) saveEntries() error {
	// Ensure directory exists
	dir := filepath.Dir(tt.config.DataFile)
	os.MkdirAll(dir, 0755)
	
	data, err := json.MarshalIndent(tt.entries, "", "  ")
	if err != nil {
		return err
	}
	
	return os.WriteFile(tt.config.DataFile, data, 0644)
}

func (tt *TimeTracker) addEntry(entry Entry) error {
	tt.entries = append(tt.entries, entry)
	return tt.saveEntries()
}

func (tt *TimeTracker) addStart() error {
	entry := Entry{
		Timestamp: time.Now(),
		Name:      "Start",
	}
	return tt.addEntry(entry)
}

func (tt *TimeTracker) extend() error {
	if len(tt.entries) == 0 {
		return fmt.Errorf("no entries to extend")
	}
	
	lastEntry := tt.entries[len(tt.entries)-1]
	if lastEntry.Name == "Start" {
		return fmt.Errorf("cannot extend start entry")
	}
	
	entry := Entry{
		Timestamp: time.Now(),
		Name:      lastEntry.Name,
		Comment:   lastEntry.Comment,
	}
	
	return tt.addEntry(entry)
}

func (tt *TimeTracker) getCurrentStatus() string {
	if len(tt.entries) == 0 {
		return infoStyle.Render("No activities yet. Start your day!")
	}
	
	lastEntry := tt.entries[len(tt.entries)-1]
	duration := time.Since(lastEntry.Timestamp)
	
	if lastEntry.Name == "Start" {
		return currentActivityStyle.Render(fmt.Sprintf("Day started (%s ago)", 
			formatDuration(duration)))
	}
	
	return currentActivityStyle.Render(fmt.Sprintf("Latest: %s (%s ago)", 
		lastEntry.Name, formatDuration(duration)))
}

func (tt *TimeTracker) getRecentActivities(limit int) []Activity {
	activities := tt.getTodaysActivities()
	
	if len(activities) > limit {
		return activities[len(activities)-limit:]
	}
	return activities
}

func (tt *TimeTracker) getTodaysActivities() []Activity {
	today := time.Now().Truncate(24 * time.Hour)
	var todaysEntries []Entry
	
	// Get today's entries
	for _, entry := range tt.entries {
		if entry.Timestamp.After(today) {
			todaysEntries = append(todaysEntries, entry)
		}
	}
	
	if len(todaysEntries) == 0 {
		return []Activity{}
	}
	
	var activities []Activity
	
	// Convert entries to activities (each activity represents time between entries)
	for i := 0; i < len(todaysEntries); i++ {
		entry := todaysEntries[i]
		
		// Skip start entries - they don't represent completed work
		if entry.Name == "Start" {
			continue
		}
		
		// Find the previous entry to calculate duration
		var start time.Time
		if i == 0 {
			// If this is the first entry, we can't calculate duration
			continue
		} else {
			start = todaysEntries[i-1].Timestamp
		}
		
		end := entry.Timestamp
		
		activity := parseActivity(entry, start, end, false) // No "current" activities anymore
		activities = append(activities, activity)
	}
	
	return activities
}

func (tt *TimeTracker) getTodaysStats() struct {
	WorkTime  time.Duration
	BreakTime time.Duration
	TotalTime time.Duration
} {
	activities := tt.getTodaysActivities()
	
	var workTime, breakTime time.Duration
	
	for _, activity := range activities {
		switch activity.Type {
		case Work:
			workTime += activity.Duration
		case Break:
			breakTime += activity.Duration
		}
	}
	
	return struct {
		WorkTime  time.Duration
		BreakTime time.Duration
		TotalTime time.Duration
	}{
		WorkTime:  workTime,
		BreakTime: breakTime,
		TotalTime: workTime + breakTime,
	}
}

func (tt *TimeTracker) getTodaysProjects() map[string]time.Duration {
	activities := tt.getTodaysActivities()
	projects := make(map[string]time.Duration)
	
	for _, activity := range activities {
		if activity.Type == Work {
			projects[activity.Project] += activity.Duration
		}
	}
	
	return projects
}

func (tt *TimeTracker) generateTodaysSummary() string {
	stats := tt.getTodaysStats()
	activities := tt.getTodaysActivities()
	
	var summary strings.Builder
	
	// Time summary
	summary.WriteString(subtitleStyle.Render("Time Summary:") + "\n\n")
	summary.WriteString(workStyle.Render(fmt.Sprintf("  Work:  %s\n", formatDuration(stats.WorkTime))))
	summary.WriteString(breakStyle.Render(fmt.Sprintf("  Break: %s\n", formatDuration(stats.BreakTime))))
	summary.WriteString(subtitleStyle.Render(fmt.Sprintf("  Total: %s\n\n", formatDuration(stats.TotalTime))))
	
	// Project breakdown
	projects := make(map[string]time.Duration)
	for _, activity := range activities {
		if activity.Type == Work && activity.Project != "" {
			projects[activity.Project] += activity.Duration
		}
	}
	
	if len(projects) > 0 {
		summary.WriteString(subtitleStyle.Render("Projects:") + "\n\n")
		for project, duration := range projects {
			summary.WriteString(workStyle.Render(fmt.Sprintf("  %s: %s\n", project, formatDuration(duration))))
		}
	}
	
	return summary.String()
}

// Helper functions
func parseActivity(entry Entry, start, end time.Time, isCurrent bool) Activity {
	name := entry.Name
	activityType := Work
	project := ""
	task := name
	
	// Determine activity type
	if strings.HasSuffix(name, "***") {
		activityType = Ignored
		name = strings.TrimSuffix(name, " ***")
		task = name
	} else if strings.HasSuffix(name, "**") {
		activityType = Break
		name = strings.TrimSuffix(name, " **")
		task = name
	}
	
	// Parse project:task format
	if strings.Contains(name, ":") {
		parts := strings.SplitN(name, ":", 2)
		if len(parts) == 2 {
			project = strings.TrimSpace(parts[0])
			task = strings.TrimSpace(parts[1])
		}
	}
	
	return Activity{
		Name:      name,
		Start:     start,
		End:       end,
		Duration:  end.Sub(start),
		Type:      activityType,
		Project:   project,
		Task:      task,
		Comment:   entry.Comment,
		IsCurrent: isCurrent,
	}
}

func formatDuration(d time.Duration) string {
	hours := int(d.Hours())
	minutes := int(d.Minutes()) % 60
	return fmt.Sprintf("%dh%02d", hours, minutes)
}

func printCLIHelp() {
	fmt.Println("tt - Time Tracker")
	fmt.Println()
	fmt.Println("USAGE:")
	fmt.Println("  tt                    Start TUI interface")
	fmt.Println("  tt [command]          Run command and exit")
	fmt.Println()
	fmt.Println("COMMANDS:")
	fmt.Println("  -s                    Start your day")
	fmt.Println("  -a \"task name\"        Add completed task")
	fmt.Println("  -c \"comment\"          Add comment (use with -a)")
	fmt.Println("  -r                    Show today's report")
	fmt.Println("  -x                    Extend last task to now")
	fmt.Println("  -h                    Show this help")
	fmt.Println()
	fmt.Println("EXAMPLES:")
	fmt.Println("  tt -s                 # Start your day")
	fmt.Println("  tt -a \"Meeting: Standup\"")
	fmt.Println("  tt -a \"Lunch **\"      # Break task")
	fmt.Println("  tt -a \"Dev work\" -c \"Fixed login bug\"")
	fmt.Println("  tt -r                 # View today's report")
	fmt.Println("  tt -x                 # Extend last task")
	fmt.Println()
	fmt.Println("TASK TYPES:")
	fmt.Println("  Regular task:    \"Meeting: Standup\"")
	fmt.Println("  Break task:      \"Lunch **\"")
	fmt.Println("  Ignored task:    \"Commuting ***\"")
}

func printTodaysReport(tracker *TimeTracker) {
	activities := tracker.getTodaysActivities()
	stats := tracker.getTodaysStats()
	
	fmt.Println("📊 Today's Report")
	fmt.Println("================")
	fmt.Println()
	
	// Summary
	fmt.Printf("Work:  %s\n", formatDuration(stats.WorkTime))
	fmt.Printf("Break: %s\n", formatDuration(stats.BreakTime))
	fmt.Printf("Total: %s\n", formatDuration(stats.TotalTime))
	fmt.Println()
	
	// Projects
	projects := tracker.getTodaysProjects()
	if len(projects) > 0 {
		fmt.Println("Projects:")
		for project, duration := range projects {
			if project == "" {
				project = "General"
			}
			fmt.Printf("  %s: %s\n", project, formatDuration(duration))
		}
		fmt.Println()
	}
	
	// Activities
	if len(activities) > 0 {
		fmt.Println("Activities:")
		for _, activity := range activities {
			timeStr := activity.Start.Format("15:04") + "-" + activity.End.Format("15:04")
			typeStr := ""
			switch activity.Type {
			case Break:
				typeStr = " [BREAK]"
			case Ignored:
				typeStr = " [IGNORED]"
			}
			
			fmt.Printf("  %s  %s  %s%s\n", 
				timeStr, 
				formatDuration(activity.Duration), 
				activity.Name,
				typeStr)
		}
	} else {
		fmt.Println("No activities logged today.")
	}
}

func main() {
	// Parse command line flags
	var (
		addTask    = flag.String("a", "", "Add a completed task")
		startDay   = flag.Bool("s", false, "Start your day")
		showReport = flag.Bool("r", false, "Show today's report")
		extend     = flag.Bool("x", false, "Extend last task to current time")
		showHelp   = flag.Bool("h", false, "Show help")
		comment    = flag.String("c", "", "Add comment to task (use with -a)")
	)
	flag.Parse()

	// Handle CLI commands
	if *showHelp {
		printCLIHelp()
		return
	}

	// Initialize tracker for CLI operations
	tracker := &TimeTracker{}
	tracker.loadConfig()
	tracker.loadEntries()

	if *startDay {
		err := tracker.addStart()
		if err != nil {
			fmt.Printf("Error starting day: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("✅ Day started!")
		return
	}

	if *addTask != "" {
		entry := Entry{
			Timestamp: time.Now(),
			Name:      *addTask,
			Comment:   *comment,
		}
		
		err := tracker.addEntry(entry)
		if err != nil {
			fmt.Printf("Error adding task: %v\n", err)
			os.Exit(1)
		}
		
		// Calculate and show duration
		var durationMsg string
		if len(tracker.entries) > 1 {
			lastEntry := tracker.entries[len(tracker.entries)-2]
			duration := entry.Timestamp.Sub(lastEntry.Timestamp)
			durationMsg = fmt.Sprintf(" (%s)", formatDuration(duration))
		}
		
		fmt.Printf("✅ Task completed: %s%s\n", *addTask, durationMsg)
		return
	}

	if *extend {
		err := tracker.extend()
		if err != nil {
			fmt.Printf("Error extending task: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("✅ Task extended to current time!")
		return
	}

	if *showReport {
		printTodaysReport(tracker)
		return
	}

	// If no CLI flags, start TUI
	p := tea.NewProgram(initialModel(), tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		log.Fatal(err)
	}
}
