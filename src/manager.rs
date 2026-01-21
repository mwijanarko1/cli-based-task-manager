use crate::error::{Result, TaskError};
use crate::task::{Priority, Task, TaskStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use validator::Validate;

/// Configuration for task storage
#[derive(Debug, Clone)]
pub struct TaskManagerConfig {
    pub storage_path: PathBuf,
    pub auto_save: bool,
}

impl Default for TaskManagerConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("tasks.json"),
            auto_save: true,
        }
    }
}

/// Enterprise-grade task manager with persistence and comprehensive operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskManager {
    /// All tasks indexed by ID for fast lookup
    pub tasks: HashMap<String, Task>,

    /// Configuration
    #[serde(skip)]
    pub config: TaskManagerConfig,

    /// Track if data has been modified since last save
    #[serde(skip)]
    pub dirty: bool,
}

impl TaskManager {
    /// Create a new task manager with default configuration
    pub fn new() -> Self {
        Self::with_config(TaskManagerConfig::default())
    }

    /// Create a new task manager with custom configuration
    pub fn with_config(config: TaskManagerConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            config,
            dirty: false,
        }
    }

    /// Load tasks from file asynchronously
    pub async fn load(&mut self) -> Result<()> {
        if !self.config.storage_path.exists() {
            info!("No existing task file found, starting with empty task list");
            return Ok(());
        }

        let data = fs::read_to_string(&self.config.storage_path).await?;
        let loaded_tasks: Vec<Task> = serde_json::from_str(&data)?;

        self.tasks.clear();
        for task in loaded_tasks {
            self.tasks.insert(task.id.to_string(), task);
        }

        self.dirty = false;
        info!("Loaded {} tasks from {}", self.tasks.len(), self.config.storage_path.display());
        Ok(())
    }

    /// Save tasks to file asynchronously
    pub async fn save(&self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let tasks: Vec<&Task> = self.tasks.values().collect();
        let data = serde_json::to_string_pretty(&tasks)?;

        // Create directory if it doesn't exist
        if let Some(parent) = self.config.storage_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&self.config.storage_path, data).await?;
        info!("Saved {} tasks to {}", tasks.len(), self.config.storage_path.display());
        Ok(())
    }

    /// Add a new task with validation
    pub fn add_task(&mut self, title: String) -> Result<String> {
        let mut task = Task::new(title);
        task.validate().map_err(TaskError::from_validation_errors)?;

        let id = task.id.to_string();
        self.tasks.insert(id.clone(), task);
        self.dirty = true;

        info!("Added task: {}", id);
        Ok(id)
    }

    /// Add a task with full details
    pub fn add_task_detailed(
        &mut self,
        title: String,
        description: Option<String>,
        priority: Option<Priority>,
        category: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let mut task = Task::with_details(
            title,
            description,
            priority.unwrap_or(Priority::Medium),
            category,
            due_date,
        );

        task.validate().map_err(TaskError::from_validation_errors)?;

        let id = task.id.to_string();
        self.tasks.insert(id.clone(), task);
        self.dirty = true;

        info!("Added detailed task: {}", id);
        Ok(id)
    }

    /// Get a task by ID (immutable borrow)
    pub fn get_task(&self, id: &str) -> Result<&Task> {
        self.tasks.get(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))
    }

    /// Get a mutable reference to a task
    pub fn get_task_mut(&mut self, id: &str) -> Result<&mut Task> {
        self.tasks.get_mut(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))
    }

    /// Update a task
    pub fn update_task(
        &mut self,
        id: &str,
        title: Option<String>,
        description: Option<Option<String>>,
        priority: Option<Priority>,
        category: Option<Option<String>>,
        due_date: Option<Option<DateTime<Utc>>>,
    ) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.update(title, description, priority, category, due_date);
        task.validate().map_err(TaskError::from_validation_errors)?;
        self.dirty = true;

        info!("Updated task: {}", id);
        Ok(())
    }

    /// Delete a task
    pub fn delete_task(&mut self, id: &str) -> Result<Task> {
        let task = self.tasks.remove(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))?;
        self.dirty = true;

        info!("Deleted task: {}", id);
        Ok(task)
    }

    /// Mark task as complete
    pub fn complete_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        if task.status == TaskStatus::Done {
            return Err(TaskError::OperationNotAllowed("Task is already completed".to_string()));
        }
        task.complete();
        self.dirty = true;

        info!("Completed task: {}", id);
        Ok(())
    }

    /// Start working on a task
    pub fn start_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.start();
        self.dirty = true;

        info!("Started task: {}", id);
        Ok(())
    }

    /// Cancel a task
    pub fn cancel_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.cancel();
        self.dirty = true;

        info!("Cancelled task: {}", id);
        Ok(())
    }

    /// Get all tasks (immutable view)
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// Get tasks filtered by status
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks.values().filter(|task| task.status == status).collect()
    }

    /// Get tasks filtered by priority
    pub fn get_tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.tasks.values().filter(|task| task.priority == priority).collect()
    }

    /// Get tasks filtered by category
    pub fn get_tasks_by_category(&self, category: &str) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| task.category.as_ref().map_or(false, |c| c == category))
            .collect()
    }

    /// Get overdue tasks
    pub fn get_overdue_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|task| task.is_overdue()).collect()
    }

    /// Search tasks by title or description
    pub fn search_tasks(&self, query: &str) -> Vec<&Task> {
        let query = query.to_lowercase();
        self.tasks.values()
            .filter(|task| {
                task.title.to_lowercase().contains(&query) ||
                task.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query))
            })
            .collect()
    }

    /// Get tasks sorted by different criteria
    pub fn get_sorted_tasks(&self, sort_by: TaskSort) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.values().collect();

        match sort_by {
            TaskSort::CreatedAsc => tasks.sort_by_key(|t| t.created_at),
            TaskSort::CreatedDesc => tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            TaskSort::DueDateAsc => tasks.sort_by_key(|t| t.due_date),
            TaskSort::DueDateDesc => tasks.sort_by(|a, b| {
                match (a.due_date, b.due_date) {
                    (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            }),
            TaskSort::PriorityAsc => tasks.sort_by_key(|t| t.priority),
            TaskSort::PriorityDesc => tasks.sort_by(|a, b| b.priority.cmp(&a.priority)),
            TaskSort::TitleAsc => tasks.sort_by(|a, b| a.title.cmp(&b.title)),
            TaskSort::TitleDesc => tasks.sort_by(|a, b| b.title.cmp(&a.title)),
        }

        tasks
    }

    /// Get statistics about tasks
    pub fn get_stats(&self) -> TaskStats {
        let total = self.tasks.len();
        let completed = self.tasks.values().filter(|t| t.status == TaskStatus::Done).count();
        let in_progress = self.tasks.values().filter(|t| t.status == TaskStatus::InProgress).count();
        let overdue = self.get_overdue_tasks().len();

        TaskStats {
            total,
            completed,
            in_progress,
            overdue,
            completion_rate: if total > 0 { (completed as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }

    /// Clear all completed tasks
    pub fn clear_completed(&mut self) -> usize {
        let initial_count = self.tasks.len();
        self.tasks.retain(|_, task| task.status != TaskStatus::Done);
        let removed = initial_count - self.tasks.len();
        self.dirty = true;

        info!("Cleared {} completed tasks", removed);
        removed
    }

    /// Clear all tasks
    pub fn clear_all(&mut self) -> usize {
        let count = self.tasks.len();
        self.tasks.clear();
        self.dirty = true;

        info!("Cleared all {} tasks", count);
        count
    }
}

/// Sorting options for tasks
#[derive(Debug, Clone, Copy)]
pub enum TaskSort {
    CreatedAsc,
    CreatedDesc,
    DueDateAsc,
    DueDateDesc,
    PriorityAsc,
    PriorityDesc,
    TitleAsc,
    TitleDesc,
}

/// Statistics about tasks
#[derive(Debug, Clone)]
pub struct TaskStats {
    pub total: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub overdue: usize,
    pub completion_rate: f64,
}