# Architecture Overview

## Vision

The Amp game engine is designed as a AAA-level open world game engine built with Rust and Bevy, optimized for Amp development workflows. The architecture follows a clean, modular design with clear domain boundaries.

## Crate Structure

```
amp_game/
├── crates/
│   ├── amp_core/           # Core error handling and utilities
│   ├── amp_math/           # Spatial mathematics and Morton encoding
│   ├── amp_spatial/        # Hierarchical spatial partitioning
│   ├── amp_gpu/            # GPU abstraction over wgpu
│   ├── amp_world/          # ECS world management (Future)
│   ├── amp_physics/        # Physics integration (Future)
│   ├── amp_ai/             # AI systems (Future)
│   └── amp_render/         # Rendering pipeline (Future)
├── examples/               # Example applications
├── tools/                  # Development tools
└── tests/                  # Integration tests
```

## Dependency Graph

```
amp_core
  ↑
amp_math
  ↑
amp_spatial
  ↑
amp_gpu ← amp_world ← amp_physics ← amp_ai ← amp_render
```

## Key Principles

### 1. Domain Boundaries
- **Engine Layer**: Pure Rust, no Bevy dependencies (amp_core, amp_math, amp_spatial)
- **Adapter Layer**: Thin wrappers around Bevy (amp_world, amp_gpu)
- **Game Layer**: High-level game systems (amp_physics, amp_ai, amp_render)

### 2. Performance Focus
- **Morton Encoding**: Efficient spatial indexing for world streaming
- **Hierarchical LOD**: Distance-based quality management
- **GPU-Driven Rendering**: Prepare for compute shader implementation
- **Memory Management**: Object pools and per-frame arenas

### 3. Amp Optimization
- **Fast Compilation**: Minimal dependencies, parallel builds
- **Clear Interfaces**: Well-defined public APIs
- **Test Coverage**: Comprehensive unit and integration tests
- **Documentation**: All public APIs documented

## Technical Systems

### Spatial Systems (amp_spatial)
- **RegionId**: Morton-encoded spatial identifiers
- **Hierarchical Clipmap**: Multi-level detail management
- **Streaming Provider**: Async region loading interface

### GPU Systems (amp_gpu)
- **Context Management**: wgpu device and queue abstraction
- **Surface Management**: Swapchain and render pass handling
- **Shader Pipeline**: Future compute shader support

### Mathematics (amp_math)
- **Morton Encoding**: 3D spatial indexing
- **Bounds**: AABB and Sphere implementations
- **Transform Utilities**: Matrix operations and camera math

## Future Architecture

### World Streaming System
- Seamless world loading/unloading
- Memory-efficient chunk management
- Distance-based LOD transitions

### Physics Integration
- Rapier3D integration with custom vehicle physics
- Deterministic simulation for networking
- Efficient collision detection

### AI Systems
- Hierarchical pathfinding
- Behavior trees for NPCs
- Traffic simulation

### Rendering Pipeline
- GPU-driven culling
- Batched instance rendering
- Modern lighting and post-processing

## Development Workflow

### Weekly Milestones
- **Week 1**: Foundation (Current)
- **Week 2**: Config and Entity Factory
- **Week 3**: LOD and Batch Systems
- **Week 4**: Physics Integration
- **Week 5**: AI Systems
- **Week 6**: Streaming Implementation
- **Week 7**: Vertical Slice
- **Week 8**: Performance and Polish

### Quality Gates
- **Compilation**: All crates must compile with `-Dwarnings`
- **Testing**: 70%+ test coverage
- **Performance**: 60+ FPS target
- **Documentation**: All public APIs documented

## Crate Details

### amp_core
Core error handling and shared utilities used across all crates.

### amp_math
High-performance spatial mathematics with SIMD support and Morton encoding for efficient spatial indexing.

### amp_spatial
Hierarchical spatial partitioning system with async streaming capabilities for large open worlds.

### amp_gpu
Low-level GPU abstraction providing a clean interface over wgpu with support for modern rendering techniques.

### config_core
Configuration loading and management system with hierarchical file search and RON deserialization.

## Architecture Documentation

- **[Configuration System](config.md)**: Hierarchical configuration loading with RON format support

## Success Metrics

- **Build Time**: < 5 minutes for full workspace
- **Test Coverage**: ≥ 70% excluding binaries
- **Performance**: 60+ FPS on desktop
- **Memory Usage**: Efficient allocation patterns
- **Developer Experience**: Fast iteration cycles
