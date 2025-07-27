//! Frame Rate Adaptive Spawning System
//!
//! **Oracle's Performance Discipline**: Spawn budgets must adapt to system performance
//!
//! This module implements dynamic spawn budget adjustment based on real-time frame rate
//! monitoring and performance feedback.

use crate::spawn_budget::{AdvancedSpawnBudgetConfig, AdvancedSpawnMetrics};
use amp_gameplay::spawn_budget_policy::{FrameLimits, SpawnBudgetPolicy};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Frame rate monitoring and adaptation system
pub fn adaptive_spawn_budget_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    mut metrics: ResMut<AdvancedSpawnMetrics>,
    config: Res<AdvancedSpawnBudgetConfig>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
) {
    // Get current frame rate from diagnostics
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diag| diag.smoothed())
    {
        metrics.current_fps = fps as f32;

        // Calculate adaptation factor based on frame rate
        let adaptation_factor =
            calculate_fps_adaptation_factor(fps as f32, &config.frame_rate_adaptation);

        // Smooth the adaptation factor to prevent jarring changes
        metrics.fps_adaptation_factor = lerp_adaptation_factor(
            metrics.fps_adaptation_factor,
            adaptation_factor,
            config.frame_rate_adaptation.smoothing_factor,
            time.delta_secs(),
        );

        // Apply adaptation to spawn limits
        apply_adaptation_to_policy(&mut policy, metrics.fps_adaptation_factor);

        // Update metrics
        update_adaptation_metrics(&mut metrics, &policy);
    }
}

/// Calculate frame rate adaptation factor
pub fn calculate_fps_adaptation_factor(
    current_fps: f32,
    config: &super::FrameRateAdaptationConfig,
) -> f32 {
    if current_fps < config.low_fps_threshold {
        // Low FPS: reduce spawn budget
        let reduction_ratio =
            (config.low_fps_threshold - current_fps) / (config.low_fps_threshold - 30.0); // Assume 30 FPS as critical minimum
        let reduction_ratio = reduction_ratio.clamp(0.0, 1.0);

        // Interpolate between 1.0 and max_reduction_factor
        1.0 - (reduction_ratio * (1.0 - config.max_reduction_factor))
    } else if current_fps > config.high_fps_threshold {
        // High FPS: potentially increase spawn budget
        let increase_ratio =
            (current_fps - config.high_fps_threshold) / (120.0 - config.high_fps_threshold); // Assume 120 FPS as maximum useful
        let increase_ratio = increase_ratio.clamp(0.0, 1.0);

        // Interpolate between 1.0 and max_increase_factor
        1.0 + (increase_ratio * (config.max_increase_factor - 1.0))
    } else {
        // Within acceptable range: no adaptation needed
        1.0
    }
}

/// Smoothly interpolate adaptation factor changes
fn lerp_adaptation_factor(current: f32, target: f32, smoothing: f32, delta_time: f32) -> f32 {
    let lerp_speed = smoothing * delta_time * 60.0; // Normalize for 60 FPS baseline
    current + (target - current) * lerp_speed.min(1.0)
}

/// Apply adaptation factor to spawn budget policy
fn apply_adaptation_to_policy(policy: &mut SpawnBudgetPolicy, adaptation_factor: f32) {
    // Adjust per-frame spawn limits
    let base_spawns_per_frame = 50; // Default from Oracle's original configuration
    let adapted_spawns = (base_spawns_per_frame as f32 * adaptation_factor).round() as u32;
    let adapted_spawns = adapted_spawns.clamp(5, 100); // Safety bounds

    // Update frame limits
    policy.frame_limits.max_spawns_per_frame = adapted_spawns;

    // Adjust time budget proportionally
    let base_time_budget = 16.67; // 60 FPS target
    policy.frame_limits.time_budget_ms = base_time_budget / adaptation_factor;

    info!(
        "Adaptive spawn budget: factor={:.2}, spawns_per_frame={}, time_budget={:.2}ms",
        adaptation_factor, adapted_spawns, policy.frame_limits.time_budget_ms
    );
}

