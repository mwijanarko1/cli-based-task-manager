mod cli;
mod error;
mod manager;
mod task;

use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use error::Result;
use manager::{TaskManager, TaskManagerConfig};
use std::io::{self, Write};
use std::path::PathBuf;
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
            handle_list(&manager, status, priority, category, overdue, sort, limit, search)
        }
        Commands::Show { id } => handle_show(&manager, &id),
        Commands::Update { id, title, description, priority, category, due_date } => {
            handle_update(&mut manager, &id, title, description, priority, category, due_date).await
        }
        Commands::Complete { id } => handle_complete(&mut manager, &id).await,
        Commands::Start { id } => handle_start(&mut manager, &id).await,
        Commands::Cancel { id } => handle_cancel(&mut manager, &id).await,
        Commands::Delete { id, force } => handle_delete(&mut manager, &id, force).await,
        Commands::Stats => handle_stats(&manager),
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

fn handle_list(
    manager: &TaskManager,
    status: Option<cli::StatusArg>,
    priority: Option<cli::PriorityArg>,
    category: Option<String>,
    overdue: bool,
    sort: cli::SortArg,
    limit: Option<usize>,
    search: Option<String>,
) -> Result<()> {
    let mut tasks = if let Some(query) = search {
        manager.search_tasks(&query)
    } else if overdue {
        manager.get_overdue_tasks()
    } else if let Some(status) = status {
        manager.get_tasks_by_status(status.into())
    } else if let Some(priority) = priority {
        manager.get_tasks_by_priority(priority.into())
    } else if let Some(category) = category {
        manager.get_tasks_by_category(&category)
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

fn handle_show(manager: &TaskManager, id: &str) -> Result<()> {
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

async fn handle_update(
    manager: &mut TaskManager,
    id: &str,
    title: Option<String>,
    description: Option<String>,
    priority: Option<cli::PriorityArg>,
    category: Option<String>,
    due_date: Option<String>,
) -> Result<()> {
    let description = description.map(|d| if d.is_empty() { None } else { Some(d) });
    let category = category.map(|c| if c.is_empty() { None } else { Some(c) });
    let priority = priority.map(|p| p.into());

    let due_date = if let Some(date_str) = due_date {
        if date_str.is_empty() {
            Some(None)
        } else {
            Some(Some(crate::task::parse_datetime(&date_str)?))
        }
    } else {
        None
    };

    manager.update_task(id, title, description, priority, category, due_date)?;
    println!("{}", format!("✓ Updated task {}", id).green());
    Ok(())
}

async fn handle_complete(manager: &mut TaskManager, id: &str) -> Result<()> {
    manager.complete_task(id)?;
    println!("{}", format!("✓ Completed task {}", id).green());
    Ok(())
}

async fn handle_start(manager: &mut TaskManager, id: &str) -> Result<()> {
    manager.start_task(id)?;
    println!("{}", format!("▶ Started working on task {}", id).green());
    Ok(())
}

async fn handle_cancel(manager: &mut TaskManager, id: &str) -> Result<()> {
    manager.cancel_task(id)?;
    println!("{}", format!("❌ Cancelled task {}", id).yellow());
    Ok(())
}

async fn handle_delete(manager: &mut TaskManager, id: &str, force: bool) -> Result<()> {
    if !force {
        print!("Are you sure you want to delete task {}? (y/N): ", id);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    manager.delete_task(id)?;
    println!("{}", format!("🗑 Deleted task {}", id).red());
    Ok(())
}

fn handle_stats(manager: &TaskManager) -> Result<()> {
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

async fn handle_clear(manager: &mut TaskManager, all: bool, force: bool) -> Result<()> {
    let count = if all { manager.get_all_tasks().len() } else {
        manager.get_tasks_by_status(crate::task::TaskStatus::Done).len()
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
        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
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

async fn handle_import(manager: &mut TaskManager, file: PathBuf) -> Result<()> {
    let data = tokio::fs::read_to_string(&file).await?;
    let imported_tasks: Vec<crate::task::Task> = serde_json::from_str(&data)?;

    let mut imported_count = 0;
    for task in imported_tasks {
        if !manager.tasks.contains_key(&task.id.to_string()) {
            manager.tasks.insert(task.id.to_string(), task);
            imported_count += 1;
        }
    }

    manager.dirty = true;
    println!("{}", format!("📥 Imported {} tasks from {}", imported_count, file.display()).green());
    Ok(())
}

async fn handle_export(manager: &TaskManager, file: PathBuf) -> Result<()> {
    let tasks: Vec<&crate::task::Task> = manager.tasks.values().collect();
    let data = serde_json::to_string_pretty(&tasks)?;

    if let Some(parent) = file.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&file, data).await?;
    println!("{}", format!("📤 Exported {} tasks to {}", tasks.len(), file.display()).green());
    Ok(())
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

    let id = format!("{}...", &task.id.to_string()[..8]);
    let title = if task.title.len() > 40 {
        format!("{}...", &task.title[..37])
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