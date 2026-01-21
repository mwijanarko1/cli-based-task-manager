mod cli;
mod error;
mod manager;
mod task;

use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use error::{Result, TaskError};
use manager::{TaskManager, TaskManagerConfig};
use std::io::{self, Write};
use std::path::PathBuf;

/// Maximum size for import files (10MB)
const MAX_IMPORT_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum length for user input strings
const MAX_INPUT_LENGTH: usize = 1000;

/// Display constants for formatting
const UUID_DISPLAY_LENGTH: usize = 8;
const TITLE_MAX_DISPLAY: usize = 40;

/// Sanitize and validate user input
fn sanitize_input(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.len() > MAX_INPUT_LENGTH {
        return Err(TaskError::ValidationError("Input too long".to_string()));
    }
    Ok(trimmed.to_string())
}
use tracing::{error, warn, Level};
use tracing_subscriber;

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    let level = if verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Create task manager with configuration
    let config = TaskManagerConfig {
        storage_path: cli.file.unwrap_or_else(|| PathBuf::from("tasks.json")),
        auto_save: true,
    };

    let mut manager = TaskManager::with_config(config);

    // Load existing tasks
    if let Err(e) = manager.load().await {
        warn!("Failed to load tasks: {}", e);
        println!("{}", "Warning: Could not load existing tasks. Starting with empty list.".yellow());
    }

    // Execute command
    let result = match cli.command {
        Commands::Add { title, description, priority, category, due_date } => {
            handle_add(&mut manager, title, description, priority, category, due_date).await
        }
        Commands::List { status, priority, category, overdue, sort, limit, search } => {
            handle_list(&manager, status, priority, category, overdue, sort, limit, search).await
        }
        Commands::Show { id } => handle_show(&manager, &id).await,
        Commands::Update { id, title, description, priority, category, due_date } => {
            handle_update(&mut manager, &id, title, description, priority, category, due_date).await
        }
        Commands::Complete { id } => handle_complete(&mut manager, id).await,
        Commands::Start { id } => handle_start(&mut manager, id).await,
        Commands::Cancel { id } => handle_cancel(&mut manager, id).await,
        Commands::Delete { id, force } => handle_delete(&mut manager, id, force).await,
        Commands::DeleteAll { force } => handle_delete_all(&mut manager, force).await,
        Commands::Stats => handle_stats(&manager).await,
        Commands::Clear { all, force } => handle_clear(&mut manager, all, force).await,
        Commands::Import { file } => handle_import(&mut manager, file).await,
        Commands::Export { file } => handle_export(&manager, file).await,
    };

    // Auto-save if enabled and operation was successful
    if manager.config.auto_save {
        if let Err(e) = manager.save().await {
            error!("Failed to save tasks: {}", e);
            eprintln!("{}", format!("Error: Failed to save tasks: {}", e).red());
        }
    }

    result
}

/// Create a new task with the provided details
async fn handle_add(
    manager: &mut TaskManager,
    title: String,
    description: Option<String>,
    priority: cli::PriorityArg,
    category: Option<String>,
    due_date: Option<String>,
) -> Result<()> {
    let due_date_parsed = if let Some(date_str) = due_date {
        if date_str.is_empty() {
            None
        } else {
            Some(crate::task::parse_datetime(&date_str)?)
        }
    } else {
        None
    };

    let id = manager.add_task_detailed(
        title.clone(),
        description,
        Some(priority.into()),
        category,
        due_date_parsed,
    )?;

    println!("{}", format!("✓ Added task '{}' with ID: {}", title, id).green());
    Ok(())
}

