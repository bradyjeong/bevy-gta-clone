//! Performance tuning resource for centralized configuration
//!
//! This module provides the `PerformanceTuning` resource that centralizes
//! all performance-related magic numbers and constants into a single
//! configurable resource loaded from configuration files.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Central performance tuning resource that replaces magic numbers
///
/// This resource consolidates all performance-related constants that were
/// previously hardcoded throughout the codebase. It can be loaded from
/// configuration files and modified at runtime.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTuning {
    /// Culling distance thresholds
    pub culling: CullingTuning,
    /// LOD system configuration
    pub lod: LodTuning,
    /// Streaming system configuration
    pub streaming: StreamingTuning,
    /// Performance budgets and limits
    pub budgets: BudgetTuning,
    /// Batch processing configuration
    pub batching: BatchingTuning,
    /// GPU culling configuration
    pub gpu_culling: GpuCullingTuning,
}

/// Culling distance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CullingTuning {
    /// Maximum distance for vehicle culling
    pub vehicle_max_distance: f32,
    /// Maximum distance for building culling
    pub building_max_distance: f32,
    /// Maximum distance for NPC culling
    pub npc_max_distance: f32,
    /// Maximum distance for environment objects
    pub environment_max_distance: f32,
    /// Frustum culling margin
    pub frustum_margin: f32,
}

/// LOD system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodTuning {
    /// LOD transition distances
    pub transition_distances: Vec<f32>,
    /// Hysteresis factor for LOD transitions
    pub hysteresis_factor: f32,
    /// Cross-fade duration for LOD transitions
    pub cross_fade_duration: f32,
    /// Maximum LOD level
    pub max_lod_level: u32,
}

/// Streaming system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingTuning {
    /// Sector size for streaming
    pub sector_size: f32,
    /// Streaming view radius
    pub view_radius: f32,
    /// Maximum entities to spawn per frame
    pub max_spawns_per_frame: usize,
    /// Maximum entities to despawn per frame
    pub max_despawns_per_frame: usize,
    /// Streaming priority threshold
    pub priority_threshold: f32,
}

/// Performance budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTuning {
    /// Maximum active lights
    pub max_active_lights: u32,
    /// Maximum spawn queue length
    pub max_spawn_queue_length: u32,
    /// Maximum batch count per frame
    pub max_batch_count: u32,
    /// Maximum GPU memory usage (MB)
    pub max_gpu_memory_mb: u32,
    /// Frame time budget (ms)
    pub frame_time_budget_ms: f32,
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingTuning {
    /// Maximum instances per batch
    pub max_instances_per_batch: u32,
    /// Batch size threshold for combining
    pub batch_combine_threshold: u32,
    /// Buffer update frequency
    pub buffer_update_frequency: u32,
    /// Instance buffer size
    pub instance_buffer_size: u32,
}

/// GPU culling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCullingTuning {
    /// Work group size for compute shader
    pub work_group_size: u32,
    /// Maximum objects per culling pass
    pub max_objects_per_pass: u32,
    /// Culling time budget (ms)
    pub time_budget_ms: f32,
    /// Enable hierarchical culling
    pub enable_hierarchical: bool,
}

impl Default for PerformanceTuning {
    fn default() -> Self {
        Self {
            culling: CullingTuning {
                vehicle_max_distance: 150.0,
                building_max_distance: 300.0,
                npc_max_distance: 100.0,
                environment_max_distance: 200.0,
                frustum_margin: 10.0,
            },
            lod: LodTuning {
                transition_distances: vec![50.0, 100.0, 200.0, 400.0],
                hysteresis_factor: 0.1,
                cross_fade_duration: 0.5,
                max_lod_level: 4,
            },
            streaming: StreamingTuning {
                sector_size: 64.0,
                view_radius: 512.0,
                max_spawns_per_frame: 50,
                max_despawns_per_frame: 100,
                priority_threshold: 0.5,
            },
            budgets: BudgetTuning {
                max_active_lights: 256,
                max_spawn_queue_length: 1000,
                max_batch_count: 500,
                max_gpu_memory_mb: 2048,
                frame_time_budget_ms: 16.67, // 60 FPS
            },
            batching: BatchingTuning {
                max_instances_per_batch: 1000,
                batch_combine_threshold: 50,
                buffer_update_frequency: 4,
                instance_buffer_size: 10000,
            },
            gpu_culling: GpuCullingTuning {
                work_group_size: 64,
                max_objects_per_pass: 100000,
                time_budget_ms: 0.25,
                enable_hierarchical: true,
            },
        }
    }
}

impl PerformanceTuning {
    /// Load performance tuning from a configuration file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let tuning: Self = ron::from_str(&contents)?;
        Ok(tuning)
    }

    /// Save performance tuning to a configuration file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let contents = ron::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// Plugin for performance tuning resource
pub struct PerformanceTuningPlugin;

impl Plugin for PerformanceTuningPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PerformanceTuning::default());
    }
}
