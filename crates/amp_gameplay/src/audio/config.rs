//! Audio configuration and settings
//!
//! Configuration structures for audio system behavior and volume levels.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Global audio configuration
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sfx_volume: f32,
    /// Engine audio volume (0.0 to 1.0)
    pub engine_volume: f32,
    /// Environmental audio volume (0.0 to 1.0)
    pub environmental_volume: f32,
    /// Voice/dialog volume (0.0 to 1.0)
    pub voice_volume: f32,
    /// UI sound volume (0.0 to 1.0)
    pub ui_volume: f32,
    /// 3D audio settings
    pub spatial_audio: SpatialAudioConfig,
    /// Dynamic range compression
    pub dynamic_range_compression: bool,
    /// Audio quality settings
    pub quality: AudioQualityConfig,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.9,
            engine_volume: 0.7,
            environmental_volume: 0.6,
            voice_volume: 1.0,
            ui_volume: 0.8,
            spatial_audio: SpatialAudioConfig::default(),
            dynamic_range_compression: true,
            quality: AudioQualityConfig::default(),
        }
    }
}

/// 3D spatial audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioConfig {
    /// Enable 3D positional audio
    pub enabled: bool,
    /// Doppler effect strength
    pub doppler_factor: f32,
    /// Distance attenuation model
    pub distance_model: DistanceModel,
    /// Reference distance for attenuation
    pub reference_distance: f32,
    /// Maximum distance for audio
    pub max_distance: f32,
    /// Rolloff factor for distance attenuation
    pub rolloff_factor: f32,
}

impl Default for SpatialAudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            doppler_factor: 1.0,
            distance_model: DistanceModel::Linear,
            reference_distance: 1.0,
            max_distance: 1000.0,
            rolloff_factor: 1.0,
        }
    }
}

/// Audio distance attenuation models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceModel {
    Linear,
    Inverse,
    Exponential,
}

/// Audio quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Bit depth
    pub bit_depth: u16,
    /// Buffer size (samples)
    pub buffer_size: u32,
    /// Maximum simultaneous audio sources
    pub max_audio_sources: u32,
    /// Enable audio streaming for large files
    pub streaming_enabled: bool,
    /// Streaming buffer size
    pub streaming_buffer_size: u32,
}

impl Default for AudioQualityConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bit_depth: 16,
            buffer_size: 1024,
            max_audio_sources: 32,
            streaming_enabled: true,
            streaming_buffer_size: 8192,
        }
    }
}

/// Audio mixer configuration for different categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMixerConfig {
    /// EQ settings for different frequency bands
    pub equalizer: EqualizerConfig,
    /// Reverb settings
    pub reverb: ReverbConfig,
    /// Compression settings
    pub compression: CompressionConfig,
    /// Per-category audio settings
    pub category_settings: Vec<CategoryAudioConfig>,
}

/// Equalizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerConfig {
    /// Enable EQ
    pub enabled: bool,
    /// Low frequency gain (-20.0 to 20.0 dB)
    pub low_gain: f32,
    /// Mid frequency gain (-20.0 to 20.0 dB)
    pub mid_gain: f32,
    /// High frequency gain (-20.0 to 20.0 dB)
    pub high_gain: f32,
    /// Low frequency cutoff (Hz)
    pub low_cutoff: f32,
    /// High frequency cutoff (Hz)
    pub high_cutoff: f32,
}

impl Default for EqualizerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            low_gain: 0.0,
            mid_gain: 0.0,
            high_gain: 0.0,
            low_cutoff: 250.0,
            high_cutoff: 4000.0,
        }
    }
}

/// Reverb configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbConfig {
    /// Enable reverb
    pub enabled: bool,
    /// Room size (0.0 to 1.0)
    pub room_size: f32,
    /// Damping factor (0.0 to 1.0)
    pub damping: f32,
    /// Wet/dry mix (0.0 to 1.0)
    pub wet_level: f32,
    /// Dry level (0.0 to 1.0)
    pub dry_level: f32,
}

impl Default for ReverbConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Threshold level (dB)
    pub threshold: f32,
    /// Compression ratio
    pub ratio: f32,
    /// Attack time (ms)
    pub attack_time: f32,
    /// Release time (ms)
    pub release_time: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: -12.0,
            ratio: 4.0,
            attack_time: 5.0,
            release_time: 100.0,
        }
    }
}

/// Per-category audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAudioConfig {
    /// Audio category
    pub category: String,
    /// Volume multiplier
    pub volume_multiplier: f32,
    /// Pitch multiplier
    pub pitch_multiplier: f32,
    /// Enable 3D positioning
    pub spatial_enabled: bool,
    /// Priority level (higher = more important)
    pub priority: i32,
}

impl Default for CategoryAudioConfig {
    fn default() -> Self {
        Self {
            category: "default".to_string(),
            volume_multiplier: 1.0,
            pitch_multiplier: 1.0,
            spatial_enabled: false,
            priority: 0,
        }
    }
}
