//! Comprehensive performance monitoring and diagnostics for amp_render
//!
//! This module provides the PerformanceDiagnosticsPlugin that tracks critical
//! performance metrics and provides automated regression gates for CI.

use bevy::app::{App, Plugin, PostUpdate};
use bevy::diagnostic::{DiagnosticPath, Diagnostics};
use bevy::ecs::system::{Res, ResMut};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::Instant;

#[cfg(test)]
mod tests;

/// Performance monitoring plugin for amp_render
///
/// Tracks critical performance metrics and provides automated regression gates.
/// Integrates with Bevy's diagnostic system for consistent monitoring.
pub struct PerformanceDiagnosticsPlugin;

impl Plugin for PerformanceDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "diagnostics")]
        {
            app.init_resource::<PerformanceDiagnostics>().add_systems(
                PostUpdate,
                (sample_performance_metrics, update_performance_diagnostics).chain(),
            );
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            // Diagnostics disabled - no performance monitoring
            info!("Performance diagnostics disabled (feature 'diagnostics' not enabled)");
        }
    }
}

/// Diagnostic paths for performance metrics
pub struct PerformanceDiagnosticPaths;

impl PerformanceDiagnosticPaths {
    /// Draw calls per frame (target <1000, alarm >1200)
    pub const DRAW_CALLS: DiagnosticPath = DiagnosticPath::const_new("amp_render::draw_calls");

    /// Instance count being rendered (target <75k)
    pub const INSTANCE_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::instance_count");

    /// Total entities in world
    pub const TOTAL_ENTITIES: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::total_entities");

    /// Active point lights (should never exceed 200)
    pub const ACTIVE_POINT_LIGHTS: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::active_point_lights");

    /// Spawn queue length
    pub const SPAWN_QUEUE_LENGTH: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::spawn_queue_length");

    /// Despawn queue length
    pub const DESPAWN_QUEUE_LENGTH: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::despawn_queue_length");

    /// Average update time in milliseconds
    pub const AVERAGE_UPDATE_TIME: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::average_update_time");

    /// GPU culling time in milliseconds
    pub const GPU_CULLING_TIME: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::gpu_culling_time");

    /// Sectors loaded count
    pub const SECTORS_LOADED: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::sectors_loaded");

    /// Memory usage in MB
    pub const MEMORY_USAGE_MB: DiagnosticPath =
        DiagnosticPath::const_new("amp_render::memory_usage_mb");
}

/// Performance thresholds and budgets
pub struct PerformanceBudgets;

impl PerformanceBudgets {
    /// Maximum draw calls per frame (target <1000, alarm >1200)
    pub const MAX_DRAW_CALLS: f64 = 1000.0;
    pub const ALARM_DRAW_CALLS: f64 = 1200.0;

    /// Maximum instance count (target <75k)
    pub const MAX_INSTANCE_COUNT: f64 = 75000.0;

    /// Maximum total entities (regression gate)
    pub const MAX_TOTAL_ENTITIES: f64 = 1200.0;

    /// Maximum active point lights (hard limit)
    pub const MAX_ACTIVE_LIGHTS: f64 = 200.0;

    /// Maximum sectors loaded (regression gate)
    pub const MAX_SECTORS_LOADED: f64 = 200.0;

    /// Maximum average update time (8ms = 125 FPS target)
    pub const MAX_AVERAGE_UPDATE_TIME_MS: f64 = 8.0;

    /// Maximum GPU culling time (0.25ms target)
    pub const MAX_GPU_CULLING_TIME_MS: f64 = 0.25;

    /// Maximum spawn queue length (performance warning)
    pub const MAX_SPAWN_QUEUE_LENGTH: f64 = 1000.0;

    /// Maximum despawn queue length (performance warning)
    pub const MAX_DESPAWN_QUEUE_LENGTH: f64 = 1000.0;

    /// Maximum memory usage in MB (warning threshold)
    pub const MAX_MEMORY_USAGE_MB: f64 = 2048.0;
}

/// Performance diagnostics resource
///
/// Collects performance metrics and provides historical tracking for regression detection.
#[derive(Resource)]
pub struct PerformanceDiagnostics {
    /// Current frame metrics
    pub current_frame: FrameMetrics,

