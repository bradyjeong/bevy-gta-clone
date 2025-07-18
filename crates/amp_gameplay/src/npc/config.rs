//! NPC configuration system
//!
//! Loads and manages NPC behavior configuration from assets.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// NPC configuration loaded from assets/config/npc_behavior.ron
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Resource)]
pub struct NpcConfig {
    /// NPC behavior configuration
    pub npc_behavior: NpcBehaviorConfig,
}

impl Default for NpcConfig {
    fn default() -> Self {
        Self {
            npc_behavior: NpcBehaviorConfig::default(),
        }
    }
}

/// Main NPC behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NpcBehaviorConfig {
    /// Physical properties
    pub physical: PhysicalConfig,
    /// Movement properties
    pub movement: MovementConfig,
    /// Emotional system
    pub emotions: EmotionsConfig,
    /// AI behavior
    pub ai: AIConfig,
    /// Spawn settings
    pub spawn: SpawnConfig,
    /// Appearance variety
    pub appearance: AppearanceConfig,
    /// Update intervals for distance-based processing
    pub update_intervals: UpdateIntervalsConfig,
}

impl Default for NpcBehaviorConfig {
    fn default() -> Self {
        Self {
            physical: PhysicalConfig::default(),
            movement: MovementConfig::default(),
            emotions: EmotionsConfig::default(),
            ai: AIConfig::default(),
            spawn: SpawnConfig::default(),
            appearance: AppearanceConfig::default(),
            update_intervals: UpdateIntervalsConfig::default(),
        }
    }
}

/// Physical properties configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PhysicalConfig {
    pub default_height: f32,
    pub default_build: f32,
    pub capsule_radius: f32,
    pub capsule_height: f32,
    pub mass: f32,
}

impl Default for PhysicalConfig {
    fn default() -> Self {
        Self {
            default_height: 1.8,
            default_build: 1.0,
            capsule_radius: 0.4,
            capsule_height: 0.8,
            mass: 70.0,
        }
    }
}

/// Movement configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct MovementConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub turning_speed: f32,
    pub avoidance_distance: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 1.5,
            run_speed: 3.0,
            max_speed: 5.0,
            acceleration: 2.0,
            deceleration: 4.0,
            turning_speed: 1.8,
            avoidance_distance: 5.0,
        }
    }
}

/// Emotions configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct EmotionsConfig {
    pub energy_levels: EnergyLevelsConfig,
    pub stress_levels: StressLevelsConfig,
    pub mood_change_cooldown: f32,
}

impl Default for EmotionsConfig {
    fn default() -> Self {
        Self {
            energy_levels: EnergyLevelsConfig::default(),
            stress_levels: StressLevelsConfig::default(),
            mood_change_cooldown: 5.0,
        }
    }
}

/// Energy levels configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct EnergyLevelsConfig {
    pub max_energy: f32,
    pub resting_threshold: f32,
    pub tired_threshold: f32,
    pub energetic_threshold: f32,
    pub energy_drain_rate: f32,
    pub energy_recovery_rate: f32,
}

impl Default for EnergyLevelsConfig {
    fn default() -> Self {
        Self {
            max_energy: 100.0,
            resting_threshold: 20.0,
            tired_threshold: 30.0,
            energetic_threshold: 80.0,
            energy_drain_rate: 5.0,
            energy_recovery_rate: 15.0,
        }
    }
}

/// Stress levels configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StressLevelsConfig {
    pub max_stress: f32,
    pub calm_threshold: f32,
    pub stressed_threshold: f32,
    pub panic_threshold: f32,
    pub stress_buildup_rate: f32,
    pub stress_recovery_rate: f32,
}

impl Default for StressLevelsConfig {
    fn default() -> Self {
        Self {
            max_stress: 100.0,
            calm_threshold: 30.0,
            stressed_threshold: 40.0,
            panic_threshold: 70.0,
            stress_buildup_rate: 10.0,
            stress_recovery_rate: 5.0,
        }
    }
}

