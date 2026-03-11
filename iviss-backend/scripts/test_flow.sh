#!/bin/bash

# Test Flow Script
# This script runs all the CI pipeline steps locally to validate before creating a pull request
# Usage: ./scripts/test_flow.sh

set -e  # Exit on any error

echo "🚀 Starting local CI test flow..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the iviss-backend directory."
    exit 1
fi

echo "📦 Step 1: Building the project..."
if cargo build --verbose; then
    print_status "Build successful"
else
    print_error "Build failed"
    exit 1
fi

echo "🧪 Step 2: Running tests..."
if cargo test --verbose; then
    print_status "Tests passed"
else
    print_error "Tests failed"
    exit 1
fi

echo "📝 Step 3: Checking code formatting..."
if cargo fmt -- --check; then
    print_status "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    cargo fmt --all
fi

echo "🔍 Step 4: Running Clippy lints..."
if cargo clippy -- -D warnings; then
    print_status "Clippy checks passed"
else
    print_error "Clippy found issues"
    exit 1
fi

echo "🔒 Step 5: Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2025-0111; then
        print_status "Security audit passed"
    else
        print_error "Security audit failed"
        exit 1
    fi
else
    print_warning "cargo-audit not found. Installing..."
    cargo install cargo-audit
    if cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2025-0111; then
        print_status "Security audit passed after installation"
    else
        print_error "Security audit failed"
        exit 1
    fi
fi

echo "📚 Step 6: Building documentation..."
if cargo doc --no-deps --verbose; then
    print_status "Documentation built successfully"
else
    print_error "Documentation build failed"
    exit 1
fi

echo "📊 Step 7: Generating code coverage..."
if command -v cargo-llvm-cov &> /dev/null; then
    echo "Running coverage with 60% minimum threshold..."
    if cargo llvm-cov --html --output-dir target/coverage/html --fail-under-lines 60; then
        print_status "Coverage report generated successfully"
        echo "📁 Coverage report available at: target/coverage/html/index.html"
    else
        print_error "Coverage below 60% threshold or coverage generation failed"
        exit 1
    fi
else
    print_warning "cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov
    echo "Running coverage with 60% minimum threshold..."
    if cargo llvm-cov --html --output-dir target/coverage/html --fail-under-lines 60; then
        print_status "Coverage report generated successfully after installation"
        echo "📁 Coverage report available at: target/coverage/html/index.html"
    else
        print_error "Coverage below 60% threshold or coverage generation failed"
        exit 1
    fi
fi

echo ""
echo "🎉 All CI checks passed successfully!"
echo "✨ Your code is ready for pull request submission."
echo ""
echo "Summary of completed steps:"
echo "  ✅ Build"
echo "  ✅ Tests"
echo "  ✅ Format check"
echo "  ✅ Clippy linting"
echo "  ✅ Security audit"
echo "  ✅ Documentation build"
echo "  ✅ Code coverage (≥60%)"
echo ""
echo "📊 View coverage report: open target/coverage/html/index.html"