/// List tasks filtered by the provided criteria and display them in a summary table
async fn handle_list(
    manager: &TaskManager,
    status: Option<cli::StatusArg>,
    priority: Option<cli::PriorityArg>,
    category: Option<String>,
    overdue: bool,
    sort: cli::SortArg,
    limit: Option<usize>,
    search: Option<String>,
) -> Result<()> {
    let query_str = search.as_deref();
    let category_str = category.as_deref();

    let mut tasks: Vec<_> = if let Some(query) = query_str {
        manager.search_tasks(query).collect()
    } else if overdue {
        manager.get_overdue_tasks().collect()
    } else if let Some(status) = status {
        manager.get_tasks_by_status(status.into()).collect()
    } else if let Some(priority) = priority {
        manager.get_tasks_by_priority(priority.into()).collect()
    } else if let Some(category) = category_str {
        manager.get_tasks_by_category(category).collect()
    } else {
        manager.get_sorted_tasks(sort.into())
    };

    if tasks.is_empty() {
        println!("{}", "No tasks found.".yellow());
        return Ok(());
    }

    // Apply limit if specified
    if let Some(limit) = limit {
        tasks.truncate(limit);
    }

    println!("{}", format!("📋 Tasks ({} found):", tasks.len()).cyan().bold());
    println!("{}", "─".repeat(80).dimmed());

    for task in tasks {
        print_task_summary(task);
    }

    Ok(())
}

