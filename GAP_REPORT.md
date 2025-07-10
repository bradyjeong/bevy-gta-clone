# GAP REPORT: f430bc6 Feature Restoration

## Overview
This document maps the comprehensive game features from commit f430bc6 to the current Bevy 0.16.1 architecture, identifying implementation gaps and restoration strategies.

## Current Architecture Status
- **Bevy Version**: 0.16.1 (fully aligned)
- **Crate Structure**: 5-crate strategic modularity
- **Test Coverage**: 122 tests passing (18+39+40+37+18)
- **Migration Status**: ADR-0007 complete, clean foundation

## f430bc6 Feature Analysis

### 1. Configuration System (12 RON Files)
**f430bc6 Implementation**: Complete data-driven configuration system
- `assets/config/audio_settings.ron`
- `assets/config/camera_settings.ron`
- `assets/config/game_config.ron`
- `assets/config/lod_config.ron`
- `assets/config/npc_behavior.ron`
- `assets/config/performance_config.ron`
- `assets/config/performance_settings.ron`
- `assets/config/performance_tuning.ron`
- `assets/config/physics_constants.ron`
- `assets/config/ui_settings.ron`
- `assets/config/vehicle_physics.ron`
- `assets/config/vehicle_stats.ron`
- `assets/config/visual_effects.ron`
- `assets/config/world_generation.ron`

**Current Status**: ❌ **MISSING**
- No assets/ directory
- config_core exists but lacks RON asset pipeline

**Restoration Strategy**:
- **Target Crate**: `config_core`
- **Implementation**: Bevy AssetLoader for RON configs
- **Integration**: Use amp_engine's asset pipeline
- **Hot Reload**: Behind "hot-reload" feature flag

### 2. Entity Factory System
**f430bc6 Implementation**: Unified entity creation system
- `src/factories/entity_factory_unified.rs`
- Single-source-of-truth for all entity spawning
- Data-driven prefab definitions

**Current Status**: ❌ **MISSING**
- gameplay_factory has basic registry but no unified system
- No prefab definitions or spawn logic

**Restoration Strategy**:
- **Target Crate**: `gameplay_factory`
- **Implementation**: Expand prefab system with bevy_reflect
- **Integration**: Use config_core for prefab data loading
- **Benchmarks**: Target ≤1.2× legacy spawn time for 100k entities

### 3. Advanced Vehicle Physics
**f430bc6 Implementation**: Comprehensive vehicle systems
- `src/systems/movement/realistic_vehicle_physics.rs`
- `src/systems/movement/realistic_vehicle_physics_core.rs`
- `src/systems/movement/supercar_effects.rs`
- `src/systems/movement/supercar_input.rs`
- `src/systems/movement/supercar_physics.rs`
- `src/systems/movement/vehicle_sets.rs`

**Current Status**: ❌ **MISSING**
- No vehicle physics implementation
- No movement systems

**Restoration Strategy**:
- **Target Crate**: New `amp_gameplay` crate
- **Implementation**: Port to Bevy 0.16.1 + bevy_rapier3d 0.26
- **Integration**: Use amp_engine for ECS integration
- **Features**: Realistic physics, supercar effects, input handling

### 4. Audio System
**f430bc6 Implementation**: Advanced audio graph
- `src/systems/audio/realistic_vehicle_audio.rs`
- Advanced audio processing and effects

**Current Status**: ❌ **MISSING**
- No audio system implementation
- No audio dependencies

**Restoration Strategy**:
- **Target Crate**: `amp_engine`
- **Implementation**: Plugin-based audio system with bevy_kira_audio
- **Integration**: Connect to vehicle physics for reactive audio
- **Dependencies**: Add bevy_kira_audio to workspace dependencies

### 5. Level of Detail (LOD) System
**f430bc6 Implementation**: Professional LOD management
- `src/systems/lod/modern_lod_system.rs`
- `src/components/lod.rs`
- Distance-based quality management

**Current Status**: ❌ **MISSING**
- No LOD system implementation
- No distance-based quality controls

**Restoration Strategy**:
- **Target Crate**: `amp_engine`
- **Implementation**: Distance-based LOD with bevy_pbr integration
- **Integration**: Use amp_math for distance calculations
- **Components**: LOD component with quality levels

### 6. Batch Processing System
**f430bc6 Implementation**: Modern parallel job system
- `src/systems/batching.rs`
- `src/plugins/batching_plugin.rs`
- 300%+ performance improvements

**Current Status**: ❌ **MISSING**
- No batch processing implementation
- No performance optimizations

**Restoration Strategy**:
- **Target Crate**: `amp_engine`
- **Implementation**: Bevy RenderWorld phases for batch processing
- **Integration**: Compute shader optimization behind "gpu" feature
- **Benchmarks**: Target ≥2.5× speed improvement on reference scene

