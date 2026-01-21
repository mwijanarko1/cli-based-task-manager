# Vector-Based Task Manager

## Codebase Overview

This is an **enterprise-grade Rust CLI task manager** with comprehensive task management capabilities including CRUD operations, filtering, sorting, search, statistics, and interactive task selection.

**Stack:** Rust 2021, clap (CLI), tokio (async), serde/serde_json (persistence), chrono (datetime), tracing (logging)

**Structure:** Layered architecture with clear separation - CLI layer (clap) → Application logic (main.rs) → Business logic (manager.rs) → Data model (task.rs, error.rs)

**Key features:**
- 13 CLI commands (add, list, show, update, complete, start, cancel, delete, delete-all, stats, clear, import, export)
- JSON file persistence with auto-save
- Interactive task selection for state change commands
- Comprehensive filtering by status, priority, category, due date
- Full-text search with case-insensitive matching
- 8 sorting options
- Aggregate statistics with completion rate tracking
- Multi-layer validation (clap types → validator derive → business logic)

For detailed architecture, module documentation, and navigation guides, see [docs/CODEBASE_MAP.md](docs/CODEBASE_MAP.md).
