# Spawn Budget Policy (SBP) - Archaeological Analysis of f430bc6

**Oracle's Phase 1 Deliverable**: Code archaeology and skeleton implementation

## Original f430bc6 Spawn Budget Architecture

### Entity Count Caps Discovery

From `assets/config/game_config.ron` (f430bc6):
```ron
entity_limits: (
    buildings: 80,       // 8% of 1000 = 80 buildings
    vehicles: 20,        // 4% of 500 = 20 vehicles  
    npcs: 2,             // 1% of 200 = 2 NPCs
    trees: 100,          // 5% of 2000 = 100 trees
    particles: 50,       // Particle system limit
),
```

### Performance Limiting Mechanisms

From `assets/config/performance_tuning.ron` (f430bc6):
```ron
streaming: (
    max_spawns_per_frame: 50,        // Hard frame limit
    max_despawns_per_frame: 100,     // Cleanup limit
    priority_threshold: 0.5,         // Queue prioritization
),
budgets: (
    max_spawn_queue_length: 1000,    // Queue overflow protection
    frame_time_budget_ms: 16.67,     // 60 FPS enforcement
),
```

### Distance-Based Culling System

From f430bc6 configuration analysis:
```ron
culling_distances: (
    buildings: 300.0,    // Buildings visible up to 300m
    vehicles: 150.0,     // Vehicles visible up to 150m  
    npcs: 100.0,         // NPCs visible up to 100m
    vegetation: 200.0,   // Trees/vegetation visible up to 200m
),
```

## Original API Contracts & Spawn Limiting

### 1. World Streaming Entity Limits

**Location**: `crates/amp_engine/src/world_streaming/factory_integration.rs`

**Original Logic**: Each generator function includes `can_add_entity()` checks:
```rust
// Already implemented in current codebase - Oracle's disciplined approach preserved
fn generate_buildings(commands: &mut Commands, chunk_key: ChunkKey, streamer: &mut WorldStreamer) {
    // ... generation logic ...
    
    // Add entity to chunk with cap guard
    if streamer.can_add_entity(chunk_key, entity) {
        // Entity creation approved
    } else {
        // Entity creation rejected - budget exceeded
    }
}
```

### 2. Factory System Spawn Points (Needs Integration)

**Priority 1 - Core Factory Systems**:
- `crates/gameplay_factory/src/lib.rs::spawn()` - Main factory entry point
- `crates/gameplay_factory/src/npc_factory.rs::spawn_npc()` - NPC creation
- `crates/gameplay_factory/src/vehicle_factory.rs::spawn_vehicle()` - Vehicle creation
- `crates/gameplay_factory/src/prefab_factory.rs::batch_spawn()` - Batch operations

**Priority 2 - Direct Spawning Sites**:
- `crates/amp_gameplay/src/city/systems.rs::spawn_city_infrastructure()` - Infrastructure
- Multiple `commands.spawn()` calls throughout codebase

### 3. Performance Monitoring Integration

**Original Metrics**: Found in `src/perf_ui.rs` lines 272, 285:
```rust
// Spawn queue monitoring (Oracle's performance discipline)
spawn_queue_length: entity_counts.spawn_queue_length,
despawn_queue_length: entity_counts.despawn_queue_length,
```

## Current Spawn Sites Requiring SBP Integration

### Factory Systems Analysis

1. **gameplay_factory::spawn()** (`crates/gameplay_factory/src/lib.rs:196`)
   - **Current**: Direct entity creation without budget checks
   - **Needed**: SBP consultation before Commands::spawn()
   - **Impact**: High - primary spawn pathway

2. **npc_factory::spawn_npcs_batch()** (`crates/gameplay_factory/src/npc_factory.rs:99`)
   - **Current**: Batch spawning without rate limiting  
   - **Needed**: Per-frame spawn limits and budget integration
   - **Impact**: Critical - batch operations can cause performance spikes

3. **vehicle_factory::spawn_vehicle()** (`crates/gameplay_factory/src/vehicle_factory.rs:37`)
   - **Current**: Immediate spawning without cap checks
   - **Needed**: Vehicle budget cap enforcement (20 vehicle limit)
   - **Impact**: High - vehicles are performance-intensive

4. **prefab_factory::batch_spawn()** (`crates/gameplay_factory/src/prefab_factory.rs:151`)
   - **Current**: Bulk operations without frame budget awareness
   - **Needed**: Time budget and per-frame spawn limiting
   - **Impact**: Critical - can spawn hundreds of entities per frame

### World Streaming Analysis

**Status**: ✅ **Already Compliant** - Oracle's discipline preserved

