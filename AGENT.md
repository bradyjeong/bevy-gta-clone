# AGENT.md

## Commands
- Build: `cargo build --workspace` | Check: `cargo check --workspace` | Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets --all-features` | Format: `cargo fmt --all`
- Rustdoc: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- Coverage: `cargo llvm-cov --workspace --all-features` | Coverage Gate: Minimum 70%
- Run Example: `cargo run --example city_demo`
- Dev Tools: `cargo xtask ci` (full CI pipeline), `./scripts/pre-commit-check.sh` (before commits)

## Development Workflow
- **During Development**: `cargo watch -c` (continuous compilation)
- **Before Committing**: `./scripts/pre-commit-check.sh` (full CI simulation)
- **Auto-format**: `cargo fmt --all` (run frequently)
- **Golden Rule**: Never commit without running pre-commit checks

## Project Vision
**AAA-Level Open World Game** - GTA-style game built with Bevy 0.16.1 using Rust 2024 edition
- **Target**: Professional game development with Amp-optimized workflow
- **Focus**: Ecosystem alignment, fast iteration, clear boundaries

## Architecture Strategy
**Oracle-Guided Strategic Shift** - Bevy 0.16.1 + Strategic 4-5 Crate Structure + Version Consistency

### Oracle's Strategic Workspace Structure
```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ amp_gameplay/      # Game systems, components, prefabs
│   └─ amp_tools/         # xtask, build pipeline helpers (optional)
├─ examples/              # Example applications (city_demo.rs)
├─ docs/adr/              # Architecture Decision Records
└─ .github/workflows/     # CI/CD pipeline
```

### Key Principles
- **Ecosystem Alignment**: Use full Bevy 0.16.1, don't fight the ecosystem
- **Strategic Modularity**: 4-5 crates max, clear domain boundaries
- **Version Consistency**: Single source of truth for all versions in [workspace.dependencies]
- **Amp Optimized**: Focused surfaces for parallel agent development
- **Compile Speed**: Incremental builds, minimal cross-crate dependencies

## Development Workflow
- **Weekly Checkpoints**: Prevent scope creep with deliverable demos
- **60 FPS Target**: Performance gates at each milestone
- **Test Coverage**: 218 tests passing, comprehensive coverage
- **CI Time**: Full workspace build <20 seconds
- **PR Size**: ≤500 LOC per merge

## Code Style
- snake_case for variables/functions, PascalCase for structs/components
- Import order: external crates, std, bevy prelude, local crate
- Prefer `if let`/`match` over unwrap(), use Bevy constants (Vec3::ZERO)
- Components: `#[derive(Component)]` + `Default`, systems in subdirs
- Safety: Validate physics values, clamp positions/dimensions, use collision groups
- Comments: `//` style, 4-space indent, trailing commas

## Performance Targets
- **Target**: 60+ FPS on desktop, stable frame times
- **Culling**: Distance-based (buildings 300m, vehicles 150m, NPCs 100m)
- **Memory**: Object pools, per-frame arenas, minimal allocations
- **Profiling**: Built-in counters, frame analysis, bottleneck detection

## Technical Systems Implemented
### amp_core
- Engine-wide error handling with thiserror
- Result<T> alias for consistent error handling
- 11 unit tests covering all error variants

### amp_math
- Morton 3D encoding for efficient spatial indexing
- AABB and Sphere bounding volume implementations
- Transform utilities with builder patterns
- 40 unit tests with comprehensive coverage

### amp_spatial  
- RegionId with Morton-encoded spatial identifiers
- Hierarchical clipmap for multi-level detail management
- Async streaming provider interface
- 22 unit tests covering all functionality

### amp_gpu
- wgpu context and surface management
- GPU abstraction with error handling
- Purple screen rendering example
- 3 unit tests for core functionality

### amp_world
- Basic ECS world management wrapper
- Future integration point for Bevy systems
- 2 unit tests for world creation

## Current Status
✅ **ADR-0007 MIGRATION COMPLETE** - Oracle-Guided Architecture Change
- **Migration**: Successfully moved from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity
- **Architecture**: 5-crate structure with full Bevy 0.16.1 ecosystem alignment
- **Active Crates**: amp_core, amp_math, amp_engine, config_core, gameplay_factory
- **Deprecated**: amp_spatial, amp_gpu, amp_world (consolidated into amp_engine)
- **Test Status**: 122 tests passing (18+39+40+37+18)

🎯 **AAA-RESTORATION PHASE INITIATED** - Oracle's 12-Week Master Plan
- **Objective**: Restore f430bc6 "REVOLUTIONARY TRANSFORMATION" features to current Bevy 0.16.1 architecture
- **Target**: Professional AAA game development capability
- **Strategy**: Migrate behavior, not code - Re-implement in Bevy 0.16.1 idioms
- **Status**: **SPRINT 0 PREP** - Documentation update, branch creation, gap analysis

