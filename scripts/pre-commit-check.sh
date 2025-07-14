#!/bin/bash
# Pre-commit validation script to prevent CI failures
# Run this before committing to ensure all CI checks pass locally

set -e

echo "🔍 Running pre-commit checks..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "ok" ]; then
        echo -e "${GREEN}✅ $message${NC}"
    elif [ "$status" = "error" ]; then
        echo -e "${RED}❌ $message${NC}"
    else
        echo -e "${YELLOW}⚠️  $message${NC}"
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status "error" "Not in project root directory"
    exit 1
fi

print_status "info" "Step 1/6: Checking version consistency..."
if ./scripts/check-version-consistency.sh; then
    print_status "ok" "Version consistency check passed"
else
    print_status "error" "Version consistency check failed"
    exit 1
fi

print_status "info" "Step 2/6: Checking formatting..."
if RUSTFLAGS="-Dwarnings" cargo fmt --all -- --check; then
    print_status "ok" "Formatting check passed"
else
    print_status "error" "Formatting check failed"
    echo "Run: cargo fmt --all"
    exit 1
fi

print_status "info" "Step 3/6: Running clippy..."
if RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    print_status "ok" "Clippy check passed"
else
    print_status "error" "Clippy check failed"
    exit 1
fi

print_status "info" "Step 4/6: Running tests..."
if RUSTFLAGS="-Dwarnings" cargo test --workspace --all-features --lib --bins --tests; then
    print_status "ok" "Tests passed"
else
    print_status "error" "Tests failed"
    exit 1
fi

print_status "info" "Step 5/6: Checking documentation..."
if RUSTFLAGS="-Dwarnings" cargo doc --workspace --no-deps --all-features; then
    print_status "ok" "Documentation build passed"
else
    print_status "error" "Documentation build failed"
    exit 1
fi

print_status "info" "Step 6/6: Checking rustdoc warnings..."
if RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features; then
    print_status "ok" "Rustdoc warnings check passed"
else
    print_status "error" "Rustdoc warnings check failed"
    exit 1
fi

# Note: Doctests are skipped due to macOS dynamic library issues
# They can be run manually with: cargo test --doc

print_status "ok" "All pre-commit checks passed! 🎉"
echo ""
echo "Safe to commit and push to main."
