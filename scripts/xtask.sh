#!/usr/bin/env bash
# Convenience script to run xtask commands
# Usage: ./scripts/xtask.sh [command] [args...]

set -euo pipefail

# Change to workspace root
cd "$(dirname "$0")/.."

# Run xtask with provided arguments
cargo run -p xtask -- "$@"