## Oracle Guidance
- **Strategic Decisions**: Documented in [Oracle Consultations](docs/oracle-consultations.md)
- **Architecture Decisions**: Captured in [ADR system](docs/adr/README.md)
- **Key Principle**: Follow Oracle's strategic shift to Bevy 0.16.1 strictly
- **Version Consistency**: Follow Oracle's version-consistency strategy exactly
- **Weekly Verification**: Consult Oracle for milestone checkpoints

## Version Consistency Strategy
**Oracle's Lock-in Rules:**
- **Engine nucleus**: `bevy = "=0.16.1"` (patch-locked)
- **Ecosystem sidekicks**: `bevy_rapier3d = "=0.26.0"` (patch-locked)
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

## Maintenance & Live Documentation

### Files Requiring Regular Updates
These files must be kept current and reviewed during every strategic change:

**Core Documentation:**
- `Agent.md` - Commands, architecture, status (THIS FILE)
- `README.md` - Public face, quick start, features  
- `STRATEGIC_SHIFT.md` - Current migration status and roadmap
- `CONTRIBUTING.md` - Development workflow, code style, commit guidelines

**Architecture Records:**
- `docs/adr/README.md` - Index of all architectural decisions
- `docs/oracle-consultations.md` - Oracle guidance and strategic decisions
- Latest ADR (currently ADR-0007) - Active architectural strategy

**Configuration Files:**
- `Cargo.toml` - Workspace dependencies, edition, version
- `examples/Cargo.toml` - Example dependencies and structure  
- `CODEOWNERS` - Ownership aligned with current crate structure
- `.github/workflows/ci.yml` - CI pipeline matching current architecture

**Status Tracking:**
- `IMPLEMENTATION_SUMMARY.md` - Current implementation status
- Test counts and coverage metrics in CI
- Performance benchmarks and targets

### Dead Weight Prevention
**Red Flags for Cleanup:**
- Documentation referencing removed crates (amp_spatial, amp_gpu, amp_world)
- Cargo.toml dependencies not used by any crate
- Examples that don't compile or run
- CI workflows testing non-existent targets
- README features that don't exist
- ADRs marked "Superseded" without clear replacement

**Maintenance Schedule:**
- **Every commit**: Verify Agent.md status reflects reality
- **Every architectural change**: Update all docs in this list
- **Every Oracle consultation**: Update oracle-consultations.md + create ADR if needed
- **Every milestone**: Verify README.md features match implementation

## AAA-Restoration Roadmap (Oracle's 12-Week Master Plan)

### Sprint 0 (Immediate - CURRENT)
- **Branch Creation**: `restore/f430bc6` for feature restoration
- **Gap Analysis**: Create GAP_REPORT.md mapping f430bc6 features to current architecture
- **Reference Setup**: Git worktree for f430bc6 commit access
- **Documentation**: Update all docs to reflect AAA development focus

### Sprint 1-2: Data-Driven Foundations
- **Config System**: Re-create 12 RON configs as Bevy Assets in config_core
- **Entity Factory**: Port legacy factory DSL into gameplay_factory with bevy_reflect
- **Benchmarks**: Target ≤1.2× legacy spawn time for 100k entities

### Sprint 3-4: Core Gameplay & Physics
- **Vehicle Physics**: Port realistic vehicle physics to amp_gameplay crate
- **Audio Systems**: Advanced audio graph with bevy_kira_audio integration
- **Physics Integration**: Replace custom physics with bevy_rapier3d 0.26

### Sprint 5-6: Rendering & Performance
- **Batch Processing**: Bevy RenderWorld phases with 2.5× speed target
- **GPU Culling**: Compute-shader instance culling behind "gpu" feature
- **LOD System**: Distance-based quality management with bevy_pbr integration

### Sprint 7-8: Professional Integration
- **Plugin Architecture**: Wrap subsystems in Bevy Plugins + PluginGroup::AAAPlugins
- **Tooling**: xtask subcommands for demos and benchmarks
- **Service Elimination**: Complete removal of legacy service containers

### Sprint 9-12: Optimization & Release
- **Performance Gates**: 60 FPS @1080p, <1GB memory for city_demo
- **Test Coverage**: ≥75% coverage, 122 existing + 40 new tests
- **Release**: ADR-0008, v0.4.0-alpha tag, crates.io pre-release

## Quality Gates & Metrics
- **Unit Tests**: ≥75% coverage overall (llvm-cov gate)
- **Integration Tests**: All existing 122 + ≥40 new passing
- **Performance**: spawn_100k ≤3ms, frustum cull ≤0.3ms/frame
- **Lint/Format**: Clippy -D warnings, rustfmt --check
- **Documentation**: rustdoc -D warnings, examples compile
