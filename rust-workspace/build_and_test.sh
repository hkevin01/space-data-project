#!/bin/bash

# Build and test script for the Rust space communication system

echo "🚀 Space Communication System - Build and Test Script"
echo "====================================================="

# Change to workspace directory
cd /home/kevin/Projects/space-data-project/rust-workspace

echo "📋 Checking Rust version..."
rustc --version
cargo --version

echo ""
echo "🔧 Building shared library..."
cd shared
if cargo build; then
    echo "✅ Shared library built successfully"
else
    echo "❌ Shared library build failed"
    exit 1
fi

echo ""
echo "🛰️  Building satellite system..."
cd ../satellite
if cargo check; then
    echo "✅ Satellite system compiled successfully"
else
    echo "❌ Satellite system compilation failed"
    exit 1
fi

echo ""
echo "🌍 Building ground system..."
cd ../ground
if cargo check; then
    echo "✅ Ground system compiled successfully"
else
    echo "❌ Ground system compilation failed"
    exit 1
fi

echo ""
echo "🧪 Running tests..."
cd ..
if cargo test --workspace; then
    echo "✅ All tests passed"
else
    echo "❌ Some tests failed"
    exit 1
fi

echo ""
echo "📊 Generating documentation..."
if cargo doc --workspace --no-deps; then
    echo "✅ Documentation generated successfully"
else
    echo "❌ Documentation generation failed"
fi

echo ""
echo "🎉 Build and test completed successfully!"
echo ""
echo "Next steps:"
echo "  1. Start satellite simulator: cargo run --bin satellite"
echo "  2. Start ground station: cargo run --bin ground-station"
echo "  3. View documentation: cargo doc --open"
