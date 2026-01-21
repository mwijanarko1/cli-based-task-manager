use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

/// Comprehensive task model for enterprise use
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Task {
    /// Unique identifier for the task
    pub id: Uuid,

    /// Task title - required, max 200 characters
    #[validate(length(min = 1, max = 200, message = "Title must be between 1-200 characters"))]
    pub title: String,

    /// Optional detailed description
    #[validate(length(max = 2000, message = "Description must not exceed 2000 characters"))]
    pub description: Option<String>,

    /// Task priority level
    pub priority: Priority,

    /// Current status
    pub status: TaskStatus,

    /// Optional category/tag for organization
    #[validate(length(max = 50, message = "Category must not exceed 50 characters"))]
    pub category: Option<String>,

    /// Optional due date
    pub due_date: Option<DateTime<Utc>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// Optional completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with default values
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            priority: Priority::Medium,
            status: TaskStatus::Todo,
            category: None,
            due_date: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Create a task with all fields specified
    pub fn with_details(
        title: String,
        description: Option<String>,
        priority: Priority,
        category: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            priority,
            status: TaskStatus::Todo,
            category,
            due_date,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Done;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark task as in progress
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.updated_at = Utc::now();
    }

    /// Mark task as cancelled
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Update task details
    pub fn update(&mut self, title: Option<String>, description: Option<Option<String>>, priority: Option<Priority>, category: Option<Option<String>>, due_date: Option<Option<DateTime<Utc>>>) {
        if let Some(title) = title {
            self.title = title;
        }
        if let Some(description) = description {
            self.description = description;
        }
        if let Some(priority) = priority {
            self.priority = priority;
        }
        if let Some(category) = category {
            self.category = category;
        }
        if let Some(due_date) = due_date {
            self.due_date = due_date;
        }
        self.updated_at = Utc::now();
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            self.status != TaskStatus::Done && Utc::now() > due_date
        } else {
            false
        }
    }

    /// Get formatted status string
    pub fn status_display(&self) -> &'static str {
        match self.status {
            TaskStatus::Todo => "📋 TODO",
            TaskStatus::InProgress => "🔄 IN PROGRESS",
            TaskStatus::Done => "✅ DONE",
            TaskStatus::Cancelled => "❌ CANCELLED",
        }
    }

    /// Get formatted priority string
    pub fn priority_display(&self) -> &'static str {
        match self.priority {
            Priority::Low => "🟢 LOW",
            Priority::Medium => "🟡 MEDIUM",
            Priority::High => "🟠 HIGH",
            Priority::Critical => "🔴 CRITICAL",
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new(String::new())
    }
}

/// Parse a datetime string in ISO 8601 format
pub fn parse_datetime(date_str: &str) -> crate::error::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| crate::error::TaskError::DateParseError(
            format!("Invalid date format: {}. Use ISO 8601 format like '2024-01-01T12:00:00Z'", date_str)
        ))
}