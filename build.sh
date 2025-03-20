#!/bin/bash
set -e

echo "Building Timetracker (TT)..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Create directories if they don't exist
mkdir -p src/commands

# Build the project
cargo build

echo "Build completed successfully!"
echo "To run the program, use: cargo run -- [command]"
echo "For example: cargo run -- hello"
echo "Or: cargo run -- add \"Working on project\""
echo "Or: cargo run -- report"

# Instructions for installing the binary
echo ""
echo "To install the binary globally:"
echo "cargo install --path ."
echo ""
echo "This will install the 'tt' command to your PATH."
