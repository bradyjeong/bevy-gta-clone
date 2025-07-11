# F430BC6 Configuration Analysis

## Overview
This document provides a comprehensive analysis of all 14 RON configuration files from the f430bc6 reference workspace for implementation in the Bevy Asset system integration. Each configuration file has been analyzed for structure, dependencies, and equivalent Rust struct definitions.

## Configuration Files Analysis

### 1. vehicle_stats.ron
**Path:** `assets/config/vehicle_stats.ron`
**Purpose:** Defines performance statistics for different vehicle types (cars, helicopters, aircraft, boats)

**Structure Analysis:**
- Root structure: `vehicle_configs` hashmap
- 6 vehicle types: SuperCar, SportsCar, Car, Helicopter, F16, Boat
- Each vehicle has 8 identical fields

**Fields:**
- `engine_power`: f32 (range: 150.0 - 29000.0)
- `max_speed`: f32 (range: 80.0 - 2100.0)
- `acceleration`: f32 (range: 4.0 - 15.0)
- `braking_force`: f32 (range: 4.0 - 12.0)
- `turning_radius`: f32 (range: 0.0 - 12.0, 0.0 for air vehicles)
- `mass`: f32 (range: 800.0 - 8500.0)
- `fuel_capacity`: f32 (range: 50.0 - 3200.0)
- `fuel_consumption`: f32 (range: 0.08 - 1.8)

