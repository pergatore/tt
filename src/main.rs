use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod entry;
mod report;
mod storage;
mod util;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the data file (defaults to ~/.local/share/tt/entries.log)
    #[arg(long)]
    data: Option<String>,

    /// Override current time (format: "YYYY-MM-DD HH:MM")
    #[arg(long)]
    now: Option<String>,

    /// Use specific timezone
    #[arg(long)]
    timezone: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Say 'hello' when you arrive in the morning
    Hello,
    
    /// Add a completed task
    Add {
        /// Completed task description
        name: String,
        
        /// Optional comment for the task
        #[arg(short, long)]
        comment: Option<String>,
    },
    
    /// Edit task log using your system's default editor
    Edit,
    
    /// Summarize tasks for given time period
    Report {
        /// Date to report (YYYY-MM-DD or day of week)
        date: Option<String>,
        
        /// Set the current activity name
        #[arg(long, default_value = "-- Current Activity --")]
        current_activity: String,
        
        /// Do not display the current activity
        #[arg(long)]
        no_current_activity: bool,
        
        /// Specify an inclusive start date
        #[arg(long)]
        from: Option<String>,
        
        /// Specify an inclusive end date
        #[arg(long)]
        to: Option<String>,
        
        /// Show activities only for the specified project
        #[arg(long)]
        project: Option<String>,
        
        /// Show total hours per day
        #[arg(long)]
        per_day: bool,
        
        /// Output a CSV report instead of text
        #[arg(long)]
        csv_section: Option<String>,
        
        /// Specify a month (YYYY-MM, month name, 'this', 'prev')
        #[arg(long)]
        month: Option<String>,
        
        /// Specify a week ('this', 'prev', or week number)
        #[arg(long)]
        week: Option<String>,
        
        /// Show details even for multi-day reports
        #[arg(long)]
        details: bool,
        
        /// Show comments in details sections
        #[arg(long)]
        comments: bool,
    },
    
    /// Stretch the latest task to the current time
    Stretch,
    
    /// Show or modify configuration
    Config {
        /// Show default configuration
        #[arg(long)]
        default: bool,
        
        /// Show configuration filename
        #[arg(long)]
        filename: bool,
    },
}

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize configuration
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            process::exit(1);
        }
    };
    
    // Execute the appropriate command
    let result = match &cli.command {
        Commands::Hello => {
            commands::hello::execute(&cli, &config)
        },
        Commands::Add { name, comment } => {
            commands::add::execute(&cli, &config, name, comment.as_deref())
        },
        Commands::Edit => {
            commands::edit::execute(&cli, &config)
        },
        Commands::Report { 
            date, 
            current_activity, 
            no_current_activity, 
            from, 
            to, 
            project, 
            per_day, 
            csv_section, 
            month, 
            week, 
            details, 
            comments 
        } => {
            commands::report::execute(
                &cli,
                &config,
                date.as_deref(),
                current_activity,
                *no_current_activity,
                from.as_deref(),
                to.as_deref(),
                project.as_deref(),
                *per_day,
                csv_section.as_deref(),
                month.as_deref(),
                week.as_deref(),
                *details,
                *comments
            )
        },
        Commands::Stretch => {
            commands::stretch::execute(&cli, &config)
        },
        Commands::Config { default, filename } => {
            commands::config::execute(&cli, &config, *default, *filename)
        },
    };
    
    // Handle any errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
