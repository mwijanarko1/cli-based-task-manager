use crate::error::{Result, TaskError};
use crate::task::{Priority, Task, TaskStatus, UpdateValue};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
///
/// TaskManager provides a comprehensive interface for managing tasks with
/// persistence to JSON files, validation, search, filtering, and statistics.
/// It maintains an in-memory cache of tasks indexed by UUID for fast lookup.
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskManager {
    /// All tasks indexed by ID for fast lookup
    pub tasks: HashMap<String, Task>,

    /// Configuration
    #[serde(skip)]
    pub config: TaskManagerConfig,

    /// Track if data has been modified since last save
    #[serde(skip)]
    pub dirty: AtomicBool,
}

impl TaskManager {
    /// Create a new task manager with default configuration.
    ///
    /// The default configuration uses "tasks.json" as storage and enables auto-save.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_config(TaskManagerConfig::default())
    }

    /// Create a new task manager with custom configuration.
    pub fn with_config(config: TaskManagerConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            config,
            dirty: AtomicBool::new(false),
        }
    }

    /// Load tasks from the configured storage path asynchronously.
    ///
    /// If the file does not exist, it starts with an empty task list.
    /// Clears any existing tasks in memory.
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

        self.dirty.store(false, Ordering::Relaxed);
        info!("Loaded {} tasks from {}", self.tasks.len(), self.config.storage_path.display());
        Ok(())
    }

    /// Save all tasks to the configured storage path asynchronously.
    ///
    /// Only performs a save if the `dirty` flag is set to true.
    pub async fn save(&self) -> Result<()> {
        if !self.dirty.load(Ordering::Relaxed) {
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

    /// Add a new task with basic info and perform validation.
    ///
    /// Returns the ID of the newly created task.
    #[allow(dead_code)]
    pub fn add_task(&mut self, title: String) -> Result<String> {
        let task = Task::new(title);
        task.validate().map_err(TaskError::from_validation_errors)?;

        let id = task.id.to_string();
        self.tasks.insert(id.clone(), task);
        self.dirty.store(true, Ordering::Relaxed);

        info!("Added task: {}", id);
        Ok(id)
    }

    /// Add a new task with full details and perform validation.
    ///
    /// Returns the ID of the newly created task.
    pub fn add_task_detailed(
        &mut self,
        title: String,
        description: Option<String>,
        priority: Option<Priority>,
        category: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let task = Task::with_details(
            title,
            description,
            priority.unwrap_or(Priority::Medium),
            category,
            due_date,
        );

        task.validate().map_err(TaskError::from_validation_errors)?;

        let id = task.id.to_string();
        self.tasks.insert(id.clone(), task);
        self.dirty.store(true, Ordering::Relaxed);

        info!("Added detailed task: {}", id);
        Ok(id)
    }

    /// Retrieve a task by its ID.
    ///
    /// Returns `TaskError::TaskNotFound` if the task doesn't exist.
    pub fn get_task(&self, id: &str) -> Result<&Task> {
        self.tasks.get(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))
    }

    /// Retrieve a mutable reference to a task by its ID.
    ///
    /// Returns `TaskError::TaskNotFound` if the task doesn't exist.
    pub fn get_task_mut(&mut self, id: &str) -> Result<&mut Task> {
        self.tasks.get_mut(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))
    }

    /// Update an existing task's fields.
    ///
    /// Re-validates the task after update and sets the dirty flag.
    pub fn update_task(
        &mut self,
        id: &str,
        title: Option<String>,
        description: UpdateValue<String>,
        priority: Option<Priority>,
        category: UpdateValue<String>,
        due_date: UpdateValue<DateTime<Utc>>,
    ) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.update(title, description, priority, category, due_date);
        task.validate().map_err(TaskError::from_validation_errors)?;
        self.dirty.store(true, Ordering::Relaxed);

        info!("Updated task: {}", id);
        Ok(())
    }

    /// Delete a task by its ID and return it.
    ///
    /// Returns `TaskError::TaskNotFound` if the task doesn't exist.
    pub fn delete_task(&mut self, id: &str) -> Result<Task> {
        let task = self.tasks.remove(id).ok_or_else(|| TaskError::TaskNotFound(id.to_string()))?;
        self.dirty.store(true, Ordering::Relaxed);

        info!("Deleted task: {}", id);
        Ok(task)
    }

    /// Mark a task as complete.
    ///
    /// Returns an error if the task is already completed.
    pub fn complete_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        if task.status == TaskStatus::Done {
            return Err(TaskError::OperationNotAllowed("Task is already completed".to_string()));
        }
        task.complete();
        self.dirty.store(true, Ordering::Relaxed);

        info!("Completed task: {}", id);
        Ok(())
    }

    /// Move a task to the InProgress status.
    pub fn start_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.start();
        self.dirty.store(true, Ordering::Relaxed);

        info!("Started task: {}", id);
        Ok(())
    }

    /// Move a task to the Cancelled status.
    pub fn cancel_task(&mut self, id: &str) -> Result<()> {
        let task = self.get_task_mut(id)?;
        task.cancel();
        self.dirty.store(true, Ordering::Relaxed);

        info!("Cancelled task: {}", id);
        Ok(())
    }

    /// Get all tasks (immutable view)
    pub fn get_all_tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values()
    }

    /// Get tasks filtered by status
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |task| task.status == status)
    }

    /// Get tasks filtered by priority
    pub fn get_tasks_by_priority(&self, priority: Priority) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |task| task.priority == priority)
    }

    /// Get tasks filtered by category
    pub fn get_tasks_by_category<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a Task> {
        self.tasks.values()
            .filter(move |task| task.category.as_ref().map_or(false, |c| c == category))
    }

    /// Get overdue tasks
    pub fn get_overdue_tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(|task| task.is_overdue())
    }

    /// Search tasks by title or description
    pub fn search_tasks<'a>(&'a self, query: &'a str) -> impl Iterator<Item = &'a Task> {
        let query_lower = query.to_lowercase();
        self.tasks.values()
            .filter(move |task| {
                // Optimization: Case-insensitive contains without repeated to_lowercase()
                // using the query_lower which is only computed once.
                // Rust's contains is case-sensitive, so we still need to lowercase the target strings.
                // However, we can avoid allocating if we use a better approach, but for now 
                // lowercase the target and compare with query_lower.
                task.title.to_lowercase().contains(&query_lower) ||
                task.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            })
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
        let overdue = self.get_overdue_tasks().count();

        TaskStats {
            total,
            completed,
            in_progress,
            overdue,
            completion_rate: if total > 0 { (completed as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }

    /// Clear all completed tasks from memory and set the dirty flag.
    ///
    /// Returns the number of tasks removed.
    pub fn clear_completed(&mut self) -> usize {
        let initial_count = self.tasks.len();
        self.tasks.retain(|_, task| task.status != TaskStatus::Done);
        let removed = initial_count - self.tasks.len();
        self.dirty.store(true, Ordering::Relaxed);

        info!("Cleared {} completed tasks", removed);
        removed
    }

    /// Clear all tasks from memory and set the dirty flag.
    ///
    /// Returns the number of tasks removed.
    pub fn clear_all(&mut self) -> usize {
        let count = self.tasks.len();
        self.tasks.clear();
        self.dirty.store(true, Ordering::Relaxed);

        info!("Cleared all {} tasks", count);
        count
    }

    /// Import tasks from a list, skipping any that have IDs already present in memory.
    ///
    /// All imported tasks are re-validated before insertion.
    pub fn import_tasks(&mut self, tasks: Vec<Task>) -> Result<usize> {
        let mut imported_count = 0;
        for task in tasks {
            // Validate the task
            task.validate().map_err(TaskError::from_validation_errors)?;

            // Skip if task with this ID already exists
            if !self.tasks.contains_key(&task.id.to_string()) {
                self.tasks.insert(task.id.to_string(), task);
                imported_count += 1;
            }
        }

        if imported_count > 0 {
            self.dirty.store(true, Ordering::Relaxed);
        }

        info!("Imported {} tasks", imported_count);
        Ok(imported_count)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskStatus;

    #[test]
    fn test_task_manager_creation() {
        let manager = TaskManager::new();
        assert!(manager.tasks.is_empty());
        assert!(!manager.dirty.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_add_and_retrieve_task() {
        let mut manager = TaskManager::new();
        let id = manager.add_task("Test Task".to_string()).unwrap();

        let task = manager.get_task(&id).unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn test_task_completion() {
        let mut manager = TaskManager::new();
        let id = manager.add_task("Test Task".to_string()).unwrap();

        manager.complete_task(&id).unwrap();
        let task = manager.get_task(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_search_tasks() {
        let mut manager = TaskManager::new();
        manager.add_task("Buy groceries".to_string()).unwrap();
        manager.add_task("Clean house".to_string()).unwrap();
        manager.add_task("Write code".to_string()).unwrap();

        let results: Vec<_> = manager.search_tasks("house").collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Clean house");
    }

    #[test]
    fn test_task_statistics() {
        let mut manager = TaskManager::new();
        manager.add_task("Task 1".to_string()).unwrap();
        let id2 = manager.add_task("Task 2".to_string()).unwrap();

        manager.complete_task(&id2).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.completion_rate, 50.0);
    }
}