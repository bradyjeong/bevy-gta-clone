# Benchmark Process & Performance Gates

Oracle's comprehensive performance testing and validation system for the AAA-grade game engine.

## Overview

This document describes the automated benchmark CI system that validates Oracle's performance targets and ensures the codebase meets AAA-grade performance standards.

## Benchmark Workflows

### 1. Main CI Benchmark (`ci.yml`)
- **Trigger**: Every push and PR to `main`/`develop`
- **Purpose**: Quick performance validation to prevent regressions
- **Duration**: ~5-10 minutes
- **Key Metrics**:
  - 100k entity spawn time (target: ≤3.0ms, gate: ≤3.2ms)
  - Factory spawn benchmark validation
  - Memory usage basic checks

### 2. Comprehensive Benchmark (`benchmark.yml`)
- **Trigger**: PRs, nightly schedule (2 AM UTC), manual dispatch
- **Purpose**: Detailed performance analysis and trend tracking
- **Duration**: ~30-60 minutes
- **Key Metrics**:
  - Frame time measurements
  - GPU processing time (if available)
  - Memory usage statistics
  - Component count analysis
  - All crate-specific benchmarks

### 3. Weekly Comprehensive Analysis
- **Trigger**: Nightly schedule (weekly)
- **Purpose**: Long-term performance trend analysis
- **Duration**: ~90-120 minutes
- **Features**:
  - Extended benchmark iterations
  - Memory profiling with Valgrind
  - Performance regression detection
  - Trend analysis and reporting

## Performance Gates

Oracle's performance gates are enforced automatically in CI:

### Critical Gates (CI Blocking)