    /// Historical metrics for trend analysis
    pub history: VecDeque<FrameMetrics>,

    /// Frame timing for update time calculation
    pub frame_start: Instant,

    /// Performance warnings
    pub warnings: Vec<PerformanceWarning>,

    /// Hard performance limits enforced
    pub hard_limits_enabled: bool,
}

impl Default for PerformanceDiagnostics {
    fn default() -> Self {
        Self {
            current_frame: FrameMetrics::default(),
            history: VecDeque::with_capacity(60), // Keep 60 frames of history
            frame_start: Instant::now(),
            warnings: Vec::new(),
            hard_limits_enabled: false,
        }
    }
}

/// Performance metrics for a single frame
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    /// Number of draw calls issued
    pub draw_calls: u32,

    /// Number of instances rendered
    pub instance_count: u32,

    /// Total entities in world
    pub total_entities: u32,

    /// Active point lights
    pub active_point_lights: u32,

    /// Spawn queue length
    pub spawn_queue_length: u32,

    /// Despawn queue length
    pub despawn_queue_length: u32,

    /// Frame update time in milliseconds
    pub update_time_ms: f32,

    /// GPU culling time in milliseconds
    pub gpu_culling_time_ms: f32,

    /// Number of sectors loaded
    pub sectors_loaded: u32,

    /// Memory usage in MB
    pub memory_usage_mb: f32,

    /// Timestamp for this frame
    pub timestamp: Instant,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            draw_calls: 0,
            instance_count: 0,
            total_entities: 0,
            active_point_lights: 0,
            spawn_queue_length: 0,
            despawn_queue_length: 0,
            update_time_ms: 0.0,
            gpu_culling_time_ms: 0.0,
            sectors_loaded: 0,
            memory_usage_mb: 0.0,
            timestamp: Instant::now(),
        }
    }
}

/// Performance warning types
#[derive(Debug, Clone)]
pub enum PerformanceWarning {
    /// Draw calls exceeded budget
    DrawCallsBudgetExceeded { current: u32, budget: u32 },

    /// Instance count exceeded budget
    InstanceCountBudgetExceeded { current: u32, budget: u32 },

    /// Active lights exceeded hard limit
    ActiveLightsLimitExceeded { current: u32, limit: u32 },

    /// Update time exceeded target
    UpdateTimeExceeded { current_ms: f32, target_ms: f32 },

    /// GPU culling time exceeded target
    GpuCullingTimeExceeded { current_ms: f32, target_ms: f32 },

    /// Memory usage warning
    MemoryUsageHigh { current_mb: f32, warning_mb: f32 },

    /// Queue length warning
    QueueLengthHigh {
        queue_type: String,
        current: u32,
        warning: u32,
    },
}

impl PerformanceDiagnostics {
    /// Add a performance warning
    pub fn add_warning(&mut self, warning: PerformanceWarning) {
        self.warnings.push(warning);

        // Limit warning history to prevent memory growth
        if self.warnings.len() > 100 {
            self.warnings.remove(0);
        }
    }

    /// Get current performance status
    pub fn get_status(&self) -> PerformanceStatus {
        let current = &self.current_frame;

        // Check critical thresholds
        if current.draw_calls as f64 > PerformanceBudgets::ALARM_DRAW_CALLS {
            return PerformanceStatus::Critical;
        }

        if current.active_point_lights as f64 > PerformanceBudgets::MAX_ACTIVE_LIGHTS {
            return PerformanceStatus::Critical;
        }

        if current.update_time_ms as f64 > PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS * 2.0 {
            return PerformanceStatus::Critical;
        }

        // Check warning thresholds
        if current.draw_calls as f64 > PerformanceBudgets::MAX_DRAW_CALLS
            || current.instance_count as f64 > PerformanceBudgets::MAX_INSTANCE_COUNT
            || current.update_time_ms as f64 > PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS
            || current.gpu_culling_time_ms as f64 > PerformanceBudgets::MAX_GPU_CULLING_TIME_MS
        {
            return PerformanceStatus::Warning;
        }

        PerformanceStatus::Good
    }

