# Amp Game Engine

![CI Status](https://github.com/bradyjeong/bevy-gta-clone/workflows/CI/badge.svg)
![Test Coverage](https://codecov.io/gh/bradyjeong/bevy-gta-clone/branch/main/graph/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.85+-blue.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

A professional AAA-level open world game built with Bevy 0.16.1 and Rust 2024, featuring comprehensive game systems and optimized for Amp development workflows.

## ✅ ADR-0007 COMPLETE → 🎯 AAA-RESTORATION PHASE ACTIVE

**Oracle-guided architecture migration from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity + version consistency.**

**Current Status**: 122 tests passing, Oracle version consistency guards active, foundation ready for feature restoration.

**Now Active**: **12-Week AAA Restoration Plan** to restore f430bc6 "REVOLUTIONARY TRANSFORMATION" features to current Bevy 0.16.1 architecture.

### 🚀 Target Feature Set (f430bc6 → Bevy 0.16.1)
- **12 RON Configuration System**: Data-driven game tuning with hot-reload
- **Advanced Vehicle Physics**: Realistic drivetrain, suspension, tire friction curves
- **Professional LOD System**: Distance-based mesh and material swapping
- **GPU Culling & Batch Processing**: Compute shader optimization with multi-draw-indirect
- **Modern ECS Patterns**: SystemSets, QueryData, sparse storage optimization
- **Audio Graph**: Advanced audio system with bevy_kira_audio integration
- **Performance Claims**: 300%+ FPS improvement, 60% memory reduction

**Strategy**: [STRATEGIC_RESTORATION_PLAN.md](docs/STRATEGIC_RESTORATION_PLAN.md) - Oracle's 12-week roadmap

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

**ADR-0007 Migration Complete:**
- ✅ Oracle consultation complete
- ✅ Strategic 6-crate architecture implemented  
- ✅ Bevy 0.16.1 ecosystem alignment
- ✅ Version consistency guards active
- ✅ 180+ tests passing across all crates
- ✅ Foundation ready for feature restoration

**Sprint 2 Complete - Vehicle Physics Foundation:**
- ✅ Professional-grade vehicle physics system implemented
- ✅ amp_physics crate with comprehensive physics simulation
- ✅ Rapier3D integration for collision detection
- ✅ city_demo_baseline example with drivable car
- ✅ 60+ FPS stable with vehicle simulation

**Sprint 3 Active - Core Gameplay & Physics Integration:**
- 🎯 Port vehicle physics to amp_gameplay crate
- 🎯 Advanced audio system with bevy_kira_audio integration
- 🎯 Complete physics integration with bevy_rapier3d 0.30
- 🎯 Enhanced city_demo with audio and integrated physics
- 📋 Strategic plan: [STRATEGIC_RESTORATION_PLAN.md](docs/STRATEGIC_RESTORATION_PLAN.md)

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
