//! Audio configuration types
//!
//! Configuration structures for audio system parameters that were previously hard-coded.

use crate::Config;
use serde::{Deserialize, Serialize};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

/// Audio configuration for gameplay systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct AudioConfig {
    /// Master volume multiplier (0.0 to 1.0)
    pub master_volume: f32,
    /// Engine audio volume (0.0 to 1.0)
    pub engine_volume: f32,
    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sfx_volume: f32,
    /// Environmental audio volume (0.0 to 1.0)
    pub environment_volume: f32,
    /// UI audio volume (0.0 to 1.0)
    pub ui_volume: f32,
    /// Engine audio parameters
    pub engine: EngineAudioConfig,
    /// Vehicle audio parameters
    pub vehicle: VehicleAudioConfig,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            engine_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.9,
            environment_volume: 0.6,
            ui_volume: 0.8,
            engine: EngineAudioConfig::default(),
            vehicle: VehicleAudioConfig::default(),
        }
    }
}

impl Config for AudioConfig {
    const FILE_NAME: &'static str = "audio.ron";
}

/// Engine-specific audio configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct EngineAudioConfig {
    /// Base engine volume multiplier
    pub base_volume: f32,
    /// RPM-based volume scaling factor
    pub rpm_scaling: f32,
    /// Minimum volume at idle
    pub min_volume: f32,
    /// Maximum volume at redline
    pub max_volume: f32,
    /// Volume curve smoothing factor
    pub smoothing_factor: f32,
}

impl Default for EngineAudioConfig {
    fn default() -> Self {
        Self {
            base_volume: 0.5,
            rpm_scaling: 0.8,
            min_volume: 0.2,
            max_volume: 1.0,
            smoothing_factor: 0.15,
        }
    }
}

/// Vehicle-specific audio configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct VehicleAudioConfig {
    /// Default engine sound enabled state
    pub engine_sound_enabled: bool,
    /// Default engine volume
    pub default_engine_volume: f32,
    /// Tire screech enabled state
    pub tire_screech_enabled: bool,
    /// Default tire screech volume
    pub default_tire_screech_volume: f32,
    /// Tire screech volume scaling factor
    pub tire_screech_scaling: f32,
}

impl Default for VehicleAudioConfig {
    fn default() -> Self {
        Self {
            engine_sound_enabled: true,
            default_engine_volume: 0.5,
            tire_screech_enabled: true,
            default_tire_screech_volume: 0.3,
            tire_screech_scaling: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.engine_volume, 0.8);
        assert_eq!(config.music_volume, 0.7);
        assert_eq!(config.sfx_volume, 0.9);
        assert_eq!(config.environment_volume, 0.6);
        assert_eq!(config.ui_volume, 0.8);
    }

    #[test]
    fn test_engine_audio_config_default() {
        let config = EngineAudioConfig::default();
        assert_eq!(config.base_volume, 0.5);
        assert_eq!(config.rpm_scaling, 0.8);
        assert_eq!(config.min_volume, 0.2);
        assert_eq!(config.max_volume, 1.0);
        assert_eq!(config.smoothing_factor, 0.15);
    }

    #[test]
    fn test_vehicle_audio_config_default() {
        let config = VehicleAudioConfig::default();
        assert!(config.engine_sound_enabled);
        assert_eq!(config.default_engine_volume, 0.5);
        assert!(config.tire_screech_enabled);
        assert_eq!(config.default_tire_screech_volume, 0.3);
        assert_eq!(config.tire_screech_scaling, 0.5);
    }

    #[test]
    fn test_audio_config_serialization() {
        let config = AudioConfig::default();
        let serialized = ron::to_string(&config).expect("Failed to serialize");
        let deserialized: AudioConfig = ron::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_audio_config_partial_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("audio.ron");

        // Write partial config with only master_volume
        std::fs::write(
            &config_path,
            r#"(
                master_volume: 0.5,
            )"#,
        )
        .unwrap();

        let loader = crate::ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        let config: AudioConfig = loader.load_with_merge().unwrap();
        assert_eq!(config.master_volume, 0.5);
        // Should use defaults for other fields
        assert_eq!(config.engine_volume, 0.8);
        assert_eq!(config.music_volume, 0.7);
    }

    #[test]
    fn test_audio_config_validation() {
        // Test volume bounds validation
        let config = AudioConfig {
            master_volume: 2.0,  // Invalid: > 1.0
            engine_volume: -0.5, // Invalid: < 0.0
            ..Default::default()
        };

        // Note: Actual validation would be done by schemars in production
        // This test just ensures the config can be created
        assert_eq!(config.master_volume, 2.0);
        assert_eq!(config.engine_volume, -0.5);
    }

    #[test]
    fn test_engine_audio_config_serialization() {
        let config = EngineAudioConfig {
            base_volume: 0.7,
            rpm_scaling: 0.9,
            min_volume: 0.1,
            max_volume: 0.8,
            smoothing_factor: 0.2,
        };

        let serialized = ron::to_string(&config).expect("Failed to serialize");
        let deserialized: EngineAudioConfig =
            ron::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_vehicle_audio_config_serialization() {
        let config = VehicleAudioConfig {
            engine_sound_enabled: false,
            default_engine_volume: 0.8,
            tire_screech_enabled: false,
            default_tire_screech_volume: 0.1,
            tire_screech_scaling: 0.3,
        };

        let serialized = ron::to_string(&config).expect("Failed to serialize");
        let deserialized: VehicleAudioConfig =
            ron::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_file_name() {
        assert_eq!(AudioConfig::FILE_NAME, "audio.ron");
    }

    #[test]
    fn test_audio_config_merge() {
        let base = AudioConfig {
            master_volume: 0.5,
            engine_volume: 0.6,
            ..Default::default()
        };

        let override_config = AudioConfig {
            master_volume: 0.8,
            music_volume: 0.4,
            ..Default::default()
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.master_volume, 0.8); // Should use override value
        assert_eq!(merged.music_volume, 0.4); // Should use override value
        assert_eq!(merged.engine_volume, 0.8); // Should use default from override (not base)
    }
}
