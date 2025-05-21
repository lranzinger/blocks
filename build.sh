#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# ============================
# Helper Functions
# ============================

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install a Cargo binary if not present
install_cargo_binary() {
    local binary="$1"
    local version="$2" # Optional: specific version to install

    if command_exists "$binary"; then
        echo "✔ $binary is already installed."
    else
        echo "✖ $binary is not installed. Installing..."
        if [ -z "$version" ]; then
            cargo install "$binary"
        else
            cargo install "$binary" --version "$version"
        fi
        echo "✔ $binary has been installed."
    fi
}

# Function to install wasm32-unknown-unknown target
install_wasm_target() {
    if rustup target list --installed | grep -q wasm32-unknown-unknown; then
        echo "✔ wasm32-unknown-unknown target is already installed."
    else
        echo "✖ wasm32-unknown-unknown target not found. Installing..."
        rustup target add wasm32-unknown-unknown
        echo "✔ wasm32-unknown-unknown target has been installed."
    fi
}

# ============================
# Install Dependencies
# ============================

echo "🛠️  Installing Development Dependencies..."

# 1. Install simple-http-server if not installed
install_cargo_binary "simple-http-server"
install_cargo_binary "wasm-opt"

# 2. Install wasm32-unknown-unknown Rust target if not installed
install_wasm_target

echo "✅ All development dependencies are installed."

# ============================
# Build Process
# ============================

# Parse build type argument
BUILD_TYPE=${1:-debug}  # Default to debug if no argument

# Display help if requested
if [[ "$BUILD_TYPE" == "-h" || "$BUILD_TYPE" == "--help" ]]; then
    echo "Usage: ./build.sh [debug|release]"
    echo "  debug   - Build with debug symbols"
    echo "  release - Build with optimizations"
    exit 0
fi

# Validate build type
if [[ "$BUILD_TYPE" != "debug" && "$BUILD_TYPE" != "release" ]]; then
    echo "❌ Error: Invalid build type. Use 'debug' or 'release'"
    exit 1
fi

# Build based on type
if [[ "$BUILD_TYPE" == "debug" ]]; then
    echo "🏗️  Building debug version..."
    cargo build --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/debug/blocks.wasm .
    echo "✔ Debug build complete: blocks.wasm"
else
    echo "🏗️  Building release version..."
    cargo build --target wasm32-unknown-unknown --release

    echo "⚙️  Optimizing with wasm-opt..."
    wasm-opt -O3 \
        --strip-debug \
        --strip-producers \
        -o blocks.wasm \
        target/wasm32-unknown-unknown/release/blocks.wasm
    echo "✔ Release build complete and optimized: blocks.wasm"
fi

echo "✅ Build process completed successfully."
