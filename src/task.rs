use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Enum for update operations that distinguishes between keeping, clearing, or setting a value
#[derive(Debug, Clone)]
pub enum UpdateValue<T> {
    /// Keep the current value unchanged
    Keep,
    /// Clear the value (set to None)
    Clear,
    /// Set to a new value
    Set(T),
}

/// Priority levels for tasks.
///
/// Implements `PartialOrd` and `Ord` where Critical > High > Medium > Low.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Status of a task representing its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task has been created but not started
    Todo,
    /// Task is currently being worked on
    InProgress,
    /// Task has been finished successfully
    Done,
    /// Task has been cancelled and will not be completed
    Cancelled,
}

/// Comprehensive task model for enterprise use
///
/// A Task represents a single work item with all necessary metadata
/// for enterprise task management including validation, serialization,
/// and status tracking.
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
    /// Create a new task with default values and a random UUID.
    ///
    /// Sets priority to Medium and status to Todo.
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

    /// Create a task with all fields specified and a random UUID.
    ///
    /// Sets status to Todo by default.
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

    /// Mark task as completed, setting status to Done and record completion time.
    pub fn complete(&mut self) {
        self.status = TaskStatus::Done;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark task as in progress, setting status to InProgress.
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.updated_at = Utc::now();
    }

    /// Mark task as cancelled, setting status to Cancelled.
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Update task details selectively based on the provided options.
    ///
    /// Uses `UpdateValue` to determine whether to keep, clear, or set new values
    /// for description, category, and due date.
    pub fn update(&mut self, title: Option<String>, description: UpdateValue<String>, priority: Option<Priority>, category: UpdateValue<String>, due_date: UpdateValue<DateTime<Utc>>) {
        if let Some(title) = title {
            self.title = title;
        }
        match description {
            UpdateValue::Set(desc) => self.description = Some(desc),
            UpdateValue::Clear => self.description = None,
            UpdateValue::Keep => {} // Keep current value
        }
        if let Some(priority) = priority {
            self.priority = priority;
        }
        match category {
            UpdateValue::Set(cat) => self.category = Some(cat),
            UpdateValue::Clear => self.category = None,
            UpdateValue::Keep => {} // Keep current value
        }
        match due_date {
            UpdateValue::Set(date) => self.due_date = Some(date),
            UpdateValue::Clear => self.due_date = None,
            UpdateValue::Keep => {} // Keep current value
        }
        self.updated_at = Utc::now();
    }

    /// Returns true if the task is not completed and its due date has passed.
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            self.status != TaskStatus::Done && Utc::now() > due_date
        } else {
            false
        }
    }

    /// Get formatted status string with emoji for CLI display.
    pub fn status_display(&self) -> &'static str {
        match self.status {
            TaskStatus::Todo => "📋 TODO",
            TaskStatus::InProgress => "🔄 IN PROGRESS",
            TaskStatus::Done => "✅ DONE",
            TaskStatus::Cancelled => "❌ CANCELLED",
        }
    }

    /// Get formatted priority string with emoji for CLI display.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task".to_string());
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Medium);
        assert!(task.description.is_none());
        assert!(task.category.is_none());
        assert!(task.due_date.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_completion() {
        let mut task = Task::new("Test Task".to_string());
        let before_complete = task.updated_at;

        task.complete();

        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());
        assert!(task.updated_at >= before_complete);
    }

    #[test]
    fn test_task_update_with_enum() {
        let mut task = Task::new("Original".to_string());
        task.description = Some("Original desc".to_string());
        task.category = Some("Work".to_string());

        task.update(
            Some("Updated".to_string()),
            UpdateValue::Set("New desc".to_string()),
            Some(Priority::High),
            UpdateValue::Clear,
            UpdateValue::Keep,
        );

        assert_eq!(task.title, "Updated");
        assert_eq!(task.description, Some("New desc".to_string()));
        assert_eq!(task.priority, Priority::High);
        assert!(task.category.is_none()); // Cleared
    }

    #[test]
    fn test_task_is_overdue() {
        let past_date = Utc::now() - chrono::Duration::hours(1);
        let future_date = Utc::now() + chrono::Duration::hours(1);

        let overdue_task = Task::with_details(
            "Overdue".to_string(),
            None,
            Priority::High,
            None,
            Some(past_date),
        );

        let upcoming_task = Task::with_details(
            "Upcoming".to_string(),
            None,
            Priority::High,
            None,
            Some(future_date),
        );

        assert!(overdue_task.is_overdue());
        assert!(!upcoming_task.is_overdue());
    }
}