**Dependencies:** None - standalone configuration

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct VehicleStatsConfig {
    pub vehicle_configs: HashMap<String, VehicleStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleStats {
    pub engine_power: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub braking_force: f32,
    pub turning_radius: f32,
    pub mass: f32,
    pub fuel_capacity: f32,
    pub fuel_consumption: f32,
}
```

### 2. performance_settings.ron
**Path:** `assets/config/performance_settings.ron`
**Purpose:** Defines performance optimization settings including culling distances, LOD distances, spawn rates, and cache settings

**Structure Analysis:**
- 5 main sections with typed structs
- Nested configuration with specific performance targets

**Fields:**
- `culling_distances`: CullingDistances struct (buildings: 300.0, vehicles: 150.0, npcs: 100.0, vegetation: 200.0, effects: 80.0)
- `lod_distances`: LodDistances struct (high_detail: 100.0, medium_detail: 200.0, sleep_mode: 300.0)
- `spawn_rates`: SpawnRates struct (buildings: 0.08, vehicles: 0.04, trees: 0.05, npcs: 0.01)
- `performance_targets`: PerformanceTargets struct (target_fps: 60.0, frame_time_budget_ms: 16.67, max_entities: 2000, max_active_systems: 50)
- `cache_settings`: CacheSettings struct (distance_cache_size: 2048, cache_duration_frames: 5, cleanup_interval_frames: 300)

**Dependencies:** Used by multiple systems - rendering, culling, spawning, performance monitoring

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PerformanceSettingsConfig {
    pub culling_distances: CullingDistances,
    pub lod_distances: LodDistances,
    pub spawn_rates: SpawnRates,
    pub performance_targets: PerformanceTargets,
    pub cache_settings: CacheSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingDistances {
    pub buildings: f32,
    pub vehicles: f32,
    pub npcs: f32,
    pub vegetation: f32,
    pub effects: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodDistances {
    pub high_detail: f32,
    pub medium_detail: f32,
    pub sleep_mode: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRates {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub target_fps: f32,
    pub frame_time_budget_ms: f32,
    pub max_entities: u32,
    pub max_active_systems: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub distance_cache_size: u32,
    pub cache_duration_frames: u32,
    pub cleanup_interval_frames: u32,
}
```

### 3. audio_settings.ron
**Path:** `assets/config/audio_settings.ron`
**Purpose:** Comprehensive audio configuration including volumes, spatial audio, engine audio, and effects

**Structure Analysis:**
- Nested audio configuration with 7 main sections
- Mix of f32 values and Vec<f32> for curves
- Extensive volume and timing controls

**Fields:**
- `volumes`: Volume levels for all audio types (master, engine, turbo, exhaust, backfire, footstep, horn, ambient)
- `footsteps`: Footstep timing and variation settings
- `spatial`: 3D spatial audio settings (fade_distance: 100.0, max_audio_distance: 250.0)
- `timing`: Audio timing controls (cleanup_interval: 1.0, update_interval: 0.05)
- `engine`: Engine audio curves and settings (frequency/volume curves as arrays)
- `effects`: Audio effects processing (echo, filters)

**Dependencies:** Used by audio system, vehicle system, player movement system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AudioSettingsConfig {
    pub audio: AudioSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub volumes: AudioVolumes,
    pub footsteps: FootstepSettings,
    pub spatial: SpatialAudio,
    pub timing: AudioTiming,
    pub engine: EngineAudio,
    pub effects: AudioEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVolumes {
    pub master: f32,
    pub engine: f32,
    pub turbo: f32,
    pub exhaust: f32,
    pub backfire: f32,
    pub footstep: f32,
    pub horn: f32,
    pub ambient: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootstepSettings {
    pub base_interval: f32,
    pub walk_multiplier: f32,
    pub run_multiplier: f32,
    pub interval_variation: f32,
    pub min_speed_threshold: f32,
    pub volume_variation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudio {
    pub fade_distance: f32,
    pub max_audio_distance: f32,
    pub doppler_factor: f32,
    pub reverb_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTiming {
    pub cleanup_interval: f32,
    pub update_interval: f32,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineAudio {
    pub idle_frequency: f32,
    pub max_frequency: f32,
    pub frequency_curve: Vec<f32>,
    pub volume_curve: Vec<f32>,
    pub turbo_whistle_frequency: f32,
    pub backfire_chance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffects {
    pub echo_delay: f32,
    pub echo_decay: f32,
    pub low_pass_cutoff: f32,
    pub high_pass_cutoff: f32,
}
```

### 4. physics_constants.ron
**Path:** `assets/config/physics_constants.ron`
**Purpose:** Defines physics constants for ground detection, world physics, and validation bounds

**Structure Analysis:**
- 3 main sections: ground_detection, world_physics, validation
- Safety bounds and validation parameters
- Mix of positive/negative values for physics simulation

**Fields:**
- `ground_detection`: Ground detection parameters (detection_height: 100.0, various margins and offsets)
- `world_physics`: Core physics constants (gravity: -9.81, air_resistance: 0.001, friction, damping, limits)
- `validation`: Safety validation bounds (position/velocity limits)

**Dependencies:** Core physics system, collision detection, vehicle physics

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PhysicsConstantsConfig {
    pub ground_detection: GroundDetection,
    pub world_physics: WorldPhysics,
    pub validation: ValidationBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundDetection {
    pub detection_height: f32,
    pub default_ground_height: f32,
    pub min_ground_height: f32,
    pub max_ground_height: f32,
    pub extra_margin: f32,
    pub spawn_area_radius: f32,
    pub terrain_surface_offset: f32,
    pub noise_scale: f32,
    pub noise_amplitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPhysics {
    pub gravity: f32,
    pub air_resistance: f32,
    pub basic_friction: f32,
    pub max_world_coord: f32,
    pub min_world_coord: f32,
    pub max_velocity: f32,
    pub max_angular_velocity: f32,
    pub max_collider_size: f32,
    pub min_collider_size: f32,
    pub max_mass: f32,
    pub min_mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationBounds {
    pub max_position_check: f32,
    pub max_velocity_check: f32,
    pub safe_position_bounds: f32,
    pub safe_velocity_bounds: f32,
}
```

### 5. world_generation.ron
**Path:** `assets/config/world_generation.ron`
**Purpose:** Defines world generation parameters including chunk system, streaming, and content spawning

**Structure Analysis:**
- 4 main sections: world_generation, spawn_rates, entity_limits, content_distances
- Hierarchical chunk system (macro -> region -> local -> detail -> micro)
- Performance-optimized entity limits

**Fields:**
- `world_generation`: Chunk sizes and streaming radiuses, generation limits, lake configuration
- `spawn_rates`: Ultra-reduced spawn rates for performance (buildings: 0.08, vehicles: 0.04, trees: 0.05, npcs: 0.01)
- `entity_limits`: Maximum entity counts per type
- `content_distances`: Spawn distances for different content types

**Dependencies:** World streaming system, content spawning, performance management

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct WorldGenerationConfig {
    pub world_generation: WorldGeneration,
    pub spawn_rates: SpawnRates,
    pub entity_limits: EntityLimits,
    pub content_distances: ContentDistances,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGeneration {
    pub chunk_size: f32,
    pub macro_region_size: f32,
    pub region_size: f32,
    pub local_chunk_size: f32,
    pub detail_chunk_size: f32,
    pub micro_chunk_size: f32,
    pub macro_streaming_radius: f32,
    pub region_streaming_radius: f32,
    pub local_streaming_radius: f32,
    pub detail_streaming_radius: f32,
    pub micro_streaming_radius: f32,
    pub max_chunks_loaded_per_frame: u32,
    pub max_chunks_unloaded_per_frame: u32,
    pub max_content_generated_per_frame: u32,
    pub grid_resolution: f32,
    pub ground_height_offset: f32,
    pub lake_size: f32,
    pub lake_depth: f32,
    pub lake_position: (f32, f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRates {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLimits {
    pub max_buildings: u32,
    pub max_vehicles: u32,
    pub max_npcs: u32,
    pub max_trees: u32,
    pub max_particles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDistances {
    pub building_spawn_distance: f32,
    pub tree_spawn_distance: f32,
    pub vehicle_spawn_distance: f32,
    pub npc_spawn_distance: f32,
}
```

### 6. vehicle_physics.ron
**Path:** `assets/config/vehicle_physics.ron`
**Purpose:** Comprehensive vehicle physics configuration including general physics constants and specific vehicle type configurations

**Structure Analysis:**
- 2 main sections: vehicle_physics (general) and vehicle_types (specific configurations)
- 5 vehicle types: starter_car, luxury_car, supercar, helicopter, f16_fighter
- Complex nested structures with physics, visual, audio, and specialized properties

**Fields:**
- `vehicle_physics`: General physics limits and constants (time steps, suspension, tires, aerodynamics)
- `vehicle_types`: Detailed configuration for each vehicle type including body size, performance, physics properties, visual settings, audio, fuel, and special effects

**Dependencies:** Vehicle system, physics system, audio system, visual effects system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct VehiclePhysicsConfig {
    pub vehicle_physics: VehiclePhysics,
    pub vehicle_types: HashMap<String, VehicleTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePhysics {
    pub min_delta_time: f32,
    pub max_delta_time: f32,
    pub max_processing_time: f32,
    pub max_physics_entities: u32,
    pub physics_distance_threshold: f32,
    pub simplification_distance: f32,
    pub suspension_stiffness: f32,
    pub suspension_damping: f32,
    pub tire_grip_coefficient: f32,
    pub tire_slip_threshold: f32,
    pub tire_friction_curve: Vec<f32>,
    pub air_density: f32,
    pub drag_coefficient: f32,
    pub downforce_coefficient: f32,
    pub max_engine_force: f32,
    pub max_brake_force: f32,
    pub max_steering_angle: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleTypeConfig {
    pub body_size: (f32, f32, f32),
    pub collider_size: (f32, f32, f32),
    pub mass: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub braking_force: f32,
    pub turning_radius: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub center_of_mass_offset: (f32, f32, f32),
    pub default_color: (f32, f32, f32, f32),
    pub engine_volume: f32,
    pub horn_volume: f32,
    pub fuel_capacity: f32,
    pub fuel_consumption: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exhaust_timer_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exhaust_position_offset: Option<(f32, f32, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turbo_stages: Option<Vec<TurboStage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afterburner_boost: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flame_intensity_threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flicker_speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flame_scale_multiplier: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flame_colors: Option<Vec<(f32, f32, f32, f32)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurboStage {
    pub stage: u32,
    pub flame_color: (f32, f32, f32, f32),
    pub emission_intensity: f32,
}
```

### 7. lod_config.ron
**Path:** `assets/config/lod_config.ron`
**Purpose:** Level of Detail configuration for different entity types

**Structure Analysis:**
- 4 entity types: vehicle_lod, vegetation_lod, building_lod, npc_lod
- Each type has 4 distance thresholds: high -> medium -> sleep -> cull

**Fields:**
- `vehicle_lod`: (high: 100.0, medium: 300.0, sleep: 500.0, cull: 800.0)
- `vegetation_lod`: (full: 50.0, medium: 150.0, billboard: 300.0, cull: 500.0)
- `building_lod`: (high: 200.0, medium: 500.0, sleep: 1000.0, cull: 1500.0)
- `npc_lod`: (high: 50.0, medium: 100.0, sleep: 200.0, cull: 300.0)

**Dependencies:** LOD system, rendering system, performance optimization

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct LodConfig {
    pub vehicle_lod: VehicleLod,
    pub vegetation_lod: VegetationLod,
    pub building_lod: BuildingLod,
    pub npc_lod: NpcLod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLod {
    pub high_distance: f32,
    pub medium_distance: f32,
    pub sleep_distance: f32,
    pub cull_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationLod {
    pub full_distance: f32,
    pub medium_distance: f32,
    pub billboard_distance: f32,
    pub cull_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingLod {
    pub high_distance: f32,
    pub medium_distance: f32,
    pub sleep_distance: f32,
    pub cull_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcLod {
    pub high_distance: f32,
    pub medium_distance: f32,
    pub sleep_distance: f32,
    pub cull_distance: f32,
}
```

### 8. performance_config.ron
**Path:** `assets/config/performance_config.ron`
**Purpose:** Performance configuration with entity limits, spawn rates, culling distances, and cache settings

**Structure Analysis:**
- 6 main sections: target_fps, max_entities, spawn_rates, culling_distances, update_intervals, cache_settings
- Concise performance-focused configuration

**Fields:**
- `target_fps`: 60.0
- `max_entities`: Per-entity-type limits (buildings: 500, vehicles: 50, npcs: 30, vegetation: 1000)
- `spawn_rates`: Same as other configs (buildings: 0.08, vehicles: 0.04, trees: 0.05, npcs: 0.01)
- `culling_distances`: Distance-based culling thresholds
- `update_intervals`: System update timing
- `cache_settings`: Cache configuration

**Dependencies:** Performance system, entity management, rendering system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PerformanceConfig {
    pub target_fps: f32,
    pub max_entities: MaxEntities,
    pub spawn_rates: SpawnRates,
    pub culling_distances: CullingDistances,
    pub update_intervals: UpdateIntervals,
    pub cache_settings: CacheSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxEntities {
    pub buildings: u32,
    pub vehicles: u32,
    pub npcs: u32,
    pub vegetation: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRates {
    pub buildings: f32,
    pub vehicles: f32,
    pub trees: f32,
    pub npcs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingDistances {
    pub buildings: f32,
    pub vehicles: f32,
    pub npcs: f32,
    pub vegetation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntervals {
    pub road_generation: f32,
    pub dynamic_content: f32,
    pub culling: f32,
    pub lod_update: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub max_entries: u32,
    pub cache_duration: f32,
    pub cleanup_interval: f32,
}
```

### 9. performance_tuning.ron
**Path:** `assets/config/performance_tuning.ron`
**Purpose:** Comprehensive performance tuning configuration with detailed batch processing, LOD arrays, and optimization settings

**Structure Analysis:**
- 4 main sections: performance, entity_limits, spawn_validation
- Most comprehensive performance configuration with arrays for LOD distances
- Detailed batch processing configuration

**Fields:**
- `performance`: Extensive performance tuning (batch sizes, update intervals, distance cache, culling, LOD arrays, streaming)
- `entity_limits`: Entity count limits
- `spawn_validation`: Spawn validation parameters

**Dependencies:** Performance system, batch processing, LOD system, streaming system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PerformanceTuningConfig {
    pub performance: PerformanceTuning,
    pub entity_limits: EntityLimits,
    pub spawn_validation: SpawnValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTuning {
    pub target_fps: f32,
    pub frame_time_threshold: f32,
    pub max_entities_per_frame: u32,
    pub max_processing_time_ms: f32,
    pub max_render_operations: u32,
    pub batch_sizes: BatchSizes,
    pub update_intervals: UpdateIntervals,
    pub distance_cache: DistanceCache,
    pub culling: CullingSystem,
    pub lod_distances: LodDistances,
    pub streaming: StreamingSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSizes {
    pub transform_batch: u32,
    pub visibility_batch: u32,
    pub physics_batch: u32,
    pub lod_batch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntervals {
    pub render_optimization: f32,
    pub vegetation_instancing: f32,
    pub batch_processing: f32,
    pub distance_cache_debug: f32,
    pub audio_cleanup: f32,
    pub effect_update: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceCache {
    pub max_entries: u32,
    pub frame_cache_duration: u32,
    pub cleanup_per_frame: u32,
    pub efficiency_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingSystem {
    pub max_distance: f32,
    pub view_frustum_fov: f32,
    pub view_frustum_range: f32,
    pub hysteresis_buffer: f32,
    pub check_interval: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodDistances {
    pub buildings: Vec<f32>,
    pub vehicles: Vec<f32>,
    pub npcs: Vec<f32>,
    pub vegetation: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingSystem {
    pub max_loaded_per_frame: u32,
    pub max_unloaded_per_frame: u32,
    pub max_generated_per_frame: u32,
    pub processing_time_budget: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLimits {
    pub buildings: u32,
    pub vehicles: u32,
    pub npcs: u32,
    pub trees: u32,
    pub particles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnValidation {
    pub batch_size: u32,
    pub max_attempts: u32,
    pub min_distance_from_player: f32,
    pub max_distance_from_player: f32,
    pub safe_spawn_area: f32,
}
```

### 10. visual_effects.ron
**Path:** `assets/config/visual_effects.ron`
**Purpose:** Visual effects configuration including particles, exhaust effects, jet flames, vegetation rendering, lighting, and materials

**Structure Analysis:**
- 6 main sections: particles, exhaust, jet_flames, vegetation, lighting, materials
- Color arrays for effects and materials
- Comprehensive visual system configuration

**Fields:**
- `particles`: Particle system configuration
- `exhaust`: Exhaust flame effects with color arrays
- `jet_flames`: Jet engine flame effects
- `vegetation`: Vegetation rendering with instancing
- `lighting`: Lighting configuration
- `materials`: Material property definitions for different surfaces

**Dependencies:** Visual effects system, particle system, materials system, lighting system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct VisualEffectsConfig {
    pub visual_effects: VisualEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEffects {
    pub particles: ParticleSystem,
    pub exhaust: ExhaustEffects,
    pub jet_flames: JetFlames,
    pub vegetation: VegetationRendering,
    pub lighting: LightingConfig,
    pub materials: MaterialsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub max_particles_per_system: u32,
    pub particle_lifetime: f32,
    pub spawn_rate: f32,
    pub gravity_factor: f32,
    pub wind_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExhaustEffects {
    pub max_flames: u32,
    pub flame_lifetime: f32,
    pub flame_size: f32,
    pub flicker_speed: f32,
    pub emission_intensity: f32,
    pub colors: Vec<(f32, f32, f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetFlames {
    pub base_intensity: f32,
    pub afterburner_boost: f32,
    pub flame_intensity_threshold: f32,
    pub flicker_speed: f32,
    pub scale_multiplier: f32,
    pub emission_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationRendering {
    pub instancing_enabled: bool,
    pub max_instances: u32,
    pub lod_bias: f32,
    pub wind_strength: f32,
    pub colors: Vec<(f32, f32, f32, f32)>,
    pub scale_variation: (f32, f32),
    pub age_variation: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    pub ambient_intensity: f32,
    pub directional_intensity: f32,
    pub point_light_radius: f32,
    pub shadow_distance: f32,
    pub shadow_quality: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialsConfig {
    pub water: MaterialProperties,
    pub road: MaterialProperties,
    pub concrete: MaterialProperties,
    pub glass: MaterialProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    pub base_color: (f32, f32, f32, f32),
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
}
```

### 11. game_config.ron
**Path:** `assets/config/game_config.ron`
**Purpose:** Comprehensive game configuration consolidating many settings from other configs into a single master configuration

**Structure Analysis:**
- 15 main sections covering all aspects of the game
- Most comprehensive configuration file
- Consolidates settings from multiple other configs

**Fields:**
- Extensive configuration covering spawn rates, entity limits, LOD distances, physics, vehicles, NPCs, world settings, audio, visual effects, and more

**Dependencies:** Master configuration used by all systems

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct GameConfig {
    pub spawn_rates: SpawnRates,
    pub entity_limits: EntityLimits,
    pub lod_distances: LodDistances,
    pub culling_distances: CullingDistances,
    pub update_intervals: UpdateIntervals,
    pub physics: PhysicsConfig,
    pub vehicle_physics: VehiclePhysicsConfig,
    pub world: WorldConfig,
    pub audio: AudioConfig,
    pub visual: VisualConfig,
    pub npc_behavior: NpcBehaviorConfig,
    pub performance: PerformanceConfig,
    pub vehicles: HashMap<String, VehicleConfig>,
    pub npc: NpcConfig,
}

// Additional structs for each section would be defined here...
```

### 12. camera_settings.ron
**Path:** `assets/config/camera_settings.ron`
**Purpose:** Camera system configuration including chase camera, modes, smoothing, collision, and cinematic settings

**Structure Analysis:**
- 6 main sections: chase_camera, modes, smoothing, collision, cinematic
- Camera modes as hashmap with different configurations
- Comprehensive camera behavior settings

**Fields:**
- `chase_camera`: Chase camera settings (distance, height, lerp speed, look ahead)
- `modes`: Camera mode configurations (player, vehicle, aircraft)
- `smoothing`: Movement smoothing settings
- `collision`: Camera collision detection
- `cinematic`: Cinematic camera settings

**Dependencies:** Camera system, input system, collision system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct CameraSettingsConfig {
    pub camera: CameraSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    pub chase_camera: ChaseCamera,
    pub modes: HashMap<String, CameraMode>,
    pub smoothing: CameraSmoothing,
    pub collision: CameraCollision,
    pub cinematic: CinematicSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaseCamera {
    pub distance: f32,
    pub height: f32,
    pub lerp_speed: f32,
    pub look_ahead_distance: f32,
    pub look_ahead_height: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub collision_padding: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraMode {
    pub distance: f32,
    pub height: f32,
    pub offset: (f32, f32, f32),
    pub fov: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSmoothing {
    pub position_smoothing: f32,
    pub rotation_smoothing: f32,
    pub fov_smoothing: f32,
    pub shake_intensity: f32,
    pub shake_duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraCollision {
    pub enable_collision: bool,
    pub min_distance: f32,
    pub collision_layers: Vec<String>,
    pub smooth_collision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CinematicSettings {
    pub transition_speed: f32,
    pub ease_in_out: bool,
    pub auto_focus: bool,
    pub depth_of_field: bool,
}
```

### 13. npc_behavior.ron
**Path:** `assets/config/npc_behavior.ron`
**Purpose:** NPC behavior configuration including physical properties, movement, emotions, AI, spawn settings, and appearance

**Structure Analysis:**
- 7 main sections: physical, movement, emotions, ai, spawn, appearance, update_intervals
- Complex emotional system with energy and stress levels
- Appearance variety with color arrays

**Fields:**
- `physical`: Physical properties (height, build, capsule dimensions, mass)
- `movement`: Movement speeds and parameters
- `emotions`: Energy and stress level systems
- `ai`: AI behavior timing and parameters
- `spawn`: NPC spawning configuration
- `appearance`: Appearance variety (skin tones, hair colors, clothing colors)
- `update_intervals`: Update timing based on distance

**Dependencies:** NPC system, AI system, spawning system, appearance system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct NpcBehaviorConfig {
    pub npc_behavior: NpcBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcBehavior {
    pub physical: PhysicalProperties,
    pub movement: MovementProperties,
    pub emotions: EmotionalSystem,
    pub ai: AiBehavior,
    pub spawn: SpawnSettings,
    pub appearance: AppearanceVariety,
    pub update_intervals: UpdateIntervals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalProperties {
    pub default_height: f32,
    pub default_build: f32,
    pub capsule_radius: f32,
    pub capsule_height: f32,
    pub mass: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementProperties {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub turning_speed: f32,
    pub avoidance_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalSystem {
    pub energy_levels: EnergyLevels,
    pub stress_levels: StressLevels,
    pub mood_change_cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyLevels {
    pub max_energy: f32,
    pub resting_threshold: f32,
    pub tired_threshold: f32,
    pub energetic_threshold: f32,
    pub energy_drain_rate: f32,
    pub energy_recovery_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressLevels {
    pub max_stress: f32,
    pub calm_threshold: f32,
    pub stressed_threshold: f32,
    pub panic_threshold: f32,
    pub stress_buildup_rate: f32,
    pub stress_recovery_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBehavior {
    pub decision_interval: f32,
    pub path_recalculation_interval: f32,
    pub reaction_time: f32,
    pub attention_span: f32,
    pub memory_duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSettings {
    pub max_npcs: u32,
    pub spawn_interval: f32,
    pub spawn_radius: f32,
    pub despawn_distance: f32,
    pub min_spawn_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceVariety {
    pub skin_tones: Vec<(f32, f32, f32, f32)>,
    pub hair_colors: Vec<(f32, f32, f32, f32)>,
    pub clothing_colors: Vec<(f32, f32, f32, f32)>,
    pub height_variation: (f32, f32),
    pub build_variation: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntervals {
    pub close_distance: f32,
    pub far_distance: f32,
    pub close_interval: f32,
    pub medium_interval: f32,
    pub far_interval: f32,
}
```

### 14. ui_settings.ron
**Path:** `assets/config/ui_settings.ron`
**Purpose:** UI system configuration including typography, layout, colors, animation, HUD elements, and debug UI

**Structure Analysis:**
- 7 main sections: typography, layout, colors, animation, hud, debug
- Comprehensive UI styling and layout configuration
- Color arrays for theming

**Fields:**
- `typography`: Font sizes for different UI elements
- `layout`: Spacing and layout parameters
- `colors`: Color scheme for UI elements
- `animation`: Animation timing and easing
- `hud`: HUD element configurations (FPS counter, vehicle info, minimap)
- `debug`: Debug UI settings

**Dependencies:** UI system, HUD system, debug system

**Rust Struct:**
```rust
use serde::{Deserialize, Serialize};
use bevy::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct UiSettingsConfig {
    pub ui: UiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub typography: Typography,
    pub layout: Layout,
    pub colors: Colors,
    pub animation: Animation,
    pub hud: HudSettings,
    pub debug: DebugSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub default_font_size: f32,
    pub fps_font_size: f32,
    pub title_font_size: f32,
    pub subtitle_font_size: f32,
    pub body_font_size: f32,
    pub caption_font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub default_padding: f32,
    pub panel_padding: f32,
    pub button_padding: f32,
    pub margin: f32,
    pub border_radius: f32,
    pub panel_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    pub background: (f32, f32, f32, f32),
    pub text: (f32, f32, f32, f32),
    pub text_secondary: (f32, f32, f32, f32),
    pub accent: (f32, f32, f32, f32),
    pub success: (f32, f32, f32, f32),
    pub warning: (f32, f32, f32, f32),
    pub error: (f32, f32, f32, f32),
    pub panel_background: (f32, f32, f32, f32),
    pub button_background: (f32, f32, f32, f32),
    pub button_hover: (f32, f32, f32, f32),
    pub button_active: (f32, f32, f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub fade_duration: f32,
    pub slide_duration: f32,
    pub scale_duration: f32,
    pub ease_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudSettings {
    pub fps_counter: FpsCounter,
    pub vehicle_info: VehicleInfo,
    pub minimap: Minimap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpsCounter {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub update_interval: f32,
    pub show_frame_time: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub show_speed: bool,
    pub show_fuel: bool,
    pub show_damage: bool,
    pub show_gear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minimap {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub zoom_level: f32,
    pub show_entities: bool,
    pub show_roads: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettings {
    pub show_debug_info: bool,
    pub show_physics_debug: bool,
    pub show_performance_graph: bool,
    pub debug_text_size: f32,
    pub debug_colors: Vec<(f32, f32, f32, f32)>,
}
```

## Configuration Dependencies

### Inter-Configuration Dependencies

1. **Performance Chain:**
   - `performance_settings.ron` → `performance_config.ron` → `performance_tuning.ron`
   - Shared spawn rates, entity limits, culling distances

2. **Vehicle Chain:**
   - `vehicle_stats.ron` → `vehicle_physics.ron` → `game_config.ron`
   - Vehicle properties used across multiple systems

3. **LOD Chain:**
   - `lod_config.ron` → `performance_tuning.ron` (LOD arrays)
   - Distance-based optimization settings

4. **Master Configuration:**
   - `game_config.ron` consolidates settings from multiple other configs
   - Acts as a central configuration hub

### System Dependencies

1. **Physics System:**
   - `physics_constants.ron`, `vehicle_physics.ron`, `game_config.ron`

2. **Rendering System:**
   - `lod_config.ron`, `visual_effects.ron`, `performance_settings.ron`

3. **Audio System:**
   - `audio_settings.ron`, `vehicle_physics.ron` (audio volumes)

4. **UI System:**
   - `ui_settings.ron`, `camera_settings.ron`

5. **World System:**
   - `world_generation.ron`, `game_config.ron`

6. **NPC System:**
   - `npc_behavior.ron`, `performance_settings.ron` (spawn rates)

## Implementation Strategy for Bevy Asset System

### 1. Asset Loading Structure
```rust
// In config_core/src/lib.rs
pub mod assets {
    pub use crate::configs::*;
    
    pub const VEHICLE_STATS: &str = "config/vehicle_stats.ron";
    pub const PERFORMANCE_SETTINGS: &str = "config/performance_settings.ron";
    pub const AUDIO_SETTINGS: &str = "config/audio_settings.ron";
    // ... other config paths
}
```

### 2. Asset Registration
```rust
// In config_core/src/plugin.rs
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<VehicleStatsConfig>()
            .add_asset::<PerformanceSettingsConfig>()
            .add_asset::<AudioSettingsConfig>()
            // ... register all config assets
            .init_asset_loader::<RonAssetLoader<VehicleStatsConfig>>()
            .init_asset_loader::<RonAssetLoader<PerformanceSettingsConfig>>()
            // ... register all loaders
            .add_startup_system(load_configs);
    }
}
```

### 3. Config Loading System
```rust
fn load_configs(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let vehicle_stats: Handle<VehicleStatsConfig> = asset_server.load(assets::VEHICLE_STATS);
    let performance_settings: Handle<PerformanceSettingsConfig> = asset_server.load(assets::PERFORMANCE_SETTINGS);
    // ... load all configs
    
    commands.insert_resource(ConfigHandles {
        vehicle_stats,
        performance_settings,
        // ... all handles
    });
}
```

### 4. Hot Reloading Support
All configurations will support hot reloading through Bevy's asset system, allowing for runtime tuning and iteration.

## Summary

The f430bc6 configuration system consists of 14 comprehensive RON files covering all aspects of the game:

- **Vehicle configurations** (2 files): Stats and physics
- **Performance configurations** (4 files): Various optimization settings
- **Audio configuration** (1 file): Comprehensive audio system
- **Physics configuration** (1 file): Core physics constants
- **World configuration** (1 file): World generation parameters
- **Visual configuration** (1 file): Effects and materials
- **Game configuration** (1 file): Master configuration consolidation
- **UI configuration** (1 file): UI system settings
- **Camera configuration** (1 file): Camera behavior
- **NPC configuration** (1 file): NPC behavior and appearance
- **LOD configuration** (1 file): Level of detail settings

The configuration system is designed for:
- **Modularity**: Separate configs for different systems
- **Performance**: Extensive optimization settings
- **Flexibility**: Hot reloading and runtime tuning
- **Completeness**: Comprehensive coverage of all game systems
- **Data-driven**: Moving hardcoded values to configuration files

This analysis provides the foundation for implementing the Bevy Asset system integration as part of the AAA-restoration roadmap.
