# NPC Behavior System

## Overview

The NPC behavior system provides a comprehensive AI framework for Non-Player Characters in the AAA open-world game. It implements distance-based tick rates, batch processing integration, and a finite state machine for realistic NPC behaviors.

## Key Features

### 🧠 AI Components
- **NPC Component**: Core NPC properties (health, energy, stress, behavior type)
- **NpcState**: Finite state machine for behavior states (Idle, Wander, Flee)
- **NpcBrainHandle**: Batch processing integration with priority queuing
- **LastUpdateFrame**: Frame timing for performance optimization

### 🎯 Distance-Based Tick Rates
- **Close NPCs (<50m)**: Updated every frame for responsive interaction
- **Medium NPCs (50-150m)**: Updated every 15 frames for balanced performance
- **Far NPCs (>150m)**: Updated every 60 frames for maximum optimization

### ⚡ Performance Optimization
- **Batch Processing**: Integrates with `BatchType::AI` (priority 4) for efficient processing
- **Distance Caching**: Uses `DistanceCache` for optimized distance calculations
- **Performance Metrics**: Tracks updates/sec, processing time, and entity counts
- **Target Performance**: ≤2k NPC updates @100k entities (<0.3ms processing time)

### 🔄 Finite State Machine
- **Idle**: Standing still, recovering energy
- **Wander**: Moving randomly, consuming energy
- **Flee**: Running away from threats, high stress
- **Extensible**: Hooks for future behavior trees and advanced AI

## Usage

### Basic Setup

```rust
use amp_gameplay::npc::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NpcPlugin)
        .run();
}
```

### Spawning NPCs

```rust
use amp_gameplay::npc::*;
use gameplay_factory::*;

fn spawn_npc_system(
    mut commands: Commands,
    factory: Res<Factory>,
    config: Res<NpcConfig>,
) {
    // Spawn a civilian NPC
    let npc_entity = factory.spawn_npc(
        &mut commands,
        &config,
        NpcType::Civilian,
        Vec3::new(100.0, 0.0, 50.0),
        42, // unique NPC ID
    ).expect("Failed to spawn NPC");
    
    println!("Spawned NPC: {:?}", npc_entity);
}
```

### Batch Spawning

```rust
use amp_gameplay::npc::*;
use gameplay_factory::*;

fn spawn_multiple_npcs(
    mut commands: Commands,
    factory: Res<Factory>,
    config: Res<NpcConfig>,
) {
    let spawn_requests = vec![
        NpcSpawnRequest {
            npc_id: 1,
            npc_type: NpcType::Civilian,
            position: Vec3::new(0.0, 0.0, 0.0),
        },
        NpcSpawnRequest {
            npc_id: 2,
            npc_type: NpcType::Police,
            position: Vec3::new(50.0, 0.0, 0.0),
        },
    ];
    
    let entities = factory.spawn_npcs_batch(
        &mut commands,
        &config,
        &spawn_requests,
    ).expect("Failed to spawn NPCs");
    
    println!("Spawned {} NPCs", entities.len());
}
```

### Monitoring Performance

```rust
use amp_gameplay::npc::*;

fn monitor_npc_performance(
    metrics: Res<NpcMetrics>,
) {
    println!("NPC Performance Metrics:");
    println!("  Total NPCs: {}", metrics.total_npcs);
    println!("  Updates/sec: {:.2}", metrics.updates_per_second);
    println!("  Processing time: {:.3}ms", metrics.processing_time_ms);
    println!("  Avg time per NPC: {:.6}ms", metrics.avg_processing_time_per_npc);
    println!("  Distance distribution: {:?}", metrics.npcs_by_distance);
    
    // Performance warning
    if metrics.processing_time_ms > 0.3 {
        warn!("NPC processing time exceeds 0.3ms target!");
    }
}
```

## Configuration

### NPC Behavior Configuration

The system loads configuration from `assets/config/npc_behavior.ron`:

```ron
(
    npc_behavior: (
        // Physical properties
        physical: (
            default_height: 1.8,
            mass: 70.0,
            capsule_radius: 0.4,
        ),
        
        // Movement settings
        movement: (
            walk_speed: 1.5,
            run_speed: 3.0,
            max_speed: 5.0,
        ),
        
        // Emotional system
        emotions: (
            energy_levels: (
                max_energy: 100.0,
                tired_threshold: 30.0,
                energetic_threshold: 80.0,
            ),
            stress_levels: (
                max_stress: 100.0,
                panic_threshold: 70.0,
                calm_threshold: 30.0,
            ),
        ),
        
        // AI behavior
        ai: (
            decision_interval: 2.0,
            reaction_time: 0.5,
            memory_duration: 60.0,
        ),
        
        // Distance-based update intervals
        update_intervals: (
            close_distance: 50.0,
            far_distance: 150.0,
            close_interval: 0.0167,    // ~60 FPS
            medium_interval: 0.25,     // ~4 FPS
            far_interval: 1.0,         // ~1 FPS
        ),
    ),
)
```

### Customizing Behavior

```rust
use amp_gameplay::npc::*;

fn customize_npc_behavior(
    mut query: Query<(&mut NPC, &mut NpcState)>,
    config: Res<NpcConfig>,
) {
    for (mut npc, mut state) in query.iter_mut() {
        // Customize NPC properties
        npc.speed = config.npc_behavior.movement.run_speed;
        npc.energy = 50.0;
        
        // Force state transition
        if npc.stress > 80.0 {
            state.current = NpcBehaviorState::Flee;
        }
    }
}
```