    /// Calculate average metrics over history
    pub fn calculate_averages(&self) -> FrameMetrics {
        if self.history.is_empty() {
            return self.current_frame.clone();
        }

        let count = self.history.len() as f32;
        let mut avg = FrameMetrics::default();

        for frame in &self.history {
            avg.draw_calls += frame.draw_calls;
            avg.instance_count += frame.instance_count;
            avg.total_entities += frame.total_entities;
            avg.active_point_lights += frame.active_point_lights;
            avg.spawn_queue_length += frame.spawn_queue_length;
            avg.despawn_queue_length += frame.despawn_queue_length;
            avg.update_time_ms += frame.update_time_ms;
            avg.gpu_culling_time_ms += frame.gpu_culling_time_ms;
            avg.sectors_loaded += frame.sectors_loaded;
            avg.memory_usage_mb += frame.memory_usage_mb;
        }

        avg.draw_calls = (avg.draw_calls as f32 / count) as u32;
        avg.instance_count = (avg.instance_count as f32 / count) as u32;
        avg.total_entities = (avg.total_entities as f32 / count) as u32;
        avg.active_point_lights = (avg.active_point_lights as f32 / count) as u32;
        avg.spawn_queue_length = (avg.spawn_queue_length as f32 / count) as u32;
        avg.despawn_queue_length = (avg.despawn_queue_length as f32 / count) as u32;
        avg.update_time_ms /= count;
        avg.gpu_culling_time_ms /= count;
        avg.sectors_loaded = (avg.sectors_loaded as f32 / count) as u32;
        avg.memory_usage_mb /= count;
        avg.timestamp = Instant::now();

        avg
    }

    /// Check if a specific metric is within budget
    pub fn is_within_budget(&self, metric: &str) -> bool {
        let current = &self.current_frame;

        match metric {
            "draw_calls" => current.draw_calls as f64 <= PerformanceBudgets::MAX_DRAW_CALLS,
            "instance_count" => {
                current.instance_count as f64 <= PerformanceBudgets::MAX_INSTANCE_COUNT
            }
            "total_entities" => {
                current.total_entities as f64 <= PerformanceBudgets::MAX_TOTAL_ENTITIES
            }
            "active_point_lights" => {
                current.active_point_lights as f64 <= PerformanceBudgets::MAX_ACTIVE_LIGHTS
            }
            "sectors_loaded" => {
                current.sectors_loaded as f64 <= PerformanceBudgets::MAX_SECTORS_LOADED
            }
            "update_time_ms" => {
                current.update_time_ms as f64 <= PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS
            }
            "gpu_culling_time_ms" => {
                current.gpu_culling_time_ms as f64 <= PerformanceBudgets::MAX_GPU_CULLING_TIME_MS
            }
            _ => true,
        }
    }

    /// Get color coding for UI display
    pub fn get_metric_color(&self, metric: &str) -> (f32, f32, f32) {
        if self.is_within_budget(metric) {
            (0.0, 1.0, 0.0) // Green
        } else {
            match self.get_status() {
                PerformanceStatus::Good => (0.0, 1.0, 0.0),     // Green
                PerformanceStatus::Warning => (1.0, 1.0, 0.0),  // Yellow
                PerformanceStatus::Critical => (1.0, 0.0, 0.0), // Red
            }
        }
    }
}

/// Performance status indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceStatus {
    /// Performance is within acceptable limits
    Good,

    /// Performance warnings detected
    Warning,

    /// Critical performance issues
    Critical,
}

