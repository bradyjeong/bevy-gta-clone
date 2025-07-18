#!/bin/bash

# Headless benchmark script for Amp game engine
# Usage: ./scripts/run_headless_benchmark.sh [frames] [scene]

set -e

# Default values
FRAMES=${1:-1000}
SCENE=${2:-medium}
OUTPUT_DIR="target/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Running headless benchmark..."
echo "Frames: $FRAMES"
echo "Scene: $SCENE"
echo "Output: $OUTPUT_DIR"

# Function to run benchmark for a specific scene
run_benchmark() {
    local scene_name=$1
    local scene_file="assets/scenes/${scene_name}.bevy"
    local output_file="$OUTPUT_DIR/benchmark_${scene_name}_${TIMESTAMP}.json"
    
    echo "Running benchmark for scene: $scene_name"
    
    if [ ! -f "$scene_file" ]; then
        echo "Scene file not found: $scene_file"
        return 1
    fi
    
    # Run the benchmark
    RUST_LOG=info cargo run --release --example benchmark_runner -- \
        --scene "$scene_file" \
        --frames "$FRAMES" \
        --headless \
        --output "$output_file"
    
    echo "Benchmark completed for $scene_name"
    echo "Results saved to: $output_file"
}

# Run benchmark for specified scene or all scenes
if [ "$SCENE" = "all" ]; then
    echo "Running benchmarks for all scenes..."
    run_benchmark "empty"
    run_benchmark "medium"
    run_benchmark "urban_heavy"
else
    run_benchmark "$SCENE"
fi

echo "Benchmark run completed!"
echo "Results available in: $OUTPUT_DIR"

# Generate summary report
echo "Generating summary report..."
python3 scripts/generate_benchmark_report.py "$OUTPUT_DIR"

echo "Done!"
