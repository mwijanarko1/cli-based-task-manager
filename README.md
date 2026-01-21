# Task Manager - Enterprise CLI Tool

A task management CLI tool built in Rust, featuring comprehensive task lifecycle management, data persistence, and error handling.

## Features

- **Task Management**: Add, update, complete, start, and cancel tasks
- **Interactive Selection**: Choose tasks from numbered lists when ID is not provided
- **Advanced Filtering**: Filter by status, priority, category, and search queries
- **Statistics**: Comprehensive task analytics and reporting
- **Data Persistence**: JSON-based storage with automatic saving
- **Rich CLI**: Colored output, progress indicators, and intuitive commands
- **Type Safety**: Rust's ownership system prevents data races and memory errors
- **Validation**: Input validation with detailed error messages
- **Bulk Operations**: Clear completed tasks or delete all tasks at once
- **Import/Export**: Backup and migrate tasks between systems
- **Extensible**: Modular architecture for easy feature additions

## Installation

### One-Line Install (Recommended) 🚀

Get started instantly with this one-liner:

```bash
curl -fsSL https://raw.githubusercontent.com/mwijanarko1/cli-based-task-manager/main/install.sh | bash
```

**Requirements:** Rust/Cargo must be installed. If not, install Rust first:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Manual Installation

#### Option 1: Using Cargo (Direct)
```bash
cargo install --git https://github.com/mwijanarko1/cli-based-task-manager.git task-manager
```

#### Option 2: From Source
```bash
git clone https://github.com/mwijanarko1/cli-based-task-manager.git
cd cli-based-task-manager
cargo build --release
# Binary will be in target/release/task-manager
```

#### Option 3: Download Pre-built Binary
Check the [Releases](https://github.com/mwijanarko1/cli-based-task-manager/releases) page for pre-built binaries.

### Verify Installation

After installation, verify it works:

```bash
task-manager --version
task-manager --help
```

You should see the version and help output. If not, check that `~/.cargo/bin` is in your PATH.

## Quick Start

Get started with your task manager in minutes:

```bash
# 1. Build the project (if installing from source)
cargo build --release
./target/release/task-manager --help

# 2. Add your first task
task-manager add "Complete project documentation" --priority high

# 3. Add a task with more details
task-manager add "Review pull requests" \
  --description "Review and merge pending pull requests" \
  --category work \
  --priority medium

# 4. List your tasks
task-manager list

# 5. Mark a task as in progress
task-manager start <task-id>

# 6. Complete a task
task-manager complete <task-id>

# 7. Get productivity stats
task-manager stats
```

## Usage

### Basic Commands

```bash
# Get help
task-manager --help

# Add a new task
task-manager add "Learn Rust Ownership" --description "Master Rust's ownership model" --priority high

# List all tasks
task-manager list

# List only completed tasks
task-manager list --status done

# Search for tasks
task-manager list --search "rust"

# Sort tasks by priority
task-manager list --sort priority-desc

# Show task details
task-manager show <task-id>

# Complete a task (with or without ID)
task-manager complete [task-id]
task-manager start [task-id]

# Interactive selection: if no ID provided, shows numbered list

# Update a task
task-manager update <task-id> --title "New Title" --priority critical

# Get task statistics
task-manager stats

# Clear completed tasks
task-manager clear

# Delete ALL tasks (with confirmation)
task-manager delete-all

# Delete ALL tasks (without confirmation)
task-manager delete-all --force

# Export tasks to JSON
task-manager export tasks_backup.json

# Import tasks from JSON
task-manager import tasks_backup.json
```

## Task Management Workflow

### Daily Workflow Example
```bash
# Start your day - review today's tasks
task-manager list --status todo --sort priority-desc

# Begin working on highest priority task
task-manager start <task-id>

# Complete the task
task-manager complete <task-id>

# Add new tasks as they come up
task-manager add "Fix critical bug" --priority critical --category work

# End of day - review completed work
task-manager stats
task-manager list --status done

# Archive completed tasks
task-manager clear
```

### Project Management
```bash
# Create project tasks
task-manager add "Design system architecture" --category project-alpha --priority high
task-manager add "Implement core features" --category project-alpha --priority high
task-manager add "Write documentation" --category project-alpha --priority medium
task-manager add "Setup CI/CD pipeline" --category project-alpha --priority medium

# Track project progress
task-manager list --category project-alpha
task-manager stats

# Set due dates for time-sensitive tasks
task-manager update <task-id> --due-date "2024-02-01T17:00:00Z"
```

### Filtering and Organization
```bash
# View tasks by priority
task-manager list --priority critical
task-manager list --priority high

# Find overdue tasks
task-manager list --overdue

# Search for specific tasks
task-manager list --search "documentation"
task-manager list --category work

# Sort by different criteria
task-manager list --sort due-date-asc
task-manager list --sort created-desc
```

### Advanced Examples

```bash
# Add task with due date and category
task-manager add "Project deadline" \
  --description "Complete the quarterly report" \
  --priority critical \
  --category work \
  --due-date "2024-03-15T17:00:00Z"

# Filter by multiple criteria
task-manager list --status in-progress --priority high --category work

# Show overdue tasks
task-manager list --overdue

# Limit results and sort by due date
task-manager list --limit 10 --sort due-date-asc
```

## Data Storage

Tasks are automatically saved to `tasks.json` in your current directory. The file contains:

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Complete project documentation",
    "description": "Write comprehensive API documentation",
    "priority": "high",
    "status": "in_progress",
    "category": "work",
    "due_date": "2024-02-01T17:00:00Z",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T14:20:00Z",
    "completed_at": null
  }
]
```

### Backup and Migration
```bash
# Export tasks for backup
task-manager export my_tasks_backup.json

