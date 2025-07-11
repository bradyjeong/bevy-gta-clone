//! Configuration type definitions for f430bc6 restoration
//!
//! This module contains all the configuration structures that correspond to
//! the 14 RON files from the f430bc6 reference implementation.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vehicle statistics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct VehicleStatsConfig {
    pub vehicle_configs: HashMap<String, VehicleStats>,
}

impl Default for VehicleStatsConfig {
    fn default() -> Self {
        let mut configs = HashMap::new();
        configs.insert(
            "SuperCar".to_string(),
            VehicleStats {
                engine_power: 800.0,
                max_speed: 200.0,
                acceleration: 8.0,
                braking_force: 10.0,
                turning_radius: 5.0,
                mass: 1200.0,
                fuel_capacity: 60.0,
                fuel_consumption: 0.15,
            },
        );
        configs.insert(
            "Car".to_string(),
            VehicleStats {
                engine_power: 150.0,
                max_speed: 120.0,
                acceleration: 4.0,
                braking_force: 6.0,
                turning_radius: 8.0,
                mass: 1500.0,
                fuel_capacity: 50.0,
                fuel_consumption: 0.12,
            },
        );
        Self {
            vehicle_configs: configs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
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

/// Performance settings configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PerformanceSettingsConfig {
    pub culling_distance: f32,
    pub max_entities: u32,
    pub vsync: bool,
    pub target_fps: u32,
    pub quality_level: QualityLevel,
}

impl Default for PerformanceSettingsConfig {
    fn default() -> Self {
        Self {
            culling_distance: 1000.0,
            max_entities: 10000,
            vsync: true,
            target_fps: 60,
            quality_level: QualityLevel::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

/// Audio settings configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct AudioSettingsConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub audio_quality: AudioQuality,
    pub spatial_audio: bool,
}

impl Default for AudioSettingsConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.9,
            voice_volume: 1.0,
            audio_quality: AudioQuality::High,
            spatial_audio: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum AudioQuality {
    Low,
    Medium,
    High,
}

/// Physics constants configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PhysicsConstantsConfig {
    pub gravity: Vec3,
    pub air_density: f32,
    pub drag_coefficient: f32,
    pub friction_coefficient: f32,
    pub restitution_coefficient: f32,
    pub time_step: f32,
}

impl Default for PhysicsConstantsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            air_density: 1.225,
            drag_coefficient: 0.3,
            friction_coefficient: 0.7,
            restitution_coefficient: 0.2,
            time_step: 1.0 / 60.0,
        }
    }
}

/// World generation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct WorldGenerationConfig {
    pub city_size: f32,
    pub building_density: f32,
    pub road_width: f32,
    pub block_size: f32,
    pub max_building_height: f32,
    pub terrain_complexity: f32,
    pub water_level: f32,
}

impl Default for WorldGenerationConfig {
    fn default() -> Self {
        Self {
            city_size: 10000.0,
            building_density: 0.6,
            road_width: 8.0,
            block_size: 100.0,
            max_building_height: 200.0,
            terrain_complexity: 0.5,
            water_level: 0.0,
        }
    }
}

/// Vehicle physics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct VehiclePhysicsConfig {
    pub suspension_stiffness: f32,
    pub suspension_damping: f32,
    pub wheel_friction: f32,
    pub downforce_coefficient: f32,
    pub center_of_mass_offset: Vec3,
    pub differential_ratio: f32,
}

impl Default for VehiclePhysicsConfig {
    fn default() -> Self {
        Self {
            suspension_stiffness: 80000.0,
            suspension_damping: 8000.0,
            wheel_friction: 1.0,
            downforce_coefficient: 0.3,
            center_of_mass_offset: Vec3::new(0.0, -0.5, 0.0),
            differential_ratio: 3.42,
        }
    }
}

/// LOD (Level of Detail) configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct LodConfig {
    pub vehicle_lod_distances: Vec<f32>,
    pub building_lod_distances: Vec<f32>,
    pub npc_lod_distances: Vec<f32>,
    pub texture_quality_levels: Vec<f32>,
    pub geometry_quality_levels: Vec<f32>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            vehicle_lod_distances: vec![50.0, 150.0, 300.0],
            building_lod_distances: vec![100.0, 300.0, 1000.0],
            npc_lod_distances: vec![25.0, 100.0, 200.0],
            texture_quality_levels: vec![1.0, 0.75, 0.5, 0.25],
            geometry_quality_levels: vec![1.0, 0.8, 0.6, 0.4],
        }
    }
}

