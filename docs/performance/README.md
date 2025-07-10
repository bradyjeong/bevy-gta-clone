# Performance Tracking

This directory contains performance measurements and analysis for the Oracle's 12-week restoration plan.

## Oracle's Performance Strategy

The Oracle requires **300% improvement** targets with baseline denominators established in Sprint 0. All measurements use the `city_demo_baseline` scene for consistency.

## Measurement Standards

### Scene Configuration
- **Entities:** ~5,000 static meshes (simple cubes/spheres)
- **Duration:** 30 seconds minimum
- **Resolution:** 1920x1080
- **Metrics:** Average FPS, 99th percentile frame time, memory usage

### Baseline (Sprint 0)
- **Date:** 2025-01-07
- **Average FPS:** 83.53
- **99th Percentile Frame Time:** 30.55 ms
- **Memory Usage:** 1,196 MB

### Target Progression
- **Sprint 1-2:** 100% improvement (167 FPS)
- **Sprint 3-4:** 200% improvement (251 FPS)  
- **Sprint 5-6:** 300% improvement (334 FPS)

## Running Measurements

```bash
# Run baseline measurement
cargo run --example city_demo_baseline

# Results automatically saved to console
# Manual documentation in dated markdown files
```

## File Structure

```
docs/performance/
├── README.md                 # This file
├── Baseline_2025-01-07.md    # Sprint 0 baseline
├── Sprint_1_Results.md       # Sprint 1 measurements (TBD)
├── Sprint_2_Results.md       # Sprint 2 measurements (TBD)
└── ...                       # Additional sprint measurements
```

## Oracle Validation

Each measurement must include:
- ✅ System information (OS, CPU, GPU)
- ✅ Bevy version and features
- ✅ Scene entity count
- ✅ Statistical analysis (average + 99th percentile)
- ✅ Memory usage estimation
- ✅ Methodology documentation

## Performance Tracking Commands

```bash
# Build and run baseline (30 second measurement)
cargo run --example city_demo_baseline

# Monitor resource usage during test
# (External tools like Activity Monitor on macOS)

# Verify scene complexity
# Scene logs entity count during startup
```

## Historical Performance Data

| Sprint | Date | FPS | Frame Time | 99th Percentile | Memory | Notes |
|--------|------|-----|------------|-----------------|---------|-------|
| 0 (Baseline) | 2025-01-07 | 83.53 | 11.97ms | 30.55ms | 1,196MB | M4 Max baseline |
| 1 | TBD | TBD | TBD | TBD | TBD | Batching implementation |
| 2 | TBD | TBD | TBD | TBD | TBD | LOD system |
| 3 | TBD | TBD | TBD | TBD | TBD | GPU culling |

## Oracle Compliance

This tracking system follows Oracle's requirements:
- Consistent measurement methodology
- Denominator establishment for improvement calculations
- Sprint-based milestone tracking
- Hardware-specific baselines for reproducibility