# Import tasks from another system
task-manager import imported_tasks.json

# Move to a different data file
task-manager --file ~/my_tasks.json list
```

## Command Reference

### `add`
Add a new task to the manager.

```bash
task-manager add <TITLE> [OPTIONS]
```

**Options:**
- `--description <TEXT>`: Task description
- `--priority <LEVEL>`: Priority (low, medium, high, critical)
- `--category <NAME>`: Task category
- `--due-date <ISO8601>`: Due date in ISO 8601 format

### `list`
List tasks with optional filtering and sorting.

```bash
task-manager list [OPTIONS]
```

**Options:**
- `--status <STATUS>`: Filter by status (todo, in-progress, done, cancelled)
- `--priority <LEVEL>`: Filter by priority
- `--category <NAME>`: Filter by category
- `--overdue`: Show only overdue tasks
- `--search <QUERY>`: Search in title and description
- `--sort <CRITERIA>`: Sort by (created-asc, created-desc, due-date-asc, due-date-desc, priority-asc, priority-desc, title-asc, title-desc)
- `--limit <NUMBER>`: Limit number of results

### `show`
Display detailed information about a specific task.

```bash
task-manager show <TASK-ID>
```

### `update`
Update an existing task.

```bash
task-manager update <TASK-ID> [OPTIONS]
```

**Options:**
- `--title <TEXT>`: New title
- `--description <TEXT>`: New description (empty string to clear)
- `--priority <LEVEL>`: New priority
- `--category <TEXT>`: New category (empty string to clear)
- `--due-date <ISO8601>`: New due date (empty string to clear)

### `complete`
Mark a task as completed. Shows interactive selection if no ID provided.

```bash
task-manager complete [TASK-ID]
```

### `start`
Mark a task as in progress. Shows interactive selection if no ID provided.

```bash
task-manager start [TASK-ID]
```

### `cancel`
Cancel a task. Shows interactive selection if no ID provided.

```bash
task-manager cancel [TASK-ID]
```

### `delete`
Delete a task (with confirmation). Shows interactive selection if no ID provided.

```bash
task-manager delete [TASK-ID] [--force]
```

### `stats`
Display task statistics.

```bash
task-manager stats
```

### `clear`
Clear completed tasks (or all tasks with `--all`).

```bash
task-manager clear [--all] [--force]
```

### `delete-all`
Delete ALL tasks (destructive operation).

```bash
task-manager delete-all [--force]
```

### `export`
Export tasks to a JSON file.

```bash
task-manager export <FILE>
```

### `import`
Import tasks from a JSON file.

```bash
task-manager import <FILE>
```

## Configuration

### Environment Variables
- `TASK_MANAGER_FILE`: Path to the data file (default: `tasks.json`)
- `TASK_MANAGER_CONFIG`: Path to configuration file
- `RUST_LOG`: Logging level (error, warn, info, debug, trace)

### Configuration File
Create a `config.toml`, `config.json`, or `config.yaml` file:

```toml
# config.toml
data_file = "my_tasks.json"
auto_save = true
verbose = false
```

## Data Model

### Task Structure
```rust
struct Task {
    id: Uuid,                    // Unique identifier
    title: String,              // Task title (required, 1-200 chars)
    description: Option<String>, // Optional description (max 2000 chars)
    priority: Priority,         // low, medium, high, critical
    status: TaskStatus,         // todo, in-progress, done, cancelled
    category: Option<String>,   // Optional category (max 50 chars)
    due_date: Option<DateTime>, // Optional due date
    created_at: DateTime,       // Creation timestamp
    updated_at: DateTime,       // Last modification timestamp
    completed_at: Option<DateTime>, // Completion timestamp
}
```

### Priority Levels
- **Low**: Nice to have
- **Medium**: Should do
- **High**: Important
- **Critical**: Urgent

### Task Status
- **TODO**: Not started
- **IN PROGRESS**: Currently working on
- **DONE**: Completed
- **CANCELLED**: No longer needed

## Architecture

### Core Components

1. **CLI Layer** (`cli.rs`): Command-line interface using clap
2. **Task Model** (`task.rs`): Data structures and business logic
3. **Task Manager** (`manager.rs`): Core business logic and operations
4. **Error Handling** (`error.rs`): Comprehensive error types and handling
5. **Main** (`main.rs`): Application entry point and command dispatch

### Key Design Principles

- **Type Safety**: Rust's ownership system prevents memory errors
- **Error Handling**: Comprehensive error types with detailed messages
- **Separation of Concerns**: Each module has a single responsibility
- **Extensibility**: Easy to add new features and commands
- **Performance**: Efficient data structures and algorithms

## Safety & Reliability

This tool leverages Rust's ownership and borrowing system to provide:

- **Memory Safety**: No null pointer dereferences or buffer overflows
- **Thread Safety**: Data race prevention through ownership rules
- **Type Safety**: Compile-time guarantees about data validity
- **Resource Management**: Automatic cleanup of resources

## Testing

Run the test suite:

```bash
cargo test
```

Run with coverage (requires tarpaulin):

```bash
cargo tarpaulin
```

## Performance

- **Fast Startup**: Minimal initialization overhead
- **Efficient Storage**: JSON serialization with minimal memory usage
- **Scalable**: HashMap-based storage for O(1) task lookups
- **Low Memory**: No memory leaks, efficient data structures

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Troubleshooting

### Common Issues

**"Command not found" error**
```bash
# If installed via cargo, ensure ~/.cargo/bin is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or run directly from target directory
./target/release/task-manager --help
```

**Permission denied when saving tasks**
```bash
# Check write permissions on current directory
ls -la

# Or specify a different data file location
task-manager --file ~/my_tasks.json add "My Task"
```

**Tasks not persisting between sessions**
- Tasks are saved to `tasks.json` in the current directory
- Change directories with `cd` or specify a custom file with `--file`
- Check that you have write permissions

**Invalid date format errors**
```bash
# Use ISO 8601 format for dates
task-manager add "Meeting" --due-date "2024-02-01T14:30:00Z"

# Common formats:
# 2024-02-01T14:30:00Z    (UTC with Z)
# 2024-02-01T09:30:00-05:00 (with timezone offset)
```

**Build errors**
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Getting Help

```bash
# Show all available commands
task-manager --help

# Get help for specific command
task-manager add --help
task-manager list --help

# Enable verbose logging
task-manager --verbose list
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for memory safety and performance
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Serialization using [serde](https://serde.rs/)
- Date/time handling with [chrono](https://github.com/chronotope/chrono)

---