/// Update adaptation metrics for monitoring
fn update_adaptation_metrics(metrics: &mut AdvancedSpawnMetrics, policy: &SpawnBudgetPolicy) {
    // Track spawn processing efficiency
    let spawn_efficiency = if policy.metrics.total_spawn_requests > 0 {
        policy.metrics.successful_spawns as f32 / policy.metrics.total_spawn_requests as f32
    } else {
        1.0
    };

    // Update average processing time (simplified estimation)
    let estimated_processing_time = policy.frame_limits.time_budget_ms * (1.0 - spawn_efficiency);
    metrics.avg_spawn_processing_time =
        (metrics.avg_spawn_processing_time * 0.9) + (estimated_processing_time * 0.1);
}

/// Performance-based spawn budget scaling system
pub fn performance_budget_scaling_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    mut metrics: ResMut<AdvancedSpawnMetrics>,
    config: Res<AdvancedSpawnBudgetConfig>,
    time: Res<Time>,
) {
    if !config.performance_feedback.enabled {
        return;
    }

    // Get system performance metrics (simplified implementation)
    let system_load = estimate_system_load(&time);

    // Apply performance-based scaling
    if system_load > config.performance_feedback.cpu_threshold {
        // High system load: reduce spawn budget
        let reduction_factor = 0.7; // 30% reduction
        let current_limit = policy.frame_limits.max_spawns_per_frame;
        let reduced_limit = (current_limit as f32 * reduction_factor).round() as u32;
        policy.frame_limits.max_spawns_per_frame = reduced_limit.max(10);

        metrics.performance_rejections += 1;

        warn!(
            "High system load detected ({:.2}), reducing spawn budget to {}",
            system_load, policy.frame_limits.max_spawns_per_frame
        );
    }
}

/// Estimate current system load (simplified implementation)
fn estimate_system_load(_time: &Res<Time>) -> f32 {
    // TODO: Implement actual system monitoring
    // For now, return a placeholder value
    0.5 // 50% system load
}

/// Biome-aware adaptive spawning system
pub fn biome_adaptive_spawning_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    config: Res<AdvancedSpawnBudgetConfig>,
    // TODO: Add biome detection query
) {
    // TODO: Implement biome-specific adaptation
    // This will integrate with the hierarchical world generation system

    debug!(
        "Biome-aware adaptive spawning: adaptation_factor={:.2}",
        config.frame_rate_adaptation.smoothing_factor
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawn_budget::FrameRateAdaptationConfig;

    #[test]
    fn test_fps_adaptation_factor_calculation() {
        let config = FrameRateAdaptationConfig {
            target_fps: 60.0,
            low_fps_threshold: 45.0,
            high_fps_threshold: 75.0,
            max_reduction_factor: 0.3,
            max_increase_factor: 1.5,
            smoothing_factor: 0.1,
        };

        // Test low FPS scenario
        let low_fps_factor = calculate_fps_adaptation_factor(35.0, &config);
        assert!(low_fps_factor < 1.0);
        assert!(low_fps_factor >= config.max_reduction_factor);

        // Test high FPS scenario
        let high_fps_factor = calculate_fps_adaptation_factor(90.0, &config);
        assert!(high_fps_factor > 1.0);
        assert!(high_fps_factor <= config.max_increase_factor);

        // Test normal FPS scenario
        let normal_fps_factor = calculate_fps_adaptation_factor(60.0, &config);
        assert!((normal_fps_factor - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_adaptation_factor_smoothing() {
        let current = 1.0;
        let target = 0.5;
        let smoothing = 0.1;
        let delta_time = 1.0 / 60.0; // 60 FPS

        let result = lerp_adaptation_factor(current, target, smoothing, delta_time);
        assert!(result > target);
        assert!(result < current);
    }
}