### 7. Rendering & Culling
**f430bc6 Implementation**: GPU-ready culling system
- GPU-prepared culling infrastructure
- Instanced vegetation rendering
- `src/systems/rendering/vegetation_instancing.rs`
- `src/components/instanced_vegetation.rs`

**Current Status**: ❌ **MISSING**
- No culling implementation
- No instanced rendering

**Restoration Strategy**:
- **Target Crate**: `amp_engine`
- **Implementation**: Compute shader instance culling
- **Integration**: Feature-flagged "gpu" with CPU fallback
- **Benchmarks**: Target ≤0.3ms/frame culling time

### 8. World Generation & Content
**f430bc6 Implementation**: Dynamic world systems
- `src/systems/world/dynamic_content.rs`
- `src/systems/world/layered_generation.rs`
- `src/systems/world/unified_factory_setup.rs`

**Current Status**: ❌ **MISSING**
- No world generation systems
- No dynamic content loading

**Restoration Strategy**:
- **Target Crate**: `amp_gameplay`
- **Implementation**: Bevy-native world generation
- **Integration**: Use config_core for world parameters
- **Features**: Layered generation, dynamic content streaming

### 9. Plugin Architecture
**f430bc6 Implementation**: Professional plugin system
- `src/plugins/game_plugin.rs`
- `src/plugins/vehicle_plugin.rs`
- Comprehensive plugin architecture

**Current Status**: ⚠️ **PARTIAL**
- amp_engine has basic plugin structure
- No game-specific plugins

**Restoration Strategy**:
- **Target Crate**: `amp_engine` + `amp_gameplay`
- **Implementation**: PluginGroup::AAAPlugins wrapper
- **Integration**: Bevy-native plugin system
- **Features**: Game, vehicle, audio, rendering plugins

### 10. Configuration Loading
**f430bc6 Implementation**: Advanced config loading
- `src/systems/config_loader.rs`
- `src/config/` module structure
- Hot-reload capable

**Current Status**: ⚠️ **PARTIAL**
- config_core has basic loading
- No hot-reload system

**Restoration Strategy**:
- **Target Crate**: `config_core`
- **Implementation**: Bevy AssetLoader with hot-reload
- **Integration**: Use amp_engine asset pipeline
- **Features**: Watch-based hot reload, validation

## Implementation Priority Matrix

### Sprint 1-2: Foundations
1. **Config System** (High Impact, Medium Complexity)
2. **Entity Factory** (High Impact, Medium Complexity)

### Sprint 3-4: Core Gameplay
3. **Vehicle Physics** (High Impact, High Complexity)
4. **Audio System** (Medium Impact, Medium Complexity)

### Sprint 5-6: Performance
5. **Batch Processing** (High Impact, High Complexity)
6. **LOD System** (Medium Impact, Medium Complexity)

### Sprint 7-8: Advanced Features
7. **GPU Culling** (High Impact, High Complexity)
8. **World Generation** (Medium Impact, High Complexity)

### Sprint 9-12: Polish & Integration
9. **Plugin Architecture** (Low Impact, Low Complexity)
10. **Rendering Systems** (Medium Impact, Medium Complexity)

## Risk Assessment

### High Risk
- **GPU Culling**: Compute shader compatibility across hardware
- **Vehicle Physics**: Determinism with bevy_rapier3d
- **Performance Gates**: Meeting 300%+ improvement claims

### Medium Risk
- **Asset Pipeline**: Hot-reload integration with Bevy
- **Config System**: RON format compatibility
- **Audio Integration**: bevy_kira_audio version compatibility

### Low Risk
- **Entity Factory**: Well-understood ECS patterns
- **Plugin Architecture**: Standard Bevy plugin system
- **Documentation**: Straightforward doc updates

## Success Metrics

### Technical Metrics
- **Test Coverage**: Maintain 122 existing + add ≥40 new tests
- **Performance**: 60 FPS @1080p, <1GB memory, spawn_100k ≤3ms
- **Build Time**: Full workspace build <20 seconds
- **Coverage**: ≥75% overall test coverage

### Quality Metrics
- **Lint**: Clippy -D warnings clean
- **Format**: rustfmt --check passing
- **Documentation**: rustdoc -D warnings clean
- **Examples**: All examples compile and run

### Professional Metrics
- **Feature Parity**: All f430bc6 features restored
- **Architecture**: 5-crate structure preserved
- **Testing**: Green bar guarantee throughout
- **Release**: ADR-0008 + v0.4.0-alpha tag

## Next Steps

1. **Create `restore/f430bc6` branch**
2. **Set up git worktree for f430bc6 reference**
3. **Begin Sprint 1: Config System implementation**
4. **Establish CI benchmarks and performance gates**
5. **Weekly Oracle consultations for strategic guidance**

---
*This document will be updated as restoration progresses and new gaps are identified.*
