#!/bin/bash
set -e

# Load .env if it exists
if [ -f "$(dirname "$0")/../.env" ]; then
  set -a
  source "$(dirname "$0")/../.env"
  set +a
fi

# Default coverage directory if not set in .env
COVERAGE_DIR="${COVERAGE_DIR:-coverage}"

# Make coverage dir absolute relative to project root (where this script is run from)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COVERAGE_PATH="$PROJECT_ROOT/$COVERAGE_DIR"

echo "📁 Coverage output directory: $COVERAGE_PATH"
mkdir -p "$COVERAGE_PATH"

# Set LLVM_PROFILE_FILE so all .profraw files land in COVERAGE_DIR
export LLVM_PROFILE_FILE="$COVERAGE_PATH/test-%p-%m.profraw"

# Set rustflags for coverage instrumentation
export RUSTFLAGS="-C instrument-coverage"

echo "🧪 Running tests with coverage instrumentation..."
cd "$PROJECT_ROOT"
cargo test 2>&1

echo "🔍 Merging profraw files..."
llvm-profdata merge -sparse "$COVERAGE_PATH/"*.profraw -o "$COVERAGE_PATH/merged.profdata"

# Find the test binary
TEST_BINARY=$(ls "$PROJECT_ROOT/target/debug/deps/iviss_backend-"* 2>/dev/null | head -n 1)
if [ -z "$TEST_BINARY" ]; then
  echo "❌ Could not find test binary. Make sure the project has been built."
  exit 1
fi

echo "📊 Generating coverage summary..."
llvm-cov report \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile="$COVERAGE_PATH/merged.profdata" \
  --object "$TEST_BINARY" \
  > "$COVERAGE_PATH/coverage_summary.txt"

echo "📄 Generating HTML report..."
llvm-cov show \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile="$COVERAGE_PATH/merged.profdata" \
  --object "$TEST_BINARY" \
  --format=html \
  --output-dir="$COVERAGE_PATH/html" \
  2>/dev/null || true

echo ""
echo "✅ Coverage complete! Results saved to: $COVERAGE_PATH"
echo "   Summary : $COVERAGE_PATH/coverage_summary.txt"
echo "   HTML    : $COVERAGE_PATH/html/index.html (if llvm-cov show succeeded)"

# Print summary to terminal as well
cat "$COVERAGE_PATH/coverage_summary.txt"
