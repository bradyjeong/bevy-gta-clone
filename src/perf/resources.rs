use bevy::prelude::*;
use std::time::Instant;

use crate::perf::{
    PerfStatus, CRIT_ENTITY_COUNT, CRIT_FRAME_TIME_MS, CRIT_GPU_CULLING_TIME_MS,
    CRIT_GPU_MEMORY_MB, CRIT_HEAP_USAGE_MB, CRIT_PHYSICS_TIME_MS, CRIT_RENDER_TIME_MS,
    TARGET_FRAME_TIME_MS, TARGET_GPU_CULLING_TIME_MS, TARGET_PHYSICS_TIME_MS,
    TARGET_RENDER_TIME_MS, WARN_ENTITY_COUNT, WARN_FRAME_TIME_MS, WARN_GPU_CULLING_TIME_MS,
    WARN_GPU_MEMORY_MB, WARN_HEAP_USAGE_MB, WARN_PHYSICS_TIME_MS, WARN_RENDER_TIME_MS,
};

impl Default for PerfStatus {
    fn default() -> Self {
        PerfStatus::Good
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PerfThresholds {
    // Frame time thresholds (milliseconds)
    pub frame_time_target_ms: f64,
    pub frame_time_warn_ms: f64,
    pub frame_time_crit_ms: f64,

    // Physics time thresholds (milliseconds)
    pub physics_time_target_ms: f64,
    pub physics_time_warn_ms: f64,
    pub physics_time_crit_ms: f64,

    // Render time thresholds (milliseconds)
    pub render_time_target_ms: f64,
    pub render_time_warn_ms: f64,
    pub render_time_crit_ms: f64,

    // GPU culling time thresholds (milliseconds)
    pub gpu_culling_time_target_ms: f64,
    pub gpu_culling_time_warn_ms: f64,
    pub gpu_culling_time_crit_ms: f64,

    // Memory thresholds (MB)
    pub heap_usage_warn_mb: f64,
    pub heap_usage_crit_mb: f64,
    pub gpu_memory_warn_mb: f64,
    pub gpu_memory_crit_mb: f64,

    // Entity count thresholds
    pub entity_count_warn: u32,
    pub entity_count_crit: u32,
}

impl Default for PerfThresholds {
    fn default() -> Self {
        Self {
            frame_time_target_ms: TARGET_FRAME_TIME_MS,
            frame_time_warn_ms: WARN_FRAME_TIME_MS,
            frame_time_crit_ms: CRIT_FRAME_TIME_MS,

            physics_time_target_ms: TARGET_PHYSICS_TIME_MS,
            physics_time_warn_ms: WARN_PHYSICS_TIME_MS,
            physics_time_crit_ms: CRIT_PHYSICS_TIME_MS,

            render_time_target_ms: TARGET_RENDER_TIME_MS,
            render_time_warn_ms: WARN_RENDER_TIME_MS,
            render_time_crit_ms: CRIT_RENDER_TIME_MS,

            gpu_culling_time_target_ms: TARGET_GPU_CULLING_TIME_MS,
            gpu_culling_time_warn_ms: WARN_GPU_CULLING_TIME_MS,
            gpu_culling_time_crit_ms: CRIT_GPU_CULLING_TIME_MS,

            heap_usage_warn_mb: WARN_HEAP_USAGE_MB,
            heap_usage_crit_mb: CRIT_HEAP_USAGE_MB,
            gpu_memory_warn_mb: WARN_GPU_MEMORY_MB,
            gpu_memory_crit_mb: CRIT_GPU_MEMORY_MB,

            entity_count_warn: WARN_ENTITY_COUNT,
            entity_count_crit: CRIT_ENTITY_COUNT,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct GlobalPerformance {
    // Current performance metrics
    pub frame_time_ms: f64,
    pub physics_time_ms: f64,
    pub render_time_ms: f64,
    pub gpu_culling_time_ms: f64,

    // Memory metrics
    pub heap_usage_mb: f64,
    pub gpu_memory_mb: f64,

    // Entity counts
    pub entity_count: u32,
    pub npc_count: u32,
    pub vehicle_count: u32,
    pub building_count: u32,

    // Performance status
    pub overall_status: PerfStatus,
    pub frame_status: PerfStatus,
    pub physics_status: PerfStatus,
    pub render_status: PerfStatus,

    // Rate limiting for logging
    pub last_status_change: Option<Instant>,
    pub last_logged_status: Option<PerfStatus>,
    pub log_cooldown_secs: f64,
}

impl GlobalPerformance {
    pub fn new() -> Self {
        Self {
            log_cooldown_secs: 1.0, // Only log status changes once per second
            ..Default::default()
        }
    }

    pub fn update_overall_status(&mut self) {
        let worst_status = self
            .frame_status
            .worst_of(self.physics_status)
            .worst_of(self.render_status);

        // Track status changes for rate-limited logging
        if self.overall_status != worst_status {
            self.last_status_change = Some(Instant::now());
        }

        self.overall_status = worst_status;
    }

    pub fn should_log_status(&self) -> bool {
        // Log if status changed and enough time has passed since last log
        if let Some(last_change) = self.last_status_change {
            if let Some(last_logged_status) = self.last_logged_status {
                // Only log if status actually changed from what we last logged
                if last_logged_status == self.overall_status {
                    return false;
                }
            }

            // Respect cooldown period
            let elapsed = last_change.elapsed().as_secs_f64();
            elapsed >= self.log_cooldown_secs
        } else {
            // No status change, don't log
            false
        }
    }

    pub fn mark_status_logged(&mut self) {
        self.last_logged_status = Some(self.overall_status);
    }
}
