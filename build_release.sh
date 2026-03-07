#!/bin/bash
# Build LabaClaw in release mode
set -e
echo "Building LabaClaw in release mode..."
cd "$(dirname "$0")"
cargo build --release
echo "Build completed successfully!"
echo "Binary location: target/release/labaclaw"