/// Sample performance metrics from various systems
#[cfg(feature = "diagnostics")]
fn sample_performance_metrics(
    mut diagnostics: ResMut<PerformanceDiagnostics>,
    entities: Query<Entity>,
    point_lights: Query<&PointLight>,
) {
    #[cfg(feature = "profile")]
    let _span = tracy_client::span!("sample_performance_metrics");

    let now = Instant::now();
    let frame_time = now.duration_since(diagnostics.frame_start).as_secs_f32() * 1000.0;

    // Sample current frame metrics
    let mut current_frame = FrameMetrics {
        total_entities: entities.iter().count() as u32,
        active_point_lights: point_lights.iter().count() as u32,
        update_time_ms: frame_time,
        timestamp: now,
        ..Default::default()
    };

    // TODO: Sample from actual render systems
    // For now, use placeholder values that can be updated by other systems
    current_frame.draw_calls = 150; // Will be updated by batching system
    current_frame.instance_count = 25000; // Will be updated by culling system
    current_frame.gpu_culling_time_ms = 0.15; // Will be updated by GPU culling
    current_frame.sectors_loaded = 45; // Will be updated by streaming system
    current_frame.memory_usage_mb = 512.0; // Will be updated by memory tracking
    current_frame.spawn_queue_length = 25; // Will be updated by spawning system
    current_frame.despawn_queue_length = 15; // Will be updated by despawning system

    // Check for performance warnings
    diagnostics.warnings.clear();

    if current_frame.draw_calls as f64 > PerformanceBudgets::MAX_DRAW_CALLS {
        diagnostics.add_warning(PerformanceWarning::DrawCallsBudgetExceeded {
            current: current_frame.draw_calls,
            budget: PerformanceBudgets::MAX_DRAW_CALLS as u32,
        });
    }

    if current_frame.instance_count as f64 > PerformanceBudgets::MAX_INSTANCE_COUNT {
        diagnostics.add_warning(PerformanceWarning::InstanceCountBudgetExceeded {
            current: current_frame.instance_count,
            budget: PerformanceBudgets::MAX_INSTANCE_COUNT as u32,
        });
    }

    if current_frame.active_point_lights as f64 > PerformanceBudgets::MAX_ACTIVE_LIGHTS {
        diagnostics.add_warning(PerformanceWarning::ActiveLightsLimitExceeded {
            current: current_frame.active_point_lights,
            limit: PerformanceBudgets::MAX_ACTIVE_LIGHTS as u32,
        });
    }

    if current_frame.update_time_ms as f64 > PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS {
        diagnostics.add_warning(PerformanceWarning::UpdateTimeExceeded {
            current_ms: current_frame.update_time_ms,
            target_ms: PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS as f32,
        });
    }

    if current_frame.gpu_culling_time_ms as f64 > PerformanceBudgets::MAX_GPU_CULLING_TIME_MS {
        diagnostics.add_warning(PerformanceWarning::GpuCullingTimeExceeded {
            current_ms: current_frame.gpu_culling_time_ms,
            target_ms: PerformanceBudgets::MAX_GPU_CULLING_TIME_MS as f32,
        });
    }

    // Update history
    diagnostics.history.push_back(current_frame.clone());
    if diagnostics.history.len() > 60 {
        diagnostics.history.pop_front();
    }

    diagnostics.current_frame = current_frame;
    diagnostics.frame_start = now;
}

/// Update Bevy diagnostics with performance metrics
#[cfg(feature = "diagnostics")]
fn update_performance_diagnostics(
    diagnostics: Res<PerformanceDiagnostics>,
    mut bevy_diagnostics: Diagnostics,
) {
    let current = &diagnostics.current_frame;

    // Update all diagnostic values
    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::DRAW_CALLS, || {
        current.draw_calls as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::INSTANCE_COUNT, || {
        current.instance_count as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::TOTAL_ENTITIES, || {
        current.total_entities as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS, || {
        current.active_point_lights as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::SPAWN_QUEUE_LENGTH, || {
        current.spawn_queue_length as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::DESPAWN_QUEUE_LENGTH, || {
        current.despawn_queue_length as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME, || {
        current.update_time_ms as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::GPU_CULLING_TIME, || {
        current.gpu_culling_time_ms as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::SECTORS_LOADED, || {
        current.sectors_loaded as f64
    });

    bevy_diagnostics.add_measurement(&PerformanceDiagnosticPaths::MEMORY_USAGE_MB, || {
        current.memory_usage_mb as f64
    });
}

// Re-export for backwards compatibility
pub use PerformanceDiagnosticPaths as PerformanceDiagnosticIds;
