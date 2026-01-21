use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use std::path::PathBuf;

/// Enterprise Task Manager CLI
#[derive(Parser)]
#[command(name = "task-manager")]
#[command(version, about = "Enterprise-grade task management CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Data file path
    #[arg(short = 'f', long, value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task title (required)
        title: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// Task priority
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: PriorityArg,

        /// Task category
        #[arg(short, long)]
        category: Option<String>,

        /// Due date (ISO 8601 format: 2024-01-01T12:00:00Z)
        #[arg(long)]
        due_date: Option<String>,
    },

    /// List tasks with optional filtering
    List {
        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<StatusArg>,

        /// Filter by priority
        #[arg(short = 'P', long, value_enum)]
        priority: Option<PriorityArg>,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Show only overdue tasks
        #[arg(long)]
        overdue: bool,

        /// Sort tasks by criteria
        #[arg(short = 'S', long, value_enum, default_value = "created-desc")]
        sort: SortArg,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Search query (searches title and description)
        #[arg(short = 'q', long)]
        search: Option<String>,
    },

    /// Show detailed information about a specific task
    Show {
        /// Task ID
        id: String,
    },

    /// Update an existing task
    Update {
        /// Task ID
        id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description (use empty string to clear)
        #[arg(short, long)]
        description: Option<String>,

        /// New priority
        #[arg(short, long, value_enum)]
        priority: Option<PriorityArg>,

        /// New category (use empty string to clear)
        #[arg(short, long)]
        category: Option<String>,

        /// New due date (ISO 8601 format, use empty string to clear)
        #[arg(long)]
        due_date: Option<String>,
    },

    /// Mark a task as complete
    Complete {
        /// Task ID
        id: String,
    },

    /// Start working on a task
    Start {
        /// Task ID
        id: String,
    },

    /// Cancel a task
    Cancel {
        /// Task ID
        id: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Show task statistics
    Stats,

    /// Clear completed tasks
    Clear {
        /// Clear all tasks (not just completed ones)
        #[arg(long)]
        all: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Import tasks from JSON file
    Import {
        /// Path to JSON file to import
        file: PathBuf,
    },

    /// Export tasks to JSON file
    Export {
        /// Path to JSON file to export to
        file: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, ValueEnum)]
pub enum StatusArg {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Clone, ValueEnum)]
pub enum SortArg {
    CreatedAsc,
    CreatedDesc,
    DueDateAsc,
    DueDateDesc,
    PriorityAsc,
    PriorityDesc,
    TitleAsc,
    TitleDesc,
}

impl From<PriorityArg> for crate::task::Priority {
    fn from(arg: PriorityArg) -> Self {
        match arg {
            PriorityArg::Low => crate::task::Priority::Low,
            PriorityArg::Medium => crate::task::Priority::Medium,
            PriorityArg::High => crate::task::Priority::High,
            PriorityArg::Critical => crate::task::Priority::Critical,
        }
    }
}

impl From<StatusArg> for crate::task::TaskStatus {
    fn from(arg: StatusArg) -> Self {
        match arg {
            StatusArg::Todo => crate::task::TaskStatus::Todo,
            StatusArg::InProgress => crate::task::TaskStatus::InProgress,
            StatusArg::Done => crate::task::TaskStatus::Done,
            StatusArg::Cancelled => crate::task::TaskStatus::Cancelled,
        }
    }
}

impl From<SortArg> for crate::manager::TaskSort {
    fn from(arg: SortArg) -> Self {
        match arg {
            SortArg::CreatedAsc => crate::manager::TaskSort::CreatedAsc,
            SortArg::CreatedDesc => crate::manager::TaskSort::CreatedDesc,
            SortArg::DueDateAsc => crate::manager::TaskSort::DueDateAsc,
            SortArg::DueDateDesc => crate::manager::TaskSort::DueDateDesc,
            SortArg::PriorityAsc => crate::manager::TaskSort::PriorityAsc,
            SortArg::PriorityDesc => crate::manager::TaskSort::PriorityDesc,
            SortArg::TitleAsc => crate::manager::TaskSort::TitleAsc,
            SortArg::TitleDesc => crate::manager::TaskSort::TitleDesc,
        }
    }
}