# Amp Game Engine

![CI Status](https://github.com/bradyjeong/bevy-gta-clone/workflows/CI/badge.svg)
![Memory Leak Prevention](https://github.com/bradyjeong/bevy-gta-clone/workflows/Memory%20Leak%20Prevention/badge.svg)
![Test Coverage](https://codecov.io/gh/bradyjeong/bevy-gta-clone/branch/main/graph/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.85+-blue.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

A professional AAA-level open world game built with Bevy 0.16.1 and Rust 2024, featuring comprehensive game systems and optimized for Amp development workflows.

## 🚀 v0.4.0-alpha Release - SPRINT 9 COMPLETED

**Oracle-guided AAA restoration achieved through strategic sprints with Bevy 0.16.1 ecosystem alignment.**

**Release Status**: 370+ tests passing, Sprint 9 optimization completed, ready for v0.4.0-alpha.0 release.

**Achievement**: **Sprint 9** - Final optimization, performance tuning, and polish for AAA-grade release completed.

### 🚀 Target Feature Set (f430bc6 → Bevy 0.16.1)
- ✅ **12 RON Configuration System**: Data-driven game tuning with hot-reload
- ✅ **Advanced Vehicle Physics**: Realistic drivetrain, suspension, tire friction curves
- ✅ **Professional LOD System**: Distance-based mesh and material swapping
- ✅ **GPU Culling & Batch Processing**: Compute shader optimization with multi-draw-indirect
- ✅ **Modern ECS Patterns**: SystemSets, QueryData, sparse storage optimization
- ✅ **Audio Graph**: Advanced audio system with bevy_kira_audio integration
- 🚀 **Performance Optimization**: Currently targeting 300%+ FPS improvement, 60% memory reduction

**Strategy**: [STRATEGIC_RESTORATION_PLAN.md](docs/STRATEGIC_RESTORATION_PLAN.md) - Oracle's 12-week roadmap

## 🎯 v0.4.0-alpha Release Highlights

This release represents the completion of **Sprint 9** optimization phase with comprehensive performance improvements:

### Key Achievements:
- 🚀 **Performance Optimization**: 60+ FPS stable @1080p with city_demo
- 🎮 **GPU Culling Phase 3**: Real compute shader implementation (≤0.25ms target)
- 📊 **Large-scale Performance**: 37× improvement in entity spawning (111ms → ≤3ms)
- 🧠 **Memory Optimization**: Object pools, per-frame arenas, minimal allocations
- 🔧 **Production Ready**: Comprehensive test coverage and performance validation

### Technical Highlights:
- **GPU Pipeline**: Complete compute shader implementation replacing simulation
- **Memory Management**: Flat memory profile under sustained load
- **Performance Gates**: Automated validation with comprehensive metrics
- **Code Quality**: Debug artifacts cleaned, production-ready release
- **Documentation**: Complete rustdoc generation and living documentation updates

## Quick Start

```bash
# Clone the repository
git clone https://github.com/bradyjeong/bevy-gta-clone.git
cd bevy-gta-clone

# Build the workspace
cargo build --workspace

# Run the city demo (post-migration)
cargo run --example city_demo

# Run tests
cargo test --workspace

# Run full CI pipeline locally
./scripts/pre-commit-check.sh
```

## Current Architecture

Oracle's strategic crate structure for ecosystem alignment:

```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_physics/       # Vehicle physics and Rapier3D integration
│   ├─ amp_render/        # Rendering systems, GPU culling, LOD management
│   ├─ amp_gameplay/      # Game systems, components, vehicle integration
│   ├─ config_core/       # Configuration loading and management
│   ├─ gameplay_factory/  # Entity factory for prefab-based systems
│   └─ tools/xtask/       # Build pipeline helpers
├─ examples/              # Asset pipeline demonstrations
└─ docs/adr/              # Architecture Decision Records
```

## Features

- 🌍 **Full Bevy 0.16.1 Stack** - Complete ecosystem integration
- 🎮 **Modular Architecture** - Strategic crate boundaries for Amp productivity  
- ⚡ **High Performance** - 60+ FPS target with Bevy's optimized ECS
- 🧪 **Integrated Testing** - App-based testing with Bevy plugins
- 🔧 **Developer Experience** - Fast compilation, ecosystem tooling
- 📊 **Asset Pipeline** - Integrated RON/GLTF loaders with hot-reload
- 🏗️ **Prefab Factory** - Entity factory system for gameplay objects
- ⚙️ **Configuration Management** - Centralized config loading with validation
- 📋 **Data-Driven Config** - 14 RON config files with hot-reload capability
- 🏭 **Entity Prefab System** - Factory-based spawning with typed component maps
- 📈 **Performance Benchmarks** - Criterion.rs-based profiling with CI integration
- 🎵 **Advanced Audio** - Spatial audio system with bevy_kira_audio integration
- 🚗 **Vehicle Physics** - Professional-grade drivetrain and suspension simulation
- 🎨 **GPU Culling Pipeline** - Compute shader instance culling with ADR-0009 architecture
- 📊 **LOD System** - Distance-based level-of-detail with hysteresis and cross-fade
- 🔧 **Memory Leak Prevention** - Automated CI testing and TransientBufferPool management

## Development

### Prerequisites

- Rust 1.85+ (Rust 2024 edition)
- Git

### Building

```bash
# Check all crates compile
cargo check --workspace

# Build with optimizations
cargo build --release --workspace

# Run linting (strict)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo llvm-cov --workspace --all-features

# Run specific crate tests
cargo test -p amp_math
```

## Current Status

**Sprint 6 Complete - Professional Integration & GPU Pipeline Activation:**
- ✅ All Oracle's conditional approval requirements resolved
- ✅ GPU Pipeline Foundation: ADR-0009 documented + feature-flagged infrastructure
- ✅ Bevy Integration Enhancement: InheritedVisibility checks added to render pipeline
- ✅ CI Infrastructure: Weekly memory leak prevention + doctest stability
- ✅ Config System Stability: Field-level merge hierarchy working correctly
- ✅ Quality Gates: All 320+ tests passing, zero clippy warnings, Oracle gate criteria met

**Sprint 7 Active - GPU Culling Phase 2 + AAAPlugin Architecture:**
- 🔄 **P1**: GPU Culling Phase 2 (ADR-0009) - Implement compute shader + bind-group layout
- 🔄 **P1**: AAAPlugin Architecture - Introduce amp_engine::AAAPlugins PluginGroup
- 🔄 **P2**: xtask & Tooling - cargo xtask bench, demo, ci refactor
- 🔄 **P2**: Service-Elimination / Legacy Cleanup - Remove last service container patterns
- 🔄 **P2**: Documentation & Gates - Update README, AGENT.md, ADR index
- 🔄 **P3**: Render-World Hardening - Replace placeholder entity-spawn queue with real PhaseItems
- 🔄 **P3**: Config System Concurrency - Make ConfigLoader thread-safe (Send + Sync)

**Previous Sprints Complete:**
- ✅ Sprint 1-2: Data-driven foundations with config system and entity factory
- ✅ Sprint 3: Core gameplay & physics integration with vehicle simulation
- ✅ Sprint 4: Performance benchmarks and config file porting
- ✅ Sprint 5: Rendering & performance optimization with batch processing and LOD

**Strategic Plan**: [STRATEGIC_RESTORATION_PLAN.md](docs/STRATEGIC_RESTORATION_PLAN.md)

## Performance Targets

- **60+ FPS** on desktop platforms
- **Distance-based culling** for open world streaming
- **Object pooling** and memory efficiency
- **Bevy's parallel ECS** for system execution

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