## Architecture

### Component Structure

```rust
// Core NPC component
#[derive(Component)]
pub struct NPC {
    pub id: u32,
    pub npc_type: NpcType,
    pub speed: f32,
    pub health: f32,
    pub energy: f32,
    pub stress: f32,
    // ... other fields
}

// State machine
#[derive(Component)]
pub struct NpcState {
    pub current: NpcBehaviorState,
    pub previous: NpcBehaviorState,
    pub state_duration: f32,
    pub state_data: StateData,
}

// Batch processing handle
#[derive(Component)]
pub struct NpcBrainHandle {
    pub batch_handle: u32,
    pub priority: u32,
    pub cost: f32,
    pub distance_to_player: f32,
    pub frames_since_update: u32,
    pub update_interval: u32,
}
```

### State Machine

```rust
pub enum NpcBehaviorState {
    Idle,      // Standing still, recovering energy
    Wander,    // Moving randomly, consuming energy
    Flee,      // Running from threats, high stress
    Follow,    // Future: Following targets
    Interact,  // Future: Interacting with objects
}
```

### System Flow

1. **Distance Calculation**: Use `DistanceCache` to calculate distance to player
2. **Tick Rate Determination**: Set update interval based on distance category
3. **Batch Processing**: Submit NPCs to `BatchController` with `BatchType::AI`
4. **State Evaluation**: Check for state transitions based on conditions
5. **State Execution**: Execute behavior for current state
6. **Metrics Collection**: Track performance and update counts

## Performance Characteristics

### Tick Rate Optimization

| Distance Category | Range | Update Frequency | Performance Impact |
|------------------|-------|------------------|-------------------|
| Close | <50m | Every frame (60 FPS) | High responsiveness |
| Medium | 50-150m | Every 15 frames (4 FPS) | 15x performance improvement |
| Far | >150m | Every 60 frames (1 FPS) | 60x performance improvement |

### Batch Processing

- **Priority**: 4 (lowest priority after Transform, Visibility, Physics, LOD)
- **Cost**: 1.0 per NPC update
- **Budget**: 2.5ms per frame total budget
- **Deferred Processing**: NPCs deferred when budget exhausted

### Memory Usage

- **Per NPC**: ~200 bytes (components + state)
- **Distance Cache**: ~50% hit rate, 5-frame TTL
- **Batch Queue**: Dynamic allocation based on active NPCs

## Testing

### Unit Tests

```bash
# Run NPC-specific tests
cargo test --package amp_gameplay npc::tests

# Run performance tests
cargo test --package amp_gameplay npc::tests::test_performance_constraints

# Run deterministic behavior tests
cargo test --package amp_gameplay npc::tests::test_deterministic_state_transitions
```

### Integration Tests

```bash
# Run full NPC system integration
cargo test --package amp_gameplay npc::tests::test_npc_system_integration

# Run batch processing integration
cargo test --package amp_gameplay npc::tests::test_batch_processing_integration
```

### Performance Benchmarks

```bash
# Run NPC performance benchmarks
cargo bench --package amp_gameplay npc_performance

# Check processing time targets
cargo test --package amp_gameplay npc::tests::test_tick_rate_scaling
```

## Troubleshooting

### Common Issues

1. **High Processing Time**: Check NPC count and distance distribution
2. **Unresponsive NPCs**: Verify player entity has Camera component
3. **State Transitions**: Check energy/stress levels and thresholds
4. **Batch Processing**: Ensure BatchController is properly initialized

### Performance Tuning

```rust
// Adjust update intervals for better performance
let mut config = NpcConfig::default();
config.npc_behavior.update_intervals.far_interval = 2.0; // Slower far updates
config.npc_behavior.update_intervals.close_distance = 30.0; // Smaller close range
```

### Debugging

```rust
// Enable debug logging
fn debug_npc_system(
    query: Query<(Entity, &NPC, &NpcState, &NpcBrainHandle)>,
) {
    for (entity, npc, state, brain_handle) in query.iter() {
        debug!("NPC {:?}: state={:?}, distance={:.1}m, interval={}",
            entity,
            state.current,
            brain_handle.distance_to_player,
            brain_handle.update_interval
        );
    }
}
```

## Future Enhancements

### Behavior Trees
- Replace FSM with behavior trees for complex AI
- Dynamic behavior loading from assets
- Visual behavior tree editor

### Advanced AI Features
- Pathfinding integration
- Social behaviors and group dynamics
- Dynamic difficulty adjustment
- Machine learning integration

### Performance Improvements
- GPU-accelerated behavior processing
- Hierarchical spatial partitioning
- Predictive state caching
- Multi-threaded behavior evaluation

## API Reference

### Core Types

- `NPC`: Main NPC component
- `NpcState`: State machine component
- `NpcBrainHandle`: Batch processing handle
- `NpcConfig`: Configuration resource
- `NpcMetrics`: Performance metrics resource

### System Functions

- `npc_brain_system`: Main NPC processing system
- `npc_metrics_system`: Performance monitoring system
- `evaluate_state_transition`: State transition logic
- `execute_state_behavior`: State-specific behavior execution

### Factory Functions

- `spawn_npc`: Spawn single NPC
- `spawn_npcs_batch`: Spawn multiple NPCs
- `create_random_npc_config`: Generate random NPC variants

### Utility Functions

- `DistanceCategory::from_distance`: Distance classification
- `create_npc_batch_job`: Create batch processing job
- `update_npc_emotions`: Update energy and stress levels
