# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Phase 3.F Implementation**: Final cleanup and benchmarking phase
- **Performance Tuning Resource**: Centralized configuration for all performance constants
- **Consolidated Coordinate Conversion**: Unified grid-to-world math utilities in `amp_math`
- **Comprehensive Performance Documentation**: Complete profiling guide in `docs/performance.md`
- **Benchmark Scenes**: Empty, medium (10k buildings, 2k lights), and heavy (34k buildings, 30k lights) scenes
- **Headless Benchmark Scripts**: Automated 1000-frame benchmark runs with reporting
- **Feature Gating**: Diagnostics behind `#[cfg(feature = "diagnostics")]`, profiling behind `#[cfg(feature = "profile")]`

### Changed
- **Thread Safety**: Replaced `static mut BUILDING_COUNTER` with `AtomicU32` for thread-safe ID generation
- **Code Hygiene**: Removed massive `#![allow(...)]` list from `amp_render` crate
- **Magic Numbers**: Moved hardcoded constants to `PerformanceTuning` resource loaded from RON configuration
- **Coordinate Conversion**: Consolidated duplicated grid-to-world math into `amp_math::coordinate_conversion`

### Fixed
- **Performance Diagnostics**: Fixed test compilation issues with proper Bevy plugin setup
- **Atomic Operations**: Replaced unsafe static mut with thread-safe atomic operations
- **Syntax Errors**: Fixed parentheses mismatch in coordinate conversion functions

### Performance Budgets
- **Frame Time**: 16.67ms target (60 FPS)
- **GPU Culling**: ≤0.25ms per frame
- **Batch Processing**: ≤2.5ms per frame
- **LOD Updates**: ≤1.0ms per frame
- **Streaming**: ≤1.5ms per frame
- **Resource Limits**: 256 max active lights, 50 max spawns per frame, 500 max batch count

### Configuration
- **Performance Tuning**: `assets/config/performance_tuning.ron` for centralized performance configuration
- **Benchmark Scenes**: `assets/scenes/{empty,medium,urban_heavy}.bevy` for standardized testing
- **Feature Flags**: `diagnostics` and `profile` features for conditional compilation

### Documentation
- **Performance Guide**: Complete profiling and optimization guide in `docs/performance.md`
- **Benchmark Scripts**: Automated headless benchmark execution with Python reporting
- **Module Documentation**: Inline docs explaining performance budgets and system purposes

## [0.4.0-alpha.0] - 2025-01-XX

### Added
- **Sprint 9 Completion**: Final optimization and polishing phase
- **GPU Culling Phase 3**: Real compute shader implementation with sub-0.25ms target
- **Memory Optimization**: Object pools, per-frame arenas, minimal allocations
- **Performance Gates**: Stable 60+ FPS @1080p with city_demo
- **Large-scale Optimization**: 100k entity spawn improvements
- **Release Preparation**: v0.4.0-alpha tag ready for deployment

### Performance Targets Met
- **Desktop (1080p)**: 60+ FPS stable
- **Memory**: Flat memory profile under sustained load
- **GPU Culling**: Real compute shader ≤0.25ms
- **Spawn Performance**: 100k entities with 37× improvement target
- **Test Coverage**: 370+ tests passing with ≥80% coverage

### Quality Gates
- **CI Performance**: All performance gates passing
- **Clippy**: Zero warnings with `-D warnings`
- **Documentation**: Complete rustdoc coverage
- **Examples**: All examples compile and run correctly

## Previous Releases

### [0.3.0] - Sprint 8 Completion
- Integration hardening and performance baseline
- AAAPlugins rollout and professional integration
- Performance CI with automated gates

### [0.2.0] - Sprint 7 Completion  
- Professional integration and GPU pipeline activation
- Bevy render-phase integration
- Technical debt resolution

### [0.1.0] - Sprint 6 Completion
- Rendering and performance optimization
- Batch processing and GPU culling foundations
- LOD system implementation

---

For complete release notes and migration guides, see the [releases page](https://github.com/bradyjeong/bevy-gta-clone/releases).
