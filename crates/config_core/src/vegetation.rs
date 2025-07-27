//! Vegetation rendering configuration types for the game engine.

use crate::{Config, ConfigLoader};
use serde::{Deserialize, Serialize};

/// Configuration for vegetation Level of Detail (LOD) system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VegetationLODConfig {
    /// Distance threshold for full detail level (< distance)
    pub full_distance: f32,
    /// Distance threshold for medium detail level (< distance)
    pub medium_distance: f32,
    /// Distance threshold for billboard level (< distance)
    pub billboard_distance: f32,
    /// Maximum distance before culling (>= distance)
    pub cull_distance: f32,
    /// Enable adaptive LOD based on performance
    pub adaptive_lod: bool,
    /// Target frame time for adaptive LOD (seconds)
    pub target_frame_time: f32,
    /// Update frequency for LOD calculations (Hz)
    pub update_frequency: f32,
    /// Enable performance monitoring
    pub monitor_performance: bool,
}

impl Default for VegetationLODConfig {
    fn default() -> Self {
        Self {
            full_distance: 50.0,
            medium_distance: 150.0,
            billboard_distance: 300.0,
            cull_distance: 500.0,
            adaptive_lod: true,
            target_frame_time: 1.0 / 60.0, // 60 FPS
            update_frequency: 30.0,        // 30 Hz
            monitor_performance: true,
        }
    }
}

impl Config for VegetationLODConfig {
    const FILE_NAME: &'static str = "vegetation_lod.ron";

    fn merge(self, other: Self) -> Self {
        // Field-level merge: use other's values if they differ from defaults
        let defaults = Self::default();
        Self {
            full_distance: if other.full_distance != defaults.full_distance {
                other.full_distance
            } else {
                self.full_distance
            },
            medium_distance: if other.medium_distance != defaults.medium_distance {
                other.medium_distance
            } else {
                self.medium_distance
            },
            billboard_distance: if other.billboard_distance != defaults.billboard_distance {
                other.billboard_distance
            } else {
                self.billboard_distance
            },
            cull_distance: if other.cull_distance != defaults.cull_distance {
                other.cull_distance
            } else {
                self.cull_distance
            },
            adaptive_lod: if other.adaptive_lod != defaults.adaptive_lod {
                other.adaptive_lod
            } else {
                self.adaptive_lod
            },
            target_frame_time: if other.target_frame_time != defaults.target_frame_time {
                other.target_frame_time
            } else {
                self.target_frame_time
            },
            update_frequency: if other.update_frequency != defaults.update_frequency {
                other.update_frequency
            } else {
                self.update_frequency
            },
            monitor_performance: if other.monitor_performance != defaults.monitor_performance {
                other.monitor_performance
            } else {
                self.monitor_performance
            },
        }
    }
}

/// Configuration for vegetation instancing and batching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VegetationInstancingConfig {
    /// Enable GPU instancing for vegetation
    pub enable_instancing: bool,
    /// Maximum instances per batch
    pub max_instances_per_batch: u32,
    /// Batch update frequency (Hz)
    pub batch_update_frequency: f32,
    /// Enable frustum culling for instances
    pub frustum_culling: bool,
    /// Enable distance-based instance culling
    pub distance_culling: bool,
}

impl Default for VegetationInstancingConfig {
    fn default() -> Self {
        Self {
            enable_instancing: true,
            max_instances_per_batch: 1024,
            batch_update_frequency: 10.0, // 10 Hz
            frustum_culling: true,
            distance_culling: true,
        }
    }
}

impl Config for VegetationInstancingConfig {
    const FILE_NAME: &'static str = "vegetation_instancing.ron";

    fn merge(self, other: Self) -> Self {
        let defaults = Self::default();
        Self {
            enable_instancing: if other.enable_instancing != defaults.enable_instancing {
                other.enable_instancing
            } else {
                self.enable_instancing
            },
            max_instances_per_batch: if other.max_instances_per_batch
                != defaults.max_instances_per_batch
            {
                other.max_instances_per_batch
            } else {
                self.max_instances_per_batch
            },
            batch_update_frequency: if other.batch_update_frequency
                != defaults.batch_update_frequency
            {
                other.batch_update_frequency
            } else {
                self.batch_update_frequency
            },
            frustum_culling: if other.frustum_culling != defaults.frustum_culling {
                other.frustum_culling
            } else {
                self.frustum_culling
            },
            distance_culling: if other.distance_culling != defaults.distance_culling {
                other.distance_culling
            } else {
                self.distance_culling
            },
        }
    }
}

