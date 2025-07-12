# Performance Benchmarks & Gates

This document tracks performance benchmarks and gates for the GTA-style game project.

## Sprint 4 Performance Gates

### 1. Spawn Performance (`spawn_100k`)
- **Target**: ≤3.0ms on release build for 100k mixed prefabs  
- **Legacy Baseline**: ≤1.2× legacy f430bc6 performance
- **Test Command**: `cargo bench -p gameplay_factory --bench factory_spawn spawn_100k`
- **CI Integration**: Automatic benchmark execution on every PR

**Entity Mix for 100k Benchmark**:
- 25% Vehicles (25,000 entities)
- 25% NPCs (25,000 entities) 
- 25% Buildings (25,000 entities)
- 25% Props (25,000 entities)

### 2. Hot-Reload Latency
- **Target**: <16ms from disk write to ECS update
- **Test**: `cargo test -p config_core hot_reload_latency`
- **Measurement**: End-to-end latency including file system detection and ECS resource updates

## Current Performance Status

### Baseline Metrics (as of Sprint 4)
```
Benchmark: spawn_100k/mixed_prefabs/1000      Time: ~0.86ms   ✅ Under target
Benchmark: spawn_100k/mixed_prefabs/10000     Time: ~10.0ms   ⚠️  Above target
Benchmark: spawn_100k/mixed_prefabs/100000    Time: ~110.1ms  ❌ Far above target
```

**Performance Analysis:**
- **1k entities**: 0.86ms (excellent, well under 3.0ms target)
- **10k entities**: 10.0ms (3.3× over target, linear scaling showing no optimization)  
- **100k entities**: 110.1ms (36.7× over target, needs major optimization)

**Scaling Factor**: ~1.1ms per 1k entities (linear, no batch optimization detected)

### Hot-Reload Performance
```
Config file write latency:    <1ms    ✅ Excellent
Config parsing latency:       <1ms    ✅ Fast parsing
Simulated hot-reload:         <16ms   ✅ Under target
```

## Historical Performance Tracking

### Sprint 3 Vehicle Physics Performance
- Vehicle physics update time: 0.180ms/tick (target: <1.5ms) ✅
- Audio integration overhead: <0.050ms/tick ✅
- Combined FPS impact: 120+ FPS stable ✅

### Sprint 2 Foundation Performance  
- ECS world creation: <0.1ms ✅
- Component registration: <0.05ms per component ✅
- Test suite execution: 180+ tests in <30s ✅

## Performance Architecture

### Benchmark Infrastructure
- **Framework**: Criterion.rs with HTML reports
- **CI Integration**: GitHub Actions benchmark job
- **Artifact Storage**: 30-day retention for trend analysis
- **Regression Detection**: Automatic alerts on >20% degradation

### Optimization Strategies
1. **Batch Processing**: Minimize per-entity allocation overhead
2. **Memory Pools**: Pre-allocated entity storage
3. **Component Reuse**: Reduce GC pressure from component maps
4. **Parallel Spawning**: Multi-threaded entity creation when safe

### Current Performance Analysis (Sprint 4)
Based on benchmarks, the 100k spawn target requires major optimization:

**Performance Bottlenecks Identified:**
- **Linear scaling**: No batch optimization (1.1ms per 1k entities)
- **Individual spawns**: Each entity allocates separately
- **RON parsing**: String->Value conversion per entity
- **ECS overhead**: Commands queue grows linearly

**Required Improvement**: 100k entities must improve from 110ms to <3ms (37× faster)

**Recommended Implementation Path:**
1. **Phase 1**: Implement `spawn_batch()` method in Factory
2. **Phase 2**: Add memory pooling for ComponentMap objects  
3. **Phase 3**: Cache parsed component data between entities
4. **Phase 4**: Parallel spawning with work-stealing

## Performance Gates in CI

### Benchmark Job (`benchmark`)
```yaml
- name: Run gameplay_factory spawn_100k benchmark
  run: cargo bench -p gameplay_factory --bench factory_spawn spawn_100k

- name: Check performance gates  
  run: |
    echo "🎯 Performance Gate: spawn_100k should be ≤3.0ms on release build"
    # Future: Add actual gate validation script
```

### Hot-Reload Testing
- Integrated into `config_core` test suite
- Fails CI if latency exceeds 16ms
- Tests both file system detection and ECS integration

## Monitoring & Alerting

### Performance Regression Detection
- **Threshold**: >20% degradation from baseline triggers review
- **Trend Analysis**: Weekly performance reports from Criterion data
- **Platform Variance**: MacOS, Linux, Windows benchmark comparison

### Development Guidelines
- Run benchmarks locally before submitting performance-critical PRs
- Use `cargo bench` for isolated performance testing
- Profile with `perf` or `tracy` for detailed analysis

## Future Performance Work

### Sprint 5-6: Rendering & Culling
- GPU culling benchmark: <0.3ms per frame for frustum culling
- Batch rendering: 2.5× improvement over individual draws
- LOD system: <0.1ms per frame for distance calculations

### Sprint 7-8: Professional Integration  
- Plugin initialization: <50ms total startup time
- Service elimination: Remove remaining container overhead
- Memory optimization: <10MB baseline memory usage

## Troubleshooting Performance Issues

### Common Performance Bottlenecks
1. **Entity Creation**: Excessive allocation in component maps
2. **Hot-Reload**: File system polling overhead  
3. **ECS Updates**: Resource contention during config changes
4. **Memory Fragmentation**: Frequent spawn/despawn cycles

### Debug Commands
```bash
# Run single benchmark with detailed output
cargo bench -p gameplay_factory --bench factory_spawn spawn_100k -- --verbose

# Profile benchmark with perf
perf record cargo bench -p gameplay_factory --bench factory_spawn spawn_100k
perf report

# Test hot-reload latency locally
cargo test -p config_core hot_reload_latency -- --nocapture
```

## Performance Contact

For performance issues or questions:
- Review benchmark artifacts in GitHub Actions
- Check Criterion HTML reports in `target/criterion/`
- Consult AGENT.md for architecture guidance
- Reference Oracle consultations for strategic decisions
