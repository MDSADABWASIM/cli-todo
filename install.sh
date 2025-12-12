#!/bin/bash
set -e

echo "Installing todo..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust and Cargo to continue."
    exit 1
fi

# Build and install using cargo
# --path . installs from the current directory
cargo install --path .

echo "todo installed successfully!"
echo "You can now run the application by typing 'todo' in your terminal."