/// Combined vegetation configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct VegetationConfig {
    /// LOD configuration
    pub lod: VegetationLODConfig,
    /// Instancing configuration
    pub instancing: VegetationInstancingConfig,
}

impl Config for VegetationConfig {
    const FILE_NAME: &'static str = "vegetation.ron";

    fn merge(self, other: Self) -> Self {
        Self {
            lod: self.lod.merge(other.lod),
            instancing: self.instancing.merge(other.instancing),
        }
    }
}

impl VegetationConfig {
    /// Load vegetation configuration from the filesystem
    pub fn load() -> crate::Result<Self> {
        let loader = ConfigLoader::new();
        loader.load_with_merge()
    }

    /// Get LOD configuration
    pub fn lod(&self) -> &VegetationLODConfig {
        &self.lod
    }

    /// Get instancing configuration
    pub fn instancing(&self) -> &VegetationInstancingConfig {
        &self.instancing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vegetation_lod_config_default() {
        let config = VegetationLODConfig::default();
        assert_eq!(config.full_distance, 50.0);
        assert_eq!(config.medium_distance, 150.0);
        assert_eq!(config.billboard_distance, 300.0);
        assert_eq!(config.cull_distance, 500.0);
        assert!(config.adaptive_lod);
        assert_eq!(config.target_frame_time, 1.0 / 60.0);
        assert_eq!(config.update_frequency, 30.0);
        assert!(config.monitor_performance);
    }

    #[test]
    fn test_vegetation_instancing_config_default() {
        let config = VegetationInstancingConfig::default();
        assert!(config.enable_instancing);
        assert_eq!(config.max_instances_per_batch, 1024);
        assert_eq!(config.batch_update_frequency, 10.0);
        assert!(config.frustum_culling);
        assert!(config.distance_culling);
    }

    #[test]
    fn test_vegetation_config_default() {
        let config = VegetationConfig::default();
        assert_eq!(config.lod.full_distance, 50.0);
        assert!(config.instancing.enable_instancing);
    }

    #[test]
    fn test_vegetation_config_serialization() {
        let config = VegetationConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: VegetationConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_vegetation_config_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vegetation.ron");

        // Write custom config
        std::fs::write(
            &config_path,
            r#"(
                lod: (
                    full_distance: 75.0,
                    medium_distance: 200.0,
                    billboard_distance: 400.0,
                    cull_distance: 800.0,
                    adaptive_lod: false,
                    target_frame_time: 0.020,
                    update_frequency: 20.0,
                    monitor_performance: false,
                ),
                instancing: (
                    enable_instancing: true,
                    max_instances_per_batch: 2048,
                    batch_update_frequency: 15.0,
                    frustum_culling: true,
                    distance_culling: false,
                )
            )"#,
        )
        .unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        let config: VegetationConfig = loader.load_with_merge().unwrap();
        assert_eq!(config.lod.full_distance, 75.0);
        assert_eq!(config.lod.medium_distance, 200.0);
        assert_eq!(config.lod.billboard_distance, 400.0);
        assert_eq!(config.lod.cull_distance, 800.0);
        assert!(!config.lod.adaptive_lod);
        assert_eq!(config.lod.target_frame_time, 0.020);
        assert_eq!(config.lod.update_frequency, 20.0);
        assert!(!config.lod.monitor_performance);

        assert!(config.instancing.enable_instancing);
        assert_eq!(config.instancing.max_instances_per_batch, 2048);
        assert_eq!(config.instancing.batch_update_frequency, 15.0);
        assert!(config.instancing.frustum_culling);
        assert!(!config.instancing.distance_culling);
    }

    #[test]
    fn test_vegetation_config_partial_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("vegetation.ron");

        // Write partial config (only LOD settings)
        std::fs::write(
            &config_path,
            r#"(
                lod: (
                    full_distance: 100.0,
                    adaptive_lod: false,
                )
            )"#,
        )
        .unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        let config: VegetationConfig = loader.load_with_merge().unwrap();

        // Should override specified values
        assert_eq!(config.lod.full_distance, 100.0);
        assert!(!config.lod.adaptive_lod);

        // Should use defaults for unspecified values
        assert_eq!(config.lod.medium_distance, 150.0);
        assert_eq!(config.lod.billboard_distance, 300.0);
        assert!(config.instancing.enable_instancing);
        assert_eq!(config.instancing.max_instances_per_batch, 1024);
    }
}
