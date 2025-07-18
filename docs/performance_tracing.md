# Performance Tracing with perf_trace

The `perf_trace` feature provides comprehensive performance tracing capabilities for the Amp engine, enabling detailed performance analysis and optimization.

## Overview

The tracing system provides:
- **Chrome DevTools Integration**: Export traces for visualization in Chrome DevTools
- **Structured Logging**: Hierarchical span-based performance measurement
- **Feature-Gated**: Zero runtime overhead when disabled
- **Comprehensive Coverage**: Instrumentation across all major systems

## Quick Start

### Enable Tracing

Add the `perf_trace` feature to your Cargo.toml:

```toml
[dependencies]
amp_engine = { path = "crates/amp_engine", features = ["perf_trace"] }
```

Or enable it globally:

```toml
[features]
default = ["perf_trace"]
```

### Basic Usage

```rust
use amp_engine::tracing::{init_default_tracing, TracingConfig};

fn main() {
    // Initialize tracing with default configuration
    if let Err(e) = init_default_tracing() {
        eprintln!("Failed to initialize tracing: {}", e);
    }
    
    // Your application code here
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AAAPlugins::default())
        .run();
}
```

### Custom Configuration

```rust
use amp_engine::tracing::{init_tracing, TracingConfig};

fn main() {
    let config = TracingConfig {
        chrome_output: true,
        chrome_path: "./my_trace.json".to_string(),
        console_output: true,
        filter_level: "amp_engine=debug,amp_render=trace".to_string(),
    };
    
    if let Err(e) = init_tracing(&config) {
        eprintln!("Failed to initialize tracing: {}", e);
    }
    
    // Your application code
}
```

## Configuration Options

### TracingConfig

- `chrome_output`: Enable Chrome DevTools trace output (default: true)
- `chrome_path`: Path for Chrome trace file (default: "./trace.json")
- `console_output`: Enable console trace output (default: false)
- `filter_level`: Filter level for tracing (default: all Amp crates at trace level)

### Environment Variables

Override configuration using environment variables:

```bash
RUST_LOG=amp_engine=trace,amp_render=debug cargo run --example perf_100k --features perf_trace
```

## Instrumented Systems

The following systems are instrumented with performance tracing:

### Engine Systems
- Plugin initialization and staging
- System scheduling and execution
- Memory pool operations

### Rendering Systems
- Instance extraction and culling
- CPU frustum culling
- GPU compute culling
- Batch processing
- LOD system updates

### Gameplay Systems
- Transform synchronization
- Vehicle physics integration
- Entity spawning and factory operations

### Factory Systems
- Bundle preparation and compilation
- Batch entity spawning
- Component map processing

## Usage Examples

### Running with Tracing

```bash
# Run perf_100k example with tracing
cargo run --example perf_100k --features perf_trace

# Run city demo with tracing
cargo run --example city_demo_baseline --features perf_trace,rapier3d_030

# Run with custom trace file
TRACE_FILE=./my_trace.json cargo run --example perf_100k --features perf_trace
```

### Viewing Traces

1. Open Chrome browser
2. Navigate to `chrome://tracing`
3. Click "Load" and select your trace file (default: `./trace.json`)
4. Explore the performance timeline

### Performance Analysis

The trace output includes:
- **Span Timing**: Precise timing for each system and operation
- **Hierarchical Structure**: Nested spans show system relationships
- **Custom Metrics**: Entity counts, processing times, memory usage
- **Frame Boundaries**: Clear visualization of frame processing

## Integration with Existing Tools

### Tracy Integration

The `perf_trace` feature works alongside existing Tracy profiling:

```bash
# Enable both Tracy and perf_trace
cargo run --example city_demo_baseline --features tracy,perf_trace
```

### Benchmarking

Use with criterion benchmarks for detailed analysis:

```bash
# Run benchmarks with tracing
cargo bench --features perf_trace
```

## Macros and Helpers

### Performance Span Macro

```rust
use amp_engine::perf_span;

fn my_system() {
    perf_span!("my_system_processing");
    // Your system code here
}
```

### Performance Event Macro

```rust
use amp_engine::perf_event;

fn my_system() {
    perf_event!("system_started", entities = 1000);
    // Processing...
    perf_event!("system_completed");
}
```

### Performance Timing Macro

```rust
use amp_engine::perf_time;

fn my_system() {
    let result = perf_time!("expensive_operation", {
        // Expensive computation
        compute_something()
    });
}
```

## Performance Impact

### With Feature Disabled
- **Zero Runtime Overhead**: All tracing code is compiled out
- **No Dependencies**: Tracing crates not included in final binary
- **Production Ready**: Safe for release builds

### With Feature Enabled
- **Minimal Overhead**: ~1-2μs per span
- **Structured Output**: Efficient binary format
- **Configurable**: Adjust verbosity and output format

## Best Practices

### Development Workflow

1. **Enable During Development**: Use `perf_trace` for performance analysis
2. **Disable for Release**: Remove feature flag for production builds
3. **Targeted Analysis**: Use specific filter levels for focused analysis
4. **Regular Profiling**: Include tracing in CI/CD performance gates

### Optimization Workflow

1. **Identify Bottlenecks**: Use Chrome DevTools to find slow spans
2. **Measure Impact**: Compare before/after traces
3. **Validate Improvements**: Ensure optimizations are effective
4. **Document Changes**: Record performance improvements

## Troubleshooting

### Common Issues

**Trace file not generated:**
- Check file permissions in output directory
- Ensure tracing was properly initialized
- Verify feature flag is enabled

**High overhead:**
- Reduce filter level verbosity
- Disable console output
- Use selective instrumentation

**Chrome tracing not loading:**
- Ensure trace file is complete (app exited cleanly)
- Check Chrome version compatibility
- Verify JSON format is valid

### Debug Options

```rust
// Enable verbose tracing
let config = TracingConfig {
    filter_level: "trace".to_string(),
    console_output: true,
    ..Default::default()
};
```

## Integration with Oracle's Performance Gates

The tracing system integrates with Oracle's performance measurement strategy:

### Performance Targets
- **Entity Spawn**: ≤3ms for 100k entities
- **GPU Culling**: ≤0.25ms per frame
- **Transform Sync**: ≤1.5ms per frame
- **Memory Usage**: Flat memory profile

### CI Integration

```yaml
# .github/workflows/performance.yml
- name: Run performance tests with tracing
  run: |
    cargo run --example perf_100k --features perf_trace
    # Analyze trace file for performance regressions
```

## Future Enhancements

### Planned Features
- **Real-time Monitoring**: Live performance dashboard
- **Automatic Analysis**: AI-powered bottleneck detection
- **Integration APIs**: Programmatic access to trace data
- **Performance Alerts**: Automatic regression detection

### Extensibility
- **Custom Spans**: Add domain-specific instrumentation
- **Metrics Export**: Integration with monitoring systems
- **Sampling**: Configurable trace sampling for production

## References

- [Oracle's Performance Strategy](docs/oracle-consultations.md)
- [Chrome DevTools Tracing](https://developer.chrome.com/docs/devtools/performance/reference/)
- [Tracing Crate Documentation](https://docs.rs/tracing)
- [Performance Benchmarking Guide](benches/README.md)
