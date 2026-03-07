#!/bin/bash
# Quick setup script to run LabaClaw web dashboard

echo "🦀 LabaClaw Web Dashboard Setup"
echo "================================"

# Check if web assets are built
if [ ! -d "web/dist" ]; then
    echo "📦 Building web assets first..."
    cd web
    npm run build
    cd ..
    echo "✅ Web assets built!"
fi

# Build the project
echo "🔨 Building LabaClaw binary with embedded web dashboard..."
cargo build --release

# Check if build was successful
if [ -f "target/release/labaclaw" ]; then
    echo "✅ Build successful! Web dashboard is embedded in the binary."
    echo ""
    echo "🚀 Starting LabaClaw Gateway..."
    echo "📱 Dashboard URL: http://127.0.0.1:3000/"
    echo "🔧 API Endpoint: http://127.0.0.1:3000/api/"
    echo "⏹️  Press Ctrl+C to stop the gateway"
    echo ""

    # Start the gateway
    ./target/release/labaclaw gateway --open-dashboard
else
    echo "❌ Build failed! Please check the error messages above."
    exit 1
fi
