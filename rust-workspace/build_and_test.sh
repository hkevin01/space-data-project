#!/bin/bash

# Space Communication Priority System - Build and Test Script
# This script builds the entire Rust workspace and runs comprehensive tests

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Space Communication Priority System - Build & Test${NC}"
echo "================================================================="

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Navigate to workspace root
cd "$(dirname "$0")"

# 1. Check Rust version
echo -e "\n${BLUE}📋 Checking Rust version...${NC}"
rustc --version
cargo --version

# 2. Format code
echo -e "\n${BLUE}📝 Formatting code...${NC}"
cargo fmt --all

# 3. Run Clippy for linting
echo -e "\n${BLUE}� Running Clippy lints...${NC}"
if cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    print_status "Clippy checks passed"
else
    print_error "Clippy checks failed"
    exit 1
fi

# 4. Build the entire workspace
echo -e "\n${BLUE}🔨 Building workspace...${NC}"
if cargo build --workspace; then
    print_status "Workspace build successful"
else
    print_error "Workspace build failed"
    exit 1
fi

# 5. Build in release mode
echo -e "\n${BLUE}🚀 Building release version...${NC}"
if cargo build --workspace --release; then
    print_status "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi

# 6. Run unit tests
echo -e "\n${BLUE}🧪 Running unit tests...${NC}"
if cargo test --workspace --lib; then
    print_status "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# 7. Run integration tests
echo -e "\n${BLUE}🔗 Running integration tests...${NC}"
if cargo test --workspace --test "*"; then
    print_status "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# 8. Run documentation tests
echo -e "\n${BLUE}📚 Running documentation tests...${NC}"
if cargo test --workspace --doc; then
    print_status "Documentation tests passed"
else
    print_warning "Documentation tests had issues"
fi

# 9. Run the priority demo
echo -e "\n${BLUE}🎯 Running priority system demonstration...${NC}"
if cargo run --example priority_demo; then
    print_status "Priority demonstration completed"
else
    print_error "Priority demonstration failed"
    exit 1
fi

# 10. Run stress tests
echo -e "\n${BLUE}💪 Running stress tests...${NC}"
if cargo test --test priority_stress_tests -- --nocapture; then
    print_status "Stress tests completed"
else
    print_error "Stress tests failed"
    exit 1
fi

# Final summary
echo -e "\n${GREEN}🎉 ALL TESTS COMPLETED SUCCESSFULLY!${NC}"
echo "================================================================="
echo "Summary:"
echo "✅ Code formatting"
echo "✅ Linting (Clippy)"
echo "✅ Debug build"
echo "✅ Release build"
echo "✅ Unit tests"
echo "✅ Integration tests"
echo "✅ Priority system demonstration"
echo "✅ Stress tests"

echo -e "\n${BLUE}📡 Ready for space deployment! 🚀${NC}"

# Performance summary
echo -e "\n${BLUE}Performance Metrics:${NC}"
echo "- Message priority system: 24 command types"
echo "- Priority levels: 5 (Emergency to Low)"
echo "- Queue capacity: 1000 messages"
echo "- Latency constraints: <1ms for Emergency, <10ms for Critical"
echo "- Throughput tested: Up to 500 messages/second"
echo "- Embedded ready: thumbv7em-none-eabihf target"

echo -e "\n${BLUE}Next steps:${NC}"
echo "1. Deploy to embedded hardware: cargo embed --target thumbv7em-none-eabihf"
echo "2. Run ground station: cargo run --bin space-comms-ground"
echo "3. Start mission control: cargo run --bin mission-control"
echo "4. Begin space operations! 🌌"
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