#### Gate 1: 100k Entity Spawn Time
- **Target**: ≤3.0ms (Oracle's 37× improvement goal)
- **Threshold**: ≤3.2ms (allowing 6.7% headroom for CI variance)
- **Current**: ~111ms → target ≤3.0ms (37× improvement needed)
- **Measurement**: `perf_100k` example with optimized pattern
- **Failure Action**: CI fails, blocks merge

#### Gate 2: Factory Spawn Performance
- **Target**: ≤5.0ms for 100k entities
- **Measurement**: `factory_spawn` Criterion benchmark
- **Failure Action**: Warning (non-blocking)

#### Gate 3: Memory Usage
- **Target**: <90% system memory during benchmarks
- **Measurement**: System memory monitoring
- **Failure Action**: Warning (non-blocking)

### Advisory Gates (Non-blocking)

#### GPU Culling Performance
- **Target**: ≤0.25ms per frame
- **Measurement**: GPU culling benchmark (when available)
- **Failure Action**: Warning only (GPU may not be available in CI)

#### Frame Processing Time
- **Target**: ≤16.6ms (60 FPS gate)
- **Measurement**: Frame time analysis in `perf_100k`
- **Failure Action**: Advisory notice

## Benchmark Configuration

### Reproducible Environment
- **CPU Governor**: Performance mode
- **Test Threads**: Single-threaded (`RUST_TEST_THREADS=1`)
- **Compiler Flags**: `-C target-cpu=native -C opt-level=3`
- **Random Seed**: Fixed seed (42) for deterministic results
- **Iterations**: 3-5 for CI, 10+ for comprehensive benchmarks

### System Requirements
- **OS**: Ubuntu Latest (CI environment)
- **Memory**: Minimum 8GB for comprehensive benchmarks
- **CPU**: Performance governor enabled
- **Dependencies**: Full system dependency installation

## Benchmark Types

### 1. perf_100k Example
**Purpose**: Oracle's primary performance gate validation
**Command**: `cargo run --release --example perf_100k`
**Patterns**:
- `basic`: Basic Transform + Name + Visibility components
- `optimized`: Pre-compiled bundles with memory pools
- `mixed`: Mixed prefab types (vehicles, NPCs, buildings, props)

**Key Metrics**:
- Entity spawn time (ms)
- Memory usage (bytes)
- Component count
- Frame processing time

### 2. Factory Spawn Benchmark
**Purpose**: Criterion-based factory system validation
**Command**: `cargo bench -p gameplay_factory --bench factory_spawn`
**Scenarios**:
- DSL parsing performance
- Pre-compiled bundle optimization
- Mixed prefab type spawning

**Key Metrics**:
- Spawn time distribution
- Memory allocation patterns
- Scaling behavior (1k, 10k, 100k entities)

### 3. Gameplay Performance Benchmark
**Purpose**: Core gameplay system performance
**Command**: `cargo bench -p amp_gameplay --bench gameplay_performance`
**Systems**:
- Physics integration
- Audio system performance
- Component system overhead

### 4. GPU Culling Benchmark
**Purpose**: GPU vs CPU culling performance comparison
**Command**: `cargo bench -p amp_render --bench gpu_vs_cpu_culling`
**Scenarios**:
- Frustum culling performance
- GPU compute shader execution
- Memory transfer overhead

## Oracle's Optimization Strategy

### Current Performance Status
- **DSL Mixed Prefabs (100k)**: ~108ms
- **Optimized PrecompiledBundle (100k)**: ~5.6ms
- **Improvement Factor**: ~19× (target: 37×)
- **Gap**: Additional 1.95× improvement needed

### Optimization Roadmap

#### Phase 1: Pre-compiled Bundles ✅
- Remove DSL parsing from hot path
- Use pre-compiled component bundles
- **Achievement**: 19× improvement

#### Phase 2: Memory Pools (Current)
- Pre-allocate entity storage
- Object pooling for components
- **Target**: Additional 2× improvement

#### Phase 3: Batch Processing
- Optimize entity spawn batching
- Minimize allocations with per-frame arenas
- **Target**: Final optimization to reach 37× goal

## Usage Instructions

### Running Benchmarks Locally

#### Quick Performance Check
```bash
# Run the main performance gate test
cargo run --release --example perf_100k -- \
  --entity-count 100000 \
  --test-pattern optimized \
  --iterations 5 \
  --warmup
```

#### Comprehensive Benchmark Suite
```bash
# Run all Criterion benchmarks
cargo bench --workspace --all-features

# Run specific benchmark
cargo bench -p gameplay_factory --bench factory_spawn

# Run with JSON output for analysis
cargo bench --bench factory_spawn -- --output-format json
```

#### Memory Profiling
```bash
# Run with memory statistics
cargo run --release --example perf_100k -- \
  --entity-count 100000 \
  --test-pattern all \
  --memory-stats

# Run with Valgrind (Linux only)
valgrind --tool=massif --stacks=yes \
  cargo run --release --example perf_100k
```

### Interpreting Results

#### Performance Gate Status
- **✅ PASSED**: Performance target met
- **❌ FAILED**: Performance target not met (blocks CI)
- **⚠️ SLOW**: Performance degraded but not blocking

#### Benchmark Output
```
🎯 Oracle's Performance Gate Validation
======================================
Target: spawn_100k ≤3.0ms (Oracle's 37× improvement goal)
Threshold: ≤3.2ms (allowing headroom for CI variance)

Measured: 2.8ms
✅ PERFORMANCE GATE PASSED: 2.8ms ≤ 3.2ms
```

#### Optimization Recommendations
When gates fail, the system provides Oracle's optimization recommendations:
1. Use pre-compiled bundles instead of DSL parsing
2. Implement memory pools for entity storage
3. Batch spawn operations for better cache locality
4. Remove component validation from hot path
5. Use bevy's built-in component optimizations

## Artifact Storage

### Benchmark Results
- **Path**: `target/criterion/`
- **Retention**: 30 days (PR runs), 90 days (nightly)
- **Format**: HTML reports, JSON data

### Performance Trends
- **Path**: `target/criterion/reports/`
- **Retention**: 365 days (weekly comprehensive)
- **Format**: HTML trend analysis, CSV data

### Raw Output
- **Files**: `perf_100k_results.txt`, `factory_spawn_results.json`
- **Retention**: 30 days
- **Format**: Plain text, JSON

## Integration with Development Workflow

### Pre-commit Checks
- Run `./scripts/pre-commit-check.sh` before commits
- Includes quick performance validation

### PR Review Process
- Automated benchmark results posted as PR comments
- Performance gate status in CI checks
- Detailed artifacts available for review

### Release Validation
- Comprehensive benchmarks run before releases
- Performance regression detection
- Long-term trend analysis

## Troubleshooting

### Common Issues

#### Gate Failures
1. **Check system load**: High CPU/memory usage can affect results
2. **Verify compiler flags**: Ensure release build with optimizations
3. **Check for regressions**: Compare with previous successful runs

#### CI Environment Issues
1. **Timeout**: Increase timeout for comprehensive benchmarks
2. **Memory limits**: Adjust benchmark entity counts for CI constraints
3. **GPU availability**: GPU benchmarks may fail in CI (expected)

### Performance Debugging
```bash
# Profile hot paths
cargo run --release --example perf_100k -- \
  --entity-count 10000 \
  --test-pattern optimized \
  --iterations 1

# Generate flame graphs
cargo flamegraph --example perf_100k
```

## Contributing

### Adding New Benchmarks
1. Create benchmark in appropriate crate (`benches/` directory)
2. Add to CI workflow in `benchmark.yml`
3. Define performance gates if needed
4. Update this documentation

### Modifying Performance Gates
1. Consult Oracle's guidance on target changes
2. Update thresholds in CI workflows
3. Update documentation
4. Test gate behavior in CI

### Reporting Issues
- Include benchmark output and system information
- Provide reproduction steps
- Reference Oracle's optimization recommendations

## References

- [Oracle's Strategic Guidance](docs/oracle-consultations.md)
- [ADR-0008: AAA Feature Restoration](docs/adr/ADR-0008-aaa-feature-restoration.md)
- [Criterion Benchmark Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Bevy Performance Guide](https://bevyengine.org/learn/book/getting-started/ecs/#performance)