/// Display detailed information about a single task, including all metadata and status
async fn handle_show(manager: &TaskManager, id: &str) -> Result<()> {
    let task = manager.get_task(id)?;

    println!("{}", format!("📄 Task Details: {}", task.id).cyan().bold());
    println!("{}", "─".repeat(40).dimmed());

    println!("{} {}", "Title:".bold(), task.title);
    println!("{} {}", "Status:".bold(), task.status_display());
    println!("{} {}", "Priority:".bold(), task.priority_display());
    println!("{} {}", "Created:".bold(), task.created_at.format("%Y-%m-%d %H:%M:%S UTC"));

    if let Some(ref desc) = task.description {
        println!("{} {}", "Description:".bold(), desc);
    }

    if let Some(ref category) = task.category {
        println!("{} {}", "Category:".bold(), category);
    }

    if let Some(due_date) = task.due_date {
        let due_str = due_date.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        if task.is_overdue() {
            println!("{} {} {}", "Due Date:".bold(), due_str.red(), "(OVERDUE)".red().bold());
        } else {
            println!("{} {}", "Due Date:".bold(), due_str);
        }
    }

    if let Some(completed_at) = task.completed_at {
        println!("{} {}", "Completed:".bold(), completed_at.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    Ok(())
}

/// Update an existing task's details selectively
async fn handle_update(
    manager: &mut TaskManager,
    id: &str,
    title: Option<String>,
    description: Option<String>,
    priority: Option<cli::PriorityArg>,
    category: Option<String>,
    due_date: Option<String>,
) -> Result<()> {
    use crate::task::UpdateValue;

    let description = match description {
        Some(d) if d.is_empty() => UpdateValue::Clear,
        Some(d) => UpdateValue::Set(d),
        None => UpdateValue::Keep,
    };
    let category = match category {
        Some(c) if c.is_empty() => UpdateValue::Clear,
        Some(c) => UpdateValue::Set(c),
        None => UpdateValue::Keep,
    };
    let priority = priority.map(|p| p.into());

    let due_date = match due_date {
        Some(d) if d.is_empty() => UpdateValue::Clear,
        Some(d) => UpdateValue::Set(crate::task::parse_datetime(&d)?),
        None => UpdateValue::Keep,
    };

    manager.update_task(id, title, description, priority, category, due_date)?;
    println!("{}", format!("✓ Updated task {}", id).green());
    Ok(())
}

/// Mark a task as completed, recording completion time
async fn handle_complete(manager: &mut TaskManager, id: Option<String>) -> Result<()> {
    let task_id = match id {
        Some(id) => id,
        None => select_task_interactive(manager).await?,
    };

    manager.complete_task(&task_id)?;
    println!("{}", format!("✓ Completed task {}", task_id).green());
    Ok(())
}

/// Mark a task as being worked on (In Progress)
async fn handle_start(manager: &mut TaskManager, id: Option<String>) -> Result<()> {
    let task_id = match id {
        Some(id) => id,
        None => select_task_interactive(manager).await?,
    };

    manager.start_task(&task_id)?;
    println!("{}", format!("▶ Started working on task {}", task_id).green());
    Ok(())
}

/// Mark a task as cancelled
async fn handle_cancel(manager: &mut TaskManager, id: Option<String>) -> Result<()> {
    let task_id = match id {
        Some(id) => id,
        None => select_task_interactive(manager).await?,
    };

    manager.cancel_task(&task_id)?;
    println!("{}", format!("❌ Cancelled task {}", task_id).yellow());
    Ok(())
}

/// Delete a task permanently, with a confirmation prompt unless forced
async fn handle_delete(manager: &mut TaskManager, id: Option<String>, force: bool) -> Result<()> {
    let task_id = match id {
        Some(id) => id,
        None => select_task_interactive(manager).await?,
    };

    if !force {
        print!("Are you sure you want to delete task {}? (y/N): ", task_id);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = sanitize_input(&input)?;
        if !input.eq_ignore_ascii_case("y") && !input.eq_ignore_ascii_case("yes") {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    manager.delete_task(&task_id)?;
    println!("{}", format!("🗑 Deleted task {}", task_id).red());
    Ok(())
}

/// Bulk delete operation for all tasks with a double-confirmation prompt
async fn handle_delete_all(manager: &mut TaskManager, force: bool) -> Result<()> {
    let count = manager.get_all_tasks().count();

    if count == 0 {
        println!("{}", "No tasks to delete.".yellow());
        return Ok(());
    }

    if !force {
        print!("Are you sure you want to delete ALL {} tasks? This action cannot be undone. (y/N): ", count);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = sanitize_input(&input)?;
        if !input.eq_ignore_ascii_case("y") && !input.eq_ignore_ascii_case("yes") {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    let removed = manager.clear_all();
    println!("{}", format!("🗑 Deleted all {} tasks", removed).red().bold());
    Ok(())
}

/// Display aggregate task statistics including completion rate and status counts
async fn handle_stats(manager: &TaskManager) -> Result<()> {
    let stats = manager.get_stats();

    println!("{}", "📊 Task Statistics".cyan().bold());
    println!("{}", "─".repeat(30).dimmed());

    println!("{} {}", "Total tasks:".bold(), stats.total);
    println!("{} {}", "Completed:".bold(), stats.completed);
    println!("{} {}", "In progress:".bold(), stats.in_progress);
    println!("{} {}", "Overdue:".bold(), stats.overdue);
    println!("{} {:.1}%", "Completion rate:".bold(), stats.completion_rate);

    Ok(())
}

/// Clear tasks based on status, supporting both completed-only and all tasks
async fn handle_clear(manager: &mut TaskManager, all: bool, force: bool) -> Result<()> {
    let count = if all { manager.get_all_tasks().count() } else {
        manager.get_tasks_by_status(crate::task::TaskStatus::Done).count()
    };

    if !force {
        let prompt = if all {
            format!("Are you sure you want to delete ALL {} tasks? (y/N): ", count)
        } else {
            format!("Are you sure you want to delete {} completed tasks? (y/N): ", count)
        };

        print!("{}", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = sanitize_input(&input)?;
        if !input.eq_ignore_ascii_case("y") && !input.eq_ignore_ascii_case("yes") {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    let removed = if all {
        manager.clear_all()
    } else {
        manager.clear_completed()
    };

    println!("{}", format!("🧹 Cleared {} tasks", removed).green());
    Ok(())
}

/// Import tasks from a JSON file with validation and duplicate skipping
async fn handle_import(manager: &mut TaskManager, file: PathBuf) -> Result<()> {
    // Canonicalize path to prevent directory traversal
    let file = file.canonicalize().map_err(|e| TaskError::FileOperationError(
        format!("Invalid file path: {}", e)
    ))?;

    // Check file size before reading
    let metadata: std::fs::Metadata = tokio::fs::metadata(&file).await?;
    if metadata.len() > MAX_IMPORT_SIZE {
        return Err(TaskError::FileOperationError(
            format!("File too large: {} bytes (max: {} bytes)", metadata.len(), MAX_IMPORT_SIZE)
        ));
    }

    // Read file data
    let data = tokio::fs::read(&file).await?;
    let imported_tasks: Vec<crate::task::Task> = serde_json::from_slice(&data)?;

    // Use the manager's import method for validation and safe insertion
    let imported_count = manager.import_tasks(imported_tasks)?;

    println!("{}", format!("📥 Imported {} tasks from {}", imported_count, file.display()).green());
    Ok(())
}

/// Export all tasks currently in memory to a JSON file
async fn handle_export(manager: &TaskManager, file: PathBuf) -> Result<()> {
    let tasks: Vec<&crate::task::Task> = manager.get_all_tasks().collect();
    let data = serde_json::to_string_pretty(&tasks)?;

    if let Some(parent) = file.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&file, data).await?;
    println!("{}", format!("📤 Exported {} tasks to {}", tasks.len(), file.display()).green());
    Ok(())
}

/// Interactively select a task from a numbered list of all available tasks
async fn select_task_interactive(manager: &TaskManager) -> Result<String> {
    let tasks = manager.get_sorted_tasks(crate::manager::TaskSort::CreatedDesc);

    if tasks.is_empty() {
        println!("{}", "No tasks available to select.".yellow());
        return Err(TaskError::ValidationError("No tasks available".to_string()));
    }

    println!("{}", "Select a task:".cyan().bold());
    println!("{}", "─".repeat(80).dimmed());

    for (i, task) in tasks.iter().enumerate() {
        print!("{}: ", format!("{:2}", i + 1).bold());
        print_task_summary(task);
    }

    println!("{}", "─".repeat(80).dimmed());
    print!("Enter task number (1-{}) or 'q' to cancel: ", tasks.len());

    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = sanitize_input(&input)?;

    if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
        println!("{}", "Selection cancelled.".yellow());
        return Err(TaskError::ValidationError("Selection cancelled".to_string()));
    }

    match input.parse::<usize>() {
        Ok(num) if num >= 1 && num <= tasks.len() => {
            let selected_task = &tasks[num - 1];
            Ok(selected_task.id.to_string())
        }
        _ => {
            println!("{}", "Invalid selection.".red());
            Err(TaskError::ValidationError("Invalid task selection".to_string()))
        }
    }
}

/// Print a summary of a task
fn print_task_summary(task: &crate::task::Task) {
    let status_icon = match task.status {
        crate::task::TaskStatus::Todo => "📋",
        crate::task::TaskStatus::InProgress => "🔄",
        crate::task::TaskStatus::Done => "✅",
        crate::task::TaskStatus::Cancelled => "❌",
    };

    let priority_color = match task.priority {
        crate::task::Priority::Low => "🟢",
        crate::task::Priority::Medium => "🟡",
        crate::task::Priority::High => "🟠",
        crate::task::Priority::Critical => "🔴",
    };

    let id = format!("{}...", task.id.to_string().get(..UUID_DISPLAY_LENGTH).unwrap_or(&task.id.to_string()));
    let title = if task.title.len() > TITLE_MAX_DISPLAY {
        format!("{}...", task.title.get(..TITLE_MAX_DISPLAY - 3).unwrap_or(&task.title))
    } else {
        task.title.clone()
    };

    print!("{} {} {} {}", status_icon, priority_color, id.dimmed(), title);

    if let Some(ref category) = task.category {
        print!(" {}", format!("[{}]", category).dimmed());
    }

    if let Some(due_date) = task.due_date {
        let due_str = due_date.format("%m/%d").to_string();
        if task.is_overdue() {
            print!(" {}", format!("📅{}", due_str).red());
        } else {
            print!(" {}", format!("📅{}", due_str).dimmed());
        }
    }

    println!();
}