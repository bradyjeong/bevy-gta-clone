//! Audio components for ECS
//!
//! Components for managing audio sources, engine sounds, and environmental audio.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// General audio source component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct AudioSource {
    /// Path to the audio asset
    pub path: String,
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
    /// Whether the audio should loop
    pub looping: bool,
    /// 3D position for spatial audio
    pub position: Vec3,
    /// Maximum audible distance
    pub max_distance: f32,
    /// Audio category for mixing
    pub category: AudioCategory,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            path: String::new(),
            volume: 1.0,
            looping: false,
            position: Vec3::ZERO,
            max_distance: 100.0,
            category: AudioCategory::SoundEffect,
        }
    }
}

/// Audio categories for mixing and volume control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AudioCategory {
    Master,
    Music,
    SoundEffect,
    Engine,
    Environment,
    Voice,
    UI,
}

impl Default for AudioCategory {
    fn default() -> Self {
        Self::SoundEffect
    }
}

/// Engine audio component for vehicle sounds
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct EngineAudio {
    /// Base engine sound path
    pub engine_sound_path: String,
    /// Current engine pitch multiplier
    pub pitch_multiplier: f32,
    /// Base pitch value
    pub base_pitch: f32,
    /// RPM to pitch conversion factor
    pub rpm_pitch_factor: f32,
    /// Engine volume based on throttle
    pub throttle_volume_factor: f32,
    /// Whether engine sound is currently playing
    pub is_playing: bool,
}

impl Default for EngineAudio {
    fn default() -> Self {
        Self {
            engine_sound_path: "audio/engine_default.ogg".to_string(),
            pitch_multiplier: 1.0,
            base_pitch: 0.8,
            rpm_pitch_factor: 0.0002,
            throttle_volume_factor: 0.5,
            is_playing: false,
        }
    }
}

/// Environmental audio component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct EnvironmentalAudio {
    /// Wind sound path
    pub wind_sound_path: String,
    /// Rain sound path
    pub rain_sound_path: String,
    /// Traffic sound path
    pub traffic_sound_path: String,
    /// Current wind intensity
    pub wind_intensity: f32,
    /// Current rain intensity
    pub rain_intensity: f32,
    /// Current traffic density
    pub traffic_density: f32,
}

impl Default for EnvironmentalAudio {
    fn default() -> Self {
        Self {
            wind_sound_path: "audio/wind.ogg".to_string(),
            rain_sound_path: "audio/rain.ogg".to_string(),
            traffic_sound_path: "audio/traffic.ogg".to_string(),
            wind_intensity: 0.0,
            rain_intensity: 0.0,
            traffic_density: 0.0,
        }
    }
}

/// Music system component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct MusicSystem {
    /// Current track path
    pub current_track_path: String,
    /// Playlist of track paths
    pub playlist: Vec<String>,
    /// Current playlist index
    pub current_index: usize,
    /// Whether music is playing
    pub is_playing: bool,
    /// Music volume
    pub volume: f32,
}

impl Default for MusicSystem {
    fn default() -> Self {
        Self {
            current_track_path: String::new(),
            playlist: Vec::new(),
            current_index: 0,
            is_playing: false,
            volume: 0.7,
        }
    }
}

/// Sound effect component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct SoundEffect {
    /// Sound effect path
    pub path: String,
    /// Volume level
    pub volume: f32,
    /// Whether to play once or loop
    pub looping: bool,
    /// 3D position
    pub position: Vec3,
}

impl Default for SoundEffect {
    fn default() -> Self {
        Self {
            path: String::new(),
            volume: 1.0,
            looping: false,
            position: Vec3::ZERO,
        }
    }
}
