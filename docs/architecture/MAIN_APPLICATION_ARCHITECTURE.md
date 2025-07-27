# GTA4-Style Main Application Architecture

## Overview

This document describes the comprehensive main application that integrates all implemented f430bc6 systems into a playable GTA4-style open-world game. Due to compilation issues with amp_engine crate dependencies, this serves as the architectural specification.

## Current Working Demo

**Use the existing working demo:** [`city_demo_baseline`](../examples/city_demo_baseline.rs)

```bash
cargo run --example city_demo_baseline --features rapier3d_030
```

This demonstrates the core systems working together with vehicle physics, audio, and gameplay.

## Planned Main Application Features

### 🎮 Game States & Architecture

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,     // Start game, settings, exit
    InGame,       // Open world gameplay
    Paused,       // Resume, settings, main menu
    Settings,     // Graphics, audio, controls
}
```

### 🌍 Systems Integration

#### **amp_core**: Foundation
- Error handling with comprehensive `Result<T>` types
- Memory allocation tracking
- Performance profiling infrastructure

#### **amp_math**: Spatial Systems
- Morton encoding for spatial indexing (3D → 1D mapping)
- Distance caching system (600% performance improvement)
- AABB and bounding volume calculations
- Chunk key management for world streaming

#### **amp_gameplay**: Core Game Logic
- **Vehicle Physics**: Realistic suspension, steering, drivetrain
- **NPC System**: AI-driven characters with distance-based optimization
- **Audio Integration**: 3D positional audio with engine sounds
- **Input Handling**: Vehicle controls and player movement

#### **gameplay_factory**: Entity Management
- Prefab-based entity spawning system
- **Performance**: 1.61ms for 100k entities (optimized)
- Support for vehicles, buildings, characters
- Hot-reloadable configuration

#### **amp_render** (when integrated): Rendering Pipeline
- GPU culling with compute shaders (<0.25ms target)
- Distance-based LOD system
- Batch processing for render optimization
- Multi-draw indirect rendering

### 🎯 Performance Targets

| System | Target Performance |
|--------|-------------------|
| **Frame Rate** | 60+ FPS stable @1080p |
| **Entity Spawning** | 1.61ms for 100k entities |
| **GPU Culling** | <0.25ms per frame |
| **World Streaming** | <0.5ms per streaming pass |
| **Memory Profile** | Flat with object pools |

### 🎮 Game Features

#### **Open World System**
- **World Streaming**: Chunk-based loading (500m chunks, 1km radius)
- **Dynamic LOD**: Distance-based quality management
- **Performance Scaling**: Automatic optimization based on frame rate

#### **Vehicle System**
- **Realistic Physics**: Suspension, tire friction, engine simulation
- **Multiple Vehicle Types**: Sports cars, sedans, trucks
- **Enter/Exit Mechanics**: Seamless player-vehicle interaction
- **Audio Integration**: RPM-based engine sounds with 3D positioning

#### **NPC System**
- **AI Behavior**: State machines with idle, patrol, chase, flee
- **Distance Optimization**: Tick rate scaling based on player distance
- **Dynamic Spawning**: Procedural generation around player
- **Performance Management**: Automatic NPC culling and management

#### **UI System**
- **HUD**: Health, stamina, money, wanted level
- **Minimap**: Real-time player position and surroundings
- **Performance Overlay**: FPS, frame time, entity count (F2)
- **Debug Info**: Chunk loading, NPC count, system metrics (F1)

### 🎮 Controls & Interaction

```
Movement Controls:
  WASD          - Player movement / Vehicle controls
  Mouse         - Camera rotation
  E             - Enter/Exit vehicle
  Space         - Jump / Handbrake
  C             - Toggle camera mode (1st/3rd person)
  F             - Toggle camera follow

UI Controls:
  F1            - Toggle debug overlay
  F2            - Toggle performance stats
  F3            - Settings menu
  TAB           - Minimap toggle
  ESC           - Pause menu
  F11           - Fullscreen toggle
```

### 🏗️ Implementation Status

#### ✅ **Completed Systems**
- Vehicle physics with realistic simulation
- NPC AI with distance-based optimization
- Audio system with 3D positioning
- Prefab factory with optimized spawning
- Distance caching with 600% performance gain
- World streaming foundation

#### 🚧 **Integration Challenges**
- amp_engine compilation issues prevent full integration
- Dependencies between crates create circular references
- GPU rendering pipeline needs Bevy 0.16.1 compatibility fixes

#### 🎯 **Recommended Approach**

1. **Use Current Demo**: [`city_demo_baseline`](../examples/city_demo_baseline.rs) showcases working systems
2. **Fix amp_engine**: Resolve compilation errors in amp_engine crate
3. **Gradual Integration**: Add systems incrementally to working baseline
4. **Performance Validation**: Ensure each system meets performance targets

### 📊 Performance Metrics

The system tracks comprehensive performance metrics:

```rust
#[derive(Resource, Debug)]
struct PerformanceMonitor {
    fps_history: Vec<f32>,           // Rolling 60-frame FPS history
    frame_time_history: Vec<f32>,    // Frame time tracking
    entity_count: u32,               // Total entities in world
    active_chunks: usize,            // World streaming chunks
    npc_count: u32,                  // Active NPC entities
    memory_usage: f32,               // Memory consumption (MB)
}
```

### 🎮 Game Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Input Layer   │    │   Game States   │    │   UI System     │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • Player Input  │    │ • MainMenu      │    │ • HUD           │
│ • Vehicle Input │    │ • InGame        │    │ • Minimap       │
│ • Camera Input  │    │ • Paused        │    │ • Performance   │
│ • UI Input      │    │ • Settings      │    │ • Debug         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                     Core Game Systems                           │
├─────────────────┬─────────────────┬─────────────────┬───────────┤
│  Physics        │  Gameplay       │  Audio          │  Render   │
├─────────────────┼─────────────────┼─────────────────┼───────────┤
│ • Vehicles      │ • NPCs          │ • Engine Sounds │ • LOD     │
│ • Collisions    │ • World Stream  │ • 3D Audio      │ • Culling │
│ • Rigid Bodies  │ • Prefabs       │ • Ambient       │ • Batching│
└─────────────────┴─────────────────┴─────────────────┴───────────┘
         │                       │                       │
┌─────────────────────────────────────────────────────────────────┐
│                     Foundation Layer                            │
├─────────────────┬─────────────────┬─────────────────┬───────────┤
│  amp_core       │  amp_math       │  amp_engine     │ Bevy ECS  │
├─────────────────┼─────────────────┼─────────────────┼───────────┤
│ • Error Handle  │ • Morton Code   │ • Memory Pools  │ • Systems │
│ • Allocation    │ • Distance Cache│ • World Stream  │ • Events  │
│ • Profiling     │ • Spatial Index │ • Batch Process │ • Resources│
└─────────────────┴─────────────────┴─────────────────┴───────────┘
```

### 🚀 Next Steps

1. **Build Current Demo**: `cargo run --example city_demo_baseline --features rapier3d_030`
2. **Fix Dependencies**: Resolve amp_engine compilation issues  
3. **Add Game States**: Implement menu system and state transitions
4. **Integrate Rendering**: Add GPU culling and batch processing
5. **Polish UI**: Complete HUD, minimap, and settings system
6. **Performance Testing**: Validate against all performance targets

This architecture provides a comprehensive foundation for a professional AAA-quality open-world game while maintaining the performance and modularity principles established in the f430bc6 systems.
