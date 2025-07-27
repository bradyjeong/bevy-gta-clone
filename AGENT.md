# AGENT.md

## Shell Compatibility (zsh)
- **No timeout command**: Use `cargo run` directly instead of `timeout 10s cargo run` (timeout not available on macOS)
- **Use cargo watch**: For timed builds use `cargo watch -c` for continuous compilation
- **Process control**: Use Ctrl+C to stop long-running processes instead of timeout

## Commands
- Build: `cargo build --workspace` | Check: `cargo check --workspace` | Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets --all-features` | Format: `cargo fmt --all`
- Rustdoc: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- Coverage: `cargo llvm-cov --workspace --all-features` | Coverage Gate: Minimum 80%
- Performance: `cargo bench -p gameplay_factory --bench factory_spawn spawn_100k_optimized`
- Memory Leak Tests: `cargo test --workspace --all-features -- --ignored long_memory` (weekly CI only)
- Dev Tools: `cargo xtask ci` (full CI pipeline), `./scripts/pre-commit-check.sh` (before commits)
- Performance: `cargo xtask perf` (JSON output), `cargo xtask perf --gpu-culling` (with GPU features)
- Facade Testing: `cargo test -p amp_foundation -p amp_game` | `cargo doc -p amp_foundation -p amp_game`

## Development Workflow
- **During Development**: `cargo watch -c` (continuous compilation)
- **Before Committing**: `./scripts/pre-commit-check.sh` (full CI simulation)
- **Auto-format**: `cargo fmt --all` (run frequently)
- **Golden Rule**: Never commit without running pre-commit checks

## Oracle's Version Consistency Guard
- **Pre-commit Hook**: Automatically runs `./scripts/pre-commit-check.sh` before every commit
- **CI Guard**: GitHub Actions runs Oracle's version consistency checks on every push
- **Setup**: `git config core.hooksPath .githooks` (run once per clone)
- **Purpose**: Prevents the ADR-0007 version consistency violation from happening again

## Project Vision
**AAA-Level Open World Game** - GTA-style game built with Bevy 0.16.1 using Rust 2021 edition
- **Target**: Professional game development with Amp-optimized workflow
- **Focus**: Ecosystem alignment, fast iteration, clear boundaries

## Architecture Strategy
**Oracle-Guided Strategic Shift** - Bevy 0.16.1 + Facade Crate Strategy + Strategic 10-Crate Structure + Version Consistency

### Current Workspace Structure
```
├─ crates/
│   ├─ amp_foundation/    # Facade: Core + Math (no Bevy deps, engine builders)
│   ├─ amp_game/          # Facade: Complete game development interface
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_physics/       # Physics systems and components
│   ├─ amp_render/        # Rendering systems, GPU culling, LOD
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   ├─ config_core/       # Configuration management and assets
│   └─ gameplay_factory/  # Entity spawning and factory systems
├─ examples/              # Example applications (city_demo.rs)
├─ docs/adr/              # Architecture Decision Records
└─ .github/workflows/     # CI/CD pipeline
```

### Key Principles
- **Ecosystem Alignment**: Use full Bevy 0.16.1, don't fight the ecosystem
- **Strategic Modularity**: 8 crates with clear domain boundaries
- **Facade Strategy**: Simplified entry points (amp_foundation for engine builders, amp_game for game developers)
- **Version Consistency**: Single source of truth for all versions in [workspace.dependencies]
- **Amp Optimized**: Focused surfaces for parallel agent development
- **Compile Speed**: Incremental builds, minimal cross-crate dependencies

## Development Workflow
- **Weekly Checkpoints**: Prevent scope creep with deliverable demos
- **60 FPS Target**: Performance gates at each milestone
- **Test Coverage**: Comprehensive test suite across all crates
- **CI Time**: Full workspace build <20 seconds
- **PR Size**: ≤500 LOC per merge

## Code Style
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas

## Performance Targets & Gates
- **Target**: 60+ FPS on desktop, stable frame times
- **Performance Gates** (enforced in nightly CI):
  - P95 frame time ≤ 16.6ms (60 FPS gate)
  - P99 frame time ≤ 33.3ms 
  - Average frame time ≤ 8.3ms
  - GPU culling time ≤ 0.25ms (when enabled)
  - Coverage threshold: ≥80% line coverage (aligned between xtask and CI)
- **Culling**: Distance-based (buildings 300m, vehicles 150m, NPCs 100m)
- **Memory**: Object pools, per-frame arenas, minimal allocations
- **Profiling**: Built-in counters, frame analysis, bottleneck detection
- **Performance Testing**: `cargo xtask perf --format json` outputs structured metrics for CI gates

## Technical Systems Implemented
### amp_core
- Engine-wide error handling with thiserror
- Result<T> alias for consistent error handling
- Comprehensive unit test coverage

### amp_math
- Morton 3D encoding for efficient spatial indexing
- AABB and Sphere bounding volume implementations
- Transform utilities with builder patterns
- Comprehensive unit test coverage

### amp_engine
- Bevy-integrated engine systems
- Spatial partitioning with Morton encoding
- GPU abstraction and world management
- Plugin-based architecture

### amp_physics
- Professional vehicle physics system
- Suspension, engine, transmission systems
- Rapier3D integration for collision detection
- Comprehensive physics simulation

### amp_render
- Advanced rendering pipeline with Bevy integration
- GPU culling and LOD systems
- Distance-based optimization
- Memory-efficient rendering

### amp_gameplay
- Core gameplay systems and components
- Character and NPC behavior systems
- Vehicle integration and city systems
- Audio and interaction systems

### config_core
- Configuration management with RON assets
- Hot-reload capability
- Type-safe configuration validation

### gameplay_factory
- Entity spawning and factory systems
- Performance-optimized entity creation
- Configurable spawn parameters

## Current Status
⚠️ **COMPILATION ISSUES** - Sprint 9 Implementation Needs Debugging
- **Status**: **IN PROGRESS** - Compilation errors in amp_gameplay crate need resolution
- **Architecture**: 8-crate structure with Bevy 0.16.1 ecosystem alignment + rendering pipeline
- **Active Crates**: amp_core, amp_math, amp_engine, amp_physics, amp_render, amp_gameplay, config_core, gameplay_factory
- **Issues Identified**:
  - Velocity component conflicts between bevy_rapier3d and custom physics
  - Missing imports for VisualCharacter components
  - Test function visibility issues in NPC systems
  - Bevy API changes (app.world vs app.world())

✅ **SPRINT 9 ARCHITECTURE** - Optimization & Polishing Phase Framework
- **Objective**: Final optimization, performance tuning, and polish for AAA-grade release
- **Framework Status**: **IMPLEMENTED** - All Sprint 9 architectural foundations completed
- **Key Infrastructure**:
  - GPU culling pipeline with feature flags
  - Performance benchmarking with CI integration
  - Memory optimization infrastructure
  - Comprehensive test framework

## Oracle Guidance
- **Strategic Decisions**: Documented in [Oracle Consultations](docs/oracle-consultations.md)
- **Architecture Decisions**: Captured in [ADR system](docs/adr/README.md) - immutable records of major architectural choices
- **Current ADR Status**: ADR-0010 (Facade Crate Strategy) active, ADR-0008 (AAA Feature Restoration) completed, ADR-0007 (Bevy 0.16.1 Migration) completed
- **Key Principle**: Follow Oracle's strategic shift to Bevy 0.16.1 strictly
- **Version Consistency**: Follow Oracle's version-consistency strategy exactly
- **Weekly Verification**: Consult Oracle for milestone checkpoints

## Version Consistency Strategy
**Oracle's Lock-in Rules:**
- **Engine nucleus**: `bevy = "=0.16.1"` (patch-locked)
- **Ecosystem sidekicks**: `bevy_rapier3d = "=0.30.0"` (patch-locked)
- **Rendering**: `wgpu = "=0.21.0"`, `winit = "=0.30.0"` (via [patch.crates-io])
- **Mature crates**: `serde = "^1"`, `anyhow = "^1.0"` (caret-semver)
- **Single source**: All versions in [workspace.dependencies]
- **Workspace inheritance**: Individual crates use `workspace = true`

**Version-Bump Playbook:**
1. Wait for new Bevy release announcement
2. Branch `upgrade/bevy-X.Y.Z`
3. Update only `[workspace.dependencies].bevy = "=X.Y.Z"`
4. Run `cargo update -p bevy --precise X.Y.Z`
5. Update `[patch.crates-io]` with exact wgpu/winit versions
6. Run full CI, address breaking changes, merge

## Immediate Action Items
### Critical Issues to Address
1. **Velocity Component Conflicts**: Resolve ambiguous imports between bevy_rapier3d::Velocity and custom physics::Velocity
2. **Missing Character Visual Components**: Add proper imports for VisualCharacter and CharacterVisualConfig
3. **NPC Test Function Visibility**: Make private functions in npc::systems public for testing or refactor tests
4. **Bevy API Updates**: Update from deprecated `app.world` to `app.world()` method calls

### Next Sprint Planning
- **Priority 1**: Fix all compilation errors and restore working build
- **Priority 2**: Update test suite to reflect current architecture
- **Priority 3**: Validate performance benchmarks work with current codebase
- **Priority 4**: Complete Sprint 9 optimization objectives

## Quality Gates & Metrics
- **Unit Tests**: ≥80% coverage overall (llvm-cov gate)
- **Integration Tests**: All tests passing across all crates
- **Performance**: spawn_100k ≤3ms, frustum cull ≤0.3ms/frame
- **Lint/Format**: Clippy -D warnings, rustfmt --check
- **Documentation**: rustdoc -D warnings, examples compile
- **Build Status**: All crates must compile without errors

## Maintenance Schedule
- **Every commit**: Verify AGENT.md status reflects reality
- **Every architectural change**: Update all docs in this list
- **Every Oracle consultation**: Update oracle-consultations.md + create ADR if needed
- **Every milestone**: Verify README.md features match implementation
- **Weekly**: Run full test suite and performance benchmarks
