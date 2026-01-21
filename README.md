# Task Manager - Enterprise CLI Tool

A task management CLI tool built in Rust, featuring comprehensive task lifecycle management, data persistence, and error handling.

## 🚀 Features

- ✅ **Task Management**: Add, update, complete, start, and cancel tasks
- 🔍 **Advanced Filtering**: Filter by status, priority, category, and search queries
- 📊 **Statistics**: Comprehensive task analytics and reporting
- 💾 **Data Persistence**: JSON-based storage with automatic saving
- 🎨 **Rich CLI**: Colored output, progress indicators, and intuitive commands
- 🔒 **Type Safety**: Rust's ownership system prevents data races and memory errors
- 📝 **Validation**: Input validation with detailed error messages
- 🔧 **Extensible**: Modular architecture for easy feature additions

## 📦 Installation

### From Source
```bash
git clone <repository-url>
cd task-manager
cargo build --release
# Binary will be in target/release/task-manager
```

### Using Cargo
```bash
cargo install --git <repository-url> task-manager
```

## 🛠️ Usage

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

# Complete a task
task-manager complete <task-id>

# Start working on a task
task-manager start <task-id>

# Update a task
task-manager update <task-id> --title "New Title" --priority critical

# Get task statistics
task-manager stats

# Clear completed tasks
task-manager clear

# Export tasks to JSON
task-manager export tasks_backup.json

# Import tasks from JSON
task-manager import tasks_backup.json
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

## 📋 Command Reference

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
Mark a task as completed.

```bash
task-manager complete <TASK-ID>
```

### `start`
Mark a task as in progress.

```bash
task-manager start <TASK-ID>
```

### `cancel`
Cancel a task.

```bash
task-manager cancel <TASK-ID>
```

### `delete`
Delete a task (with confirmation).

```bash
task-manager delete <TASK-ID> [--force]
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

## 🔧 Configuration

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

## 📊 Data Model

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
- 🟢 **Low**: Nice to have
- 🟡 **Medium**: Should do
- 🟠 **High**: Important
- 🔴 **Critical**: Urgent

### Task Status
- 📋 **TODO**: Not started
- 🔄 **IN PROGRESS**: Currently working on
- ✅ **DONE**: Completed
- ❌ **CANCELLED**: No longer needed

## 🏗️ Architecture

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

## 🔒 Safety & Reliability

This tool leverages Rust's ownership and borrowing system to provide:

- **Memory Safety**: No null pointer dereferences or buffer overflows
- **Thread Safety**: Data race prevention through ownership rules
- **Type Safety**: Compile-time guarantees about data validity
- **Resource Management**: Automatic cleanup of resources

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run with coverage (requires tarpaulin):

```bash
cargo tarpaulin
```

## 📈 Performance

- **Fast Startup**: Minimal initialization overhead
- **Efficient Storage**: JSON serialization with minimal memory usage
- **Scalable**: HashMap-based storage for O(1) task lookups
- **Low Memory**: No memory leaks, efficient data structures

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) for memory safety and performance
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Serialization using [serde](https://serde.rs/)
- Date/time handling with [chrono](https://github.com/chronotope/chrono)

---

**Built for production use with enterprise-grade reliability and safety guarantees.**