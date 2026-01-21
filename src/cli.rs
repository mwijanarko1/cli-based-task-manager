use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Enterprise Task Manager CLI
#[derive(Parser)]
#[command(name = "task-manager")]
#[command(version, about = "Enterprise-grade task management CLI tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,


    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Data file path
    #[arg(short = 'f', long, value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task to the system
    Add {
        /// Task title (required, max 200 chars)
        title: String,

        /// Optional detailed task description
        #[arg(short, long)]
        description: Option<String>,

        /// Task priority (low, medium, high, critical)
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: PriorityArg,

        /// Task category for organization
        #[arg(short, long)]
        category: Option<String>,

        /// Due date in ISO 8601 format (e.g., 2024-01-01T12:00:00Z)
        #[arg(long)]
        due_date: Option<String>,
    },

    /// List tasks with comprehensive filtering and sorting options
    List {
        /// Filter by task status (todo, in-progress, done, cancelled)
        #[arg(short, long, value_enum)]
        status: Option<StatusArg>,

        /// Filter by task priority (low, medium, high, critical)
        #[arg(short = 'P', long, value_enum)]
        priority: Option<PriorityArg>,

        /// Filter by exact category name
        #[arg(short, long)]
        category: Option<String>,

        /// Show only tasks that are overdue
        #[arg(long)]
        overdue: bool,

        /// Sort tasks by specific criteria
        #[arg(short = 'S', long, value_enum, default_value = "created-desc")]
        sort: SortArg,

        /// Limit the number of results displayed
        #[arg(short, long)]
        limit: Option<usize>,

        /// Search query that matches against title and description
        #[arg(short = 'q', long)]
        search: Option<String>,
    },

    /// Show detailed information about a specific task including all metadata
    Show {
        /// Full task UUID
        id: String,
    },

    /// Update an existing task's fields
    Update {
        /// Full task UUID
        id: String,

        /// Update the task title
        #[arg(short, long)]
        title: Option<String>,

        /// Update description (use empty string "" to clear)
        #[arg(short, long)]
        description: Option<String>,

        /// Update priority level
        #[arg(short, long, value_enum)]
        priority: Option<PriorityArg>,

        /// Update category (use empty string "" to clear)
        #[arg(short, long)]
        category: Option<String>,

        /// Update due date in ISO 8601 format (use empty string "" to clear)
        #[arg(long)]
        due_date: Option<String>,
    },

    /// Mark a task as completed (Done status)
    Complete {
        /// Task UUID (optional - triggers interactive selection if omitted)
        id: Option<String>,
    },

    /// Start working on a task (InProgress status)
    Start {
        /// Task UUID (optional - triggers interactive selection if omitted)
        id: Option<String>,
    },

    /// Cancel a task (Cancelled status)
    Cancel {
        /// Task UUID (optional - triggers interactive selection if omitted)
        id: Option<String>,
    },

    /// Delete a task permanently from the system
    Delete {
        /// Task UUID (optional - triggers interactive selection if omitted)
        id: Option<String>,

        /// Skip the interactive confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Bulk operation to delete ALL tasks in the system
    DeleteAll {
        /// Skip the interactive confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Display aggregate statistics about all tasks
    Stats,

    /// Clear tasks based on their completion status
    Clear {
        /// If set, clears all tasks regardless of status
        #[arg(long)]
        all: bool,

        /// Skip the interactive confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Bulk import tasks from a JSON file
    Import {
        /// Path to the JSON file to import from
        file: PathBuf,
    },

    /// Bulk export all tasks to a JSON file
    Export {
        /// Path where the JSON file will be created
        file: PathBuf,
    },
}

/// CLI argument variant for Priority
#[derive(Clone, ValueEnum)]
pub enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

/// CLI argument variant for Status
#[derive(Clone, ValueEnum)]
pub enum StatusArg {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

/// CLI argument variant for Sorting
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