/// Performance configuration (tuning-specific)
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PerformanceConfig {
    pub enable_culling: bool,
    pub occlusion_culling: bool,
    pub frustum_culling: bool,
    pub distance_culling: bool,
    pub batch_size: u32,
    pub thread_count: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_culling: true,
            occlusion_culling: true,
            frustum_culling: true,
            distance_culling: true,
            batch_size: 1000,
            thread_count: 4,
        }
    }
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PerformanceTuningConfig {
    pub frame_rate_limit: u32,
    pub adaptive_quality: bool,
    pub dynamic_resolution: bool,
    pub resolution_scale: f32,
    pub shadow_quality: ShadowQuality,
    pub anti_aliasing: AntiAliasing,
}

impl Default for PerformanceTuningConfig {
    fn default() -> Self {
        Self {
            frame_rate_limit: 60,
            adaptive_quality: true,
            dynamic_resolution: false,
            resolution_scale: 1.0,
            shadow_quality: ShadowQuality::High,
            anti_aliasing: AntiAliasing::FXAA,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum AntiAliasing {
    Off,
    FXAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
}

/// Visual effects configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct VisualEffectsConfig {
    pub particle_count: u32,
    pub explosion_effects: bool,
    pub weather_effects: bool,
    pub day_night_cycle: bool,
    pub volumetric_fog: bool,
    pub bloom_intensity: f32,
    pub motion_blur: bool,
}

impl Default for VisualEffectsConfig {
    fn default() -> Self {
        Self {
            particle_count: 5000,
            explosion_effects: true,
            weather_effects: true,
            day_night_cycle: true,
            volumetric_fog: true,
            bloom_intensity: 0.5,
            motion_blur: false,
        }
    }
}

/// Main game configuration (master config)
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct GameConfigAsset {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub debug_mode: bool,
    pub log_level: LogLevel,
    pub physics_enabled: bool,
    pub audio_enabled: bool,
}

impl Default for GameConfigAsset {
    fn default() -> Self {
        Self {
            window_title: "AMP Game".to_string(),
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            debug_mode: false,
            log_level: LogLevel::Info,
            physics_enabled: true,
            audio_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Camera settings configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct CameraSettingsConfig {
    pub field_of_view: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub follow_distance: f32,
    pub follow_height: f32,
    pub look_ahead_distance: f32,
    pub smoothing_factor: f32,
}

impl Default for CameraSettingsConfig {
    fn default() -> Self {
        Self {
            field_of_view: 60.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            follow_distance: 8.0,
            follow_height: 3.0,
            look_ahead_distance: 5.0,
            smoothing_factor: 0.8,
        }
    }
}

/// NPC behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct NpcBehaviorConfig {
    pub max_npcs: u32,
    pub spawn_radius: f32,
    pub despawn_radius: f32,
    pub walking_speed: f32,
    pub running_speed: f32,
    pub reaction_time: f32,
    pub aggression_level: f32,
}

impl Default for NpcBehaviorConfig {
    fn default() -> Self {
        Self {
            max_npcs: 200,
            spawn_radius: 100.0,
            despawn_radius: 150.0,
            walking_speed: 1.5,
            running_speed: 4.0,
            reaction_time: 0.5,
            aggression_level: 0.3,
        }
    }
}

/// UI settings configuration
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct UiSettingsConfig {
    pub hud_enabled: bool,
    pub minimap_enabled: bool,
    pub minimap_size: f32,
    pub speedometer_enabled: bool,
    pub health_bar_enabled: bool,
    pub ui_scale: f32,
    pub menu_animations: bool,
}

impl Default for UiSettingsConfig {
    fn default() -> Self {
        Self {
            hud_enabled: true,
            minimap_enabled: true,
            minimap_size: 200.0,
            speedometer_enabled: true,
            health_bar_enabled: true,
            ui_scale: 1.0,
            menu_animations: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_stats_config_roundtrip() {
        let config = VehicleStatsConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: VehicleStatsConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(
            config.vehicle_configs.len(),
            deserialized.vehicle_configs.len()
        );
    }

    #[test]
    fn test_performance_settings_config_roundtrip() {
        let config = PerformanceSettingsConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: PerformanceSettingsConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config.target_fps, deserialized.target_fps);
    }

    #[test]
    fn test_audio_settings_config_roundtrip() {
        let config = AudioSettingsConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: AudioSettingsConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config.master_volume, deserialized.master_volume);
    }

    #[test]
    fn test_physics_constants_config_roundtrip() {
        let config = PhysicsConstantsConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: PhysicsConstantsConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config.gravity, deserialized.gravity);
    }

    #[test]
    fn test_game_config_asset_roundtrip() {
        let config = GameConfigAsset::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: GameConfigAsset = ron::from_str(&serialized).unwrap();
        assert_eq!(config.window_title, deserialized.window_title);
    }
}
