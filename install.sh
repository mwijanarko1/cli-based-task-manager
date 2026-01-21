#!/bin/bash

# Task Manager CLI - Installation Script
# One-liner installation for the task-manager CLI tool

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check Rust installation
check_rust() {
    if ! command_exists cargo; then
        print_error "Cargo (Rust package manager) is not installed."
        print_info "Please install Rust first: https://rustup.rs/"
        print_info "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    print_success "Cargo found: $(cargo --version)"
}

# Main installation function
main() {
    echo -e "${BLUE}🚀 Installing Task Manager CLI${NC}"
    echo "=================================="

    # Check if Rust is installed
    print_info "Checking Rust installation..."
    check_rust

    # Install the task manager
    print_info "Installing task-manager from GitHub..."
    if cargo install --git https://github.com/mwijanarko1/cli-based-task-manager.git task-manager --force; then
        print_success "Task Manager CLI installed successfully!"
        echo ""
        print_info "Try it out:"
        echo "  task-manager --help"
        echo "  task-manager add 'My first task'"
        echo "  task-manager list"
        echo ""
        print_info "Documentation: https://github.com/mwijanarko1/cli-based-task-manager"
    else
        print_error "Installation failed!"
        print_info "Try installing manually:"
        print_info "cargo install --git https://github.com/mwijanarko1/cli-based-task-manager.git task-manager"
        exit 1
    fi
}

# Run main function
main