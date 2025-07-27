//! Advanced Spawn Budget Policy - Enhanced with GPU Culling Integration
//!
//! **Oracle's Advanced Vision**: Frame-rate adaptive spawning with GPU occlusion awareness
//!
//! This module extends the base spawn budget system from amp_gameplay with:
//! - **Frame Rate Adaptation**: Dynamic spawn limits based on current FPS
//! - **GPU Occlusion Integration**: Spawn decisions informed by visibility queries
//! - **Distance-Based Culling**: Enhanced position-aware budget allocation
//! - **Performance Feedback Loop**: Real-time budget adjustment based on system load
//!
//! ## Architecture Integration
//!
//! - **Primary**: amp_engine (orchestration and GPU integration)
//! - **Supporting**: amp_physics (occlusion queries), amp_math (distance calculations)
//! - **Configuration**: config_core (adaptive budget curves)

pub mod adaptive;
pub mod gpu_integration;
pub mod integration;
pub mod performance_feedback;
pub mod plugin;

#[cfg(test)]
mod simple_tests;

pub use adaptive::*;
pub use gpu_integration::*;
pub use integration::*;
pub use performance_feedback::*;
pub use plugin::*;

use amp_gameplay::spawn_budget_policy::{BiomeType, EntityType, SpawnBudgetPolicy};
use bevy::prelude::*;

/// Advanced spawn budget configuration with adaptive features
#[derive(Resource, Debug, Clone)]
pub struct AdvancedSpawnBudgetConfig {
    /// Frame rate adaptation settings
    pub frame_rate_adaptation: FrameRateAdaptationConfig,
    /// GPU occlusion integration settings
    pub occlusion_integration: OcclusionIntegrationConfig,
    /// Distance-based spawn curves
    pub distance_curves: DistanceCurveConfig,
    /// Performance feedback settings
    pub performance_feedback: PerformanceFeedbackConfig,
}

/// Frame rate adaptation configuration
#[derive(Debug, Clone)]
pub struct FrameRateAdaptationConfig {
    /// Target frame rate for optimal spawning
    pub target_fps: f32,
    /// FPS threshold below which spawning is reduced
    pub low_fps_threshold: f32,
    /// FPS threshold above which spawning can be increased
    pub high_fps_threshold: f32,
    /// Maximum reduction factor for spawn limits (0.0-1.0)
    pub max_reduction_factor: f32,
    /// Maximum increase factor for spawn limits (1.0-2.0)
    pub max_increase_factor: f32,
    /// Adaptation smoothing factor (0.0-1.0)
    pub smoothing_factor: f32,
}

/// GPU occlusion integration configuration
#[derive(Debug, Clone)]
pub struct OcclusionIntegrationConfig {
    /// Enable occlusion-aware spawning
    pub enabled: bool,
    /// Distance threshold for occlusion queries
    pub occlusion_query_distance: f32,
    /// Minimum visibility time before spawning (seconds)
    pub min_visibility_duration: f32,
    /// Occlusion test sampling rate (queries per second)
    pub sampling_rate: f32,
}

/// Distance-based spawn curve configuration
#[derive(Debug, Clone)]
pub struct DistanceCurveConfig {
    /// Spawn probability curves by entity type
    pub building_curve: DistanceCurve,
    pub vehicle_curve: DistanceCurve,
    pub npc_curve: DistanceCurve,
    pub tree_curve: DistanceCurve,
    pub particle_curve: DistanceCurve,
}

/// Distance curve for spawn probability
#[derive(Debug, Clone)]
pub struct DistanceCurve {
    /// Distance points (in meters)
    pub distances: Vec<f32>,
    /// Spawn probability at each distance (0.0-1.0)
    pub probabilities: Vec<f32>,
}

/// Performance feedback loop configuration
#[derive(Debug, Clone)]
pub struct PerformanceFeedbackConfig {
    /// Enable performance-responsive budgets
    pub enabled: bool,
    /// Performance measurement window (seconds)
    pub measurement_window: f32,
    /// CPU usage threshold for spawn reduction
    pub cpu_threshold: f32,
    /// GPU usage threshold for spawn reduction
    pub gpu_threshold: f32,
    /// Memory usage threshold for spawn reduction
    pub memory_threshold: f32,
}

/// Advanced spawn budget metrics
#[derive(Resource, Debug, Default)]
pub struct AdvancedSpawnMetrics {
    /// Current frame rate measurement
    pub current_fps: f32,
    /// Frame rate adaptation factor
    pub fps_adaptation_factor: f32,
    /// Occlusion-based spawn rejections
    pub occlusion_rejections: u64,
    /// Distance-based spawn rejections
    pub distance_rejections: u64,
    /// Performance-based spawn rejections
    pub performance_rejections: u64,
    /// Average spawn processing time (ms)
    pub avg_spawn_processing_time: f32,
    /// GPU culling integration status
    pub gpu_culling_active: bool,
}

impl Default for AdvancedSpawnBudgetConfig {
    fn default() -> Self {
        Self {
            frame_rate_adaptation: FrameRateAdaptationConfig {
                target_fps: 60.0,
                low_fps_threshold: 45.0,
                high_fps_threshold: 75.0,
                max_reduction_factor: 0.3,
                max_increase_factor: 1.5,
                smoothing_factor: 0.1,
            },
            occlusion_integration: OcclusionIntegrationConfig {
                enabled: true,
                occlusion_query_distance: 500.0,
                min_visibility_duration: 0.5,
                sampling_rate: 30.0,
            },
            distance_curves: DistanceCurveConfig::default(),
            performance_feedback: PerformanceFeedbackConfig {
                enabled: true,
                measurement_window: 2.0,
                cpu_threshold: 0.8,
                gpu_threshold: 0.85,
                memory_threshold: 0.9,
            },
        }
    }
}

impl Default for DistanceCurveConfig {
    fn default() -> Self {
        Self {
            building_curve: DistanceCurve {
                distances: vec![0.0, 100.0, 300.0, 500.0, 1000.0],
                probabilities: vec![1.0, 0.9, 0.7, 0.3, 0.0],
            },
            vehicle_curve: DistanceCurve {
                distances: vec![0.0, 50.0, 150.0, 300.0, 500.0],
                probabilities: vec![1.0, 0.8, 0.5, 0.2, 0.0],
            },
            npc_curve: DistanceCurve {
                distances: vec![0.0, 30.0, 100.0, 200.0, 400.0],
                probabilities: vec![1.0, 0.7, 0.4, 0.1, 0.0],
            },
            tree_curve: DistanceCurve {
                distances: vec![0.0, 80.0, 200.0, 400.0, 800.0],
                probabilities: vec![1.0, 0.9, 0.8, 0.5, 0.1],
            },
            particle_curve: DistanceCurve {
                distances: vec![0.0, 25.0, 75.0, 150.0, 300.0],
                probabilities: vec![1.0, 0.6, 0.3, 0.1, 0.0],
            },
        }
    }
}