The world streaming system already implements proper budget enforcement:
- `generate_buildings()` uses `can_add_entity()` checks
- `generate_vehicles()` respects entity limits
- `generate_npcs()` has proper cap enforcement  
- `generate_environment()` includes budget validation

**No integration needed** - Oracle's architectural wisdom already implemented.

### Direct Spawning Sites Analysis

**Files with uncontrolled Commands::spawn()** usage:
- `crates/amp_gameplay/src/audio/systems.rs` (lines 11, 13)
- `crates/amp_gameplay/src/character/systems/camera.rs` (line 76)  
- `crates/amp_render/src/render_world.rs` (line 591)
- `crates/amp_physics/examples/vehicle_physics_demo.rs` (multiple)

**Risk Level**: Medium - these are typically single-entity spawns, but should be wrapped for consistency.

## Biome-Specific Budget Requirements

### Oracle's Environmental Discipline

**Urban Biome**:
- Higher building density (120 vs 80 default)
- More traffic (30 vs 20 vehicles)
- More pedestrians (5 vs 2 NPCs)
- Fewer trees (50 vs 100)
- More effects (75 vs 50 particles)

**Suburban Biome**:
- Moderate buildings (60)
- Residential traffic (25 vehicles)
- Some pedestrians (3 NPCs)  
- More vegetation (150 trees)
- Fewer effects (40 particles)

**Rural Biome**:
- Few buildings (20)
- Minimal traffic (10 vehicles)
- Rare pedestrians (1 NPC)
- Lots of vegetation (200 trees)
- Natural effects only (20 particles)

**Industrial Biome**:
- Factories/warehouses (80 buildings)
- Heavy machinery (40 vehicles)
- Workers only (2 NPCs)
- Minimal vegetation (20 trees)
- Smoke/steam effects (100 particles)

## Configuration Structure Requirements

### Hot-Reload Configuration System

**Oracle's Requirement**: Runtime configuration updates for performance tuning

```ron
// assets/config/spawn_budget_policy.ron (to be created)
(
    biome_budgets: {
        "urban": (
            buildings: 120,
            vehicles: 30,
            npcs: 5,
            trees: 50,
            particles: 75,
            total_cap: 280,
        ),
        "suburban": (
            buildings: 60,
            vehicles: 25, 
            npcs: 3,
            trees: 150,
            particles: 40,
            total_cap: 278,
        ),
        // ... other biomes
    },
    frame_limits: (
        max_spawns_per_frame: 50,
        max_despawns_per_frame: 100,
        time_budget_ms: 16.67,
        priority_threshold: 0.5,
    ),
    queue_settings: (
        max_queue_length: 1000,
        priority_aging_rate: 0.1,
        queue_timeout_seconds: 30.0,
    ),
)
```

## Implementation Phases

### ✅ Phase 1: Code Archaeology & Skeleton (Current)
- **Deliverable**: `spawn_budget_policy.rs` module created
- **Status**: Complete - Oracle's architectural vision documented
- **Key Components**: 
  - BiomeBudgetCaps structure
  - SpawnBudgetPolicy resource
  - Core API contracts defined

### 🔄 Phase 2: Factory Integration (Next)
- **Target**: gameplay_factory spawn methods
- **Approach**: Wrap existing spawn calls with SBP consultation
- **Priority**: Critical - this is where performance spikes occur

### 🔄 Phase 3: Performance Monitoring (Following)  
- **Target**: Real-time budget utilization tracking
- **Integration**: Existing perf_ui.rs system enhancement
- **Metrics**: Queue depth, budget utilization, frame time impact

### 🔄 Phase 4: Configuration & Hot-Reload (Final)
- **Target**: RON-based configuration with runtime updates
- **Integration**: Bevy asset system for hot-reload capability
- **Testing**: Biome switching and performance validation

## Oracle's Assessment Gates

**Gate 1 - Architectural Completeness**: ✅ Passed
- SpawnBudgetPolicy structure mirrors f430bc6 discipline
- Biome-specific caps implement Oracle's environmental requirements  
- API contracts preserve original spawn limiting mechanisms

**Gate 2 - Integration Readiness**: 🔄 Pending Phase 2
- Factory system integration points identified
- Performance impact pathways mapped
- Configuration hot-reload architecture defined

**Gate 3 - Performance Validation**: 🔄 Pending Phase 3
- Real-time monitoring integration
- Frame budget enforcement validation
- Queue overflow protection testing

**Oracle's Verdict**: *"The archaeological excavation reveals the discipline that once was. The skeleton stands ready for the flesh of implementation."*