/// AI behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AIConfig {
    pub decision_interval: f32,
    pub path_recalculation_interval: f32,
    pub reaction_time: f32,
    pub attention_span: f32,
    pub memory_duration: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            decision_interval: 2.0,
            path_recalculation_interval: 5.0,
            reaction_time: 0.5,
            attention_span: 30.0,
            memory_duration: 60.0,
        }
    }
}

/// Spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SpawnConfig {
    pub max_npcs: u32,
    pub spawn_interval: f32,
    pub spawn_radius: f32,
    pub despawn_distance: f32,
    pub min_spawn_distance: f32,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            max_npcs: 100,
            spawn_interval: 5.0,
            spawn_radius: 900.0,
            despawn_distance: 1200.0,
            min_spawn_distance: 50.0,
        }
    }
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AppearanceConfig {
    pub skin_tones: Vec<Color>,
    pub hair_colors: Vec<Color>,
    pub clothing_colors: Vec<Color>,
    pub height_variation: (f32, f32),
    pub build_variation: (f32, f32),
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            skin_tones: vec![
                Color::srgba(0.8, 0.7, 0.6, 1.0),
                Color::srgba(0.9, 0.8, 0.7, 1.0),
                Color::srgba(0.7, 0.6, 0.5, 1.0),
                Color::srgba(0.6, 0.5, 0.4, 1.0),
            ],
            hair_colors: vec![
                Color::srgba(0.3, 0.2, 0.1, 1.0),
                Color::srgba(0.1, 0.1, 0.1, 1.0),
                Color::srgba(0.6, 0.5, 0.2, 1.0),
                Color::srgba(0.4, 0.1, 0.1, 1.0),
            ],
            clothing_colors: vec![
                Color::srgba(0.3, 0.3, 0.7, 1.0),
                Color::srgba(0.7, 0.3, 0.3, 1.0),
                Color::srgba(0.3, 0.7, 0.3, 1.0),
                Color::srgba(0.2, 0.2, 0.4, 1.0),
            ],
            height_variation: (0.8, 1.2),
            build_variation: (0.7, 1.3),
        }
    }
}

/// Update intervals configuration for distance-based processing
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateIntervalsConfig {
    /// Distance threshold for close NPCs
    pub close_distance: f32,
    /// Distance threshold for far NPCs
    pub far_distance: f32,
    /// Update interval for close NPCs (seconds)
    pub close_interval: f32,
    /// Update interval for medium distance NPCs (seconds)
    pub medium_interval: f32,
    /// Update interval for far NPCs (seconds)
    pub far_interval: f32,
}

impl Default for UpdateIntervalsConfig {
    fn default() -> Self {
        Self {
            close_distance: 50.0,
            far_distance: 150.0,
            close_interval: 0.0167, // ~60 FPS (every frame)
            medium_interval: 0.25,  // ~4 FPS (every 15 frames)
            far_interval: 1.0,      // ~1 FPS (every 60 frames)
        }
    }
}

/// Distance categories for NPC processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DistanceCategory {
    Close,  // < 50m
    Medium, // 50-150m
    Far,    // > 150m
}

impl DistanceCategory {
    /// Get the distance category from distance value
    pub fn from_distance(distance: f32, config: &UpdateIntervalsConfig) -> Self {
        if distance < config.close_distance {
            Self::Close
        } else if distance < config.far_distance {
            Self::Medium
        } else {
            Self::Far
        }
    }

    /// Get the frame interval for this distance category
    pub fn frame_interval(&self, config: &UpdateIntervalsConfig) -> u32 {
        match self {
            Self::Close => 1,   // Every frame
            Self::Medium => 15, // Every 15 frames
            Self::Far => 60,    // Every 60 frames
        }
    }

    /// Get the update interval in seconds
    pub fn update_interval(&self, config: &UpdateIntervalsConfig) -> f32 {
        match self {
            Self::Close => config.close_interval,
            Self::Medium => config.medium_interval,
            Self::Far => config.far_interval,
        }
    }
}
