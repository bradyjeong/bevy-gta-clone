//! Performance Feedback Loop for Spawn Budget Management
//!
//! **Oracle's Performance Wisdom**: Spawn budgets must respond to system reality
//!
//! This module implements real-time performance monitoring and feedback loops
//! that dynamically adjust spawn budgets based on system performance metrics.

use crate::spawn_budget::{AdvancedSpawnBudgetConfig, AdvancedSpawnMetrics};
use amp_gameplay::spawn_budget_policy::SpawnBudgetPolicy;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Performance measurement window for tracking metrics
#[derive(Resource, Debug, Default)]
pub struct PerformanceWindow {
    /// Frame time samples
    pub frame_times: Vec<f32>,
    /// CPU usage samples (estimated)
    pub cpu_usage: Vec<f32>,
    /// Memory usage samples
    pub memory_usage: Vec<f32>,
    /// GPU usage samples (estimated)
    pub gpu_usage: Vec<f32>,
    /// Sample collection timestamp
    pub last_sample_time: f32,
}

/// Performance feedback and adjustment system
pub fn performance_feedback_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    mut metrics: ResMut<AdvancedSpawnMetrics>,
    mut perf_window: ResMut<PerformanceWindow>,
    config: Res<AdvancedSpawnBudgetConfig>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
) {
    if !config.performance_feedback.enabled {
        return;
    }

    let current_time = time.elapsed_secs();

    // Collect performance samples
    collect_performance_samples(
        &mut perf_window,
        &diagnostics,
        current_time,
        config.performance_feedback.measurement_window,
    );

    // Analyze performance trends
    let performance_assessment = analyze_performance_trends(&perf_window, &config);

    // Apply performance-based adjustments
    apply_performance_adjustments(&mut policy, &mut metrics, &performance_assessment, &config);
}

/// Collect real-time performance samples
fn collect_performance_samples(
    window: &mut PerformanceWindow,
    diagnostics: &DiagnosticsStore,
    current_time: f32,
    measurement_window: f32,
) {
    // Sample every 100ms
    if current_time - window.last_sample_time < 0.1 {
        return;
    }

    window.last_sample_time = current_time;

    // Collect frame time
    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diag| diag.smoothed())
    {
        window.frame_times.push(frame_time as f32);
    }

    // Estimate CPU usage (simplified)
    let estimated_cpu = estimate_cpu_usage(diagnostics);
    window.cpu_usage.push(estimated_cpu);

    // Estimate memory usage
    let estimated_memory = estimate_memory_usage();
    window.memory_usage.push(estimated_memory);

    // Estimate GPU usage (simplified)
    let estimated_gpu = estimate_gpu_usage(diagnostics);
    window.gpu_usage.push(estimated_gpu);

    // Maintain window size
    let max_samples = (measurement_window * 10.0) as usize; // 10 samples per second

    if window.frame_times.len() > max_samples {
        window.frame_times.remove(0);
    }
    if window.cpu_usage.len() > max_samples {
        window.cpu_usage.remove(0);
    }
    if window.memory_usage.len() > max_samples {
        window.memory_usage.remove(0);
    }
    if window.gpu_usage.len() > max_samples {
        window.gpu_usage.remove(0);
    }
}

/// Performance assessment result
#[derive(Debug, Clone)]
pub struct PerformanceAssessment {
    pub avg_frame_time: f32,
    pub frame_time_variance: f32,
    pub cpu_load: f32,
    pub memory_pressure: f32,
    pub gpu_load: f32,
    pub performance_score: f32, // 0.0 (bad) to 1.0 (excellent)
    pub recommended_action: PerformanceAction,
}

/// Recommended performance action
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceAction {
    ReduceSpawns,
    MaintainSpawns,
    IncreaseSpawns,
    EmergencyThrottle,
}

/// Analyze performance trends and generate assessment
pub fn analyze_performance_trends(
    window: &PerformanceWindow,
    config: &AdvancedSpawnBudgetConfig,
) -> PerformanceAssessment {
    if window.frame_times.is_empty() {
        return PerformanceAssessment::default();
    }

    // Calculate average frame time
    let avg_frame_time = window.frame_times.iter().sum::<f32>() / window.frame_times.len() as f32;

    // Calculate frame time variance (stability metric)
    let variance = window
        .frame_times
        .iter()
        .map(|&x| (x - avg_frame_time).powi(2))
        .sum::<f32>()
        / window.frame_times.len() as f32;

    // Calculate resource usage averages
    let cpu_load = if window.cpu_usage.is_empty() {
        0.5
    } else {
        window.cpu_usage.iter().sum::<f32>() / window.cpu_usage.len() as f32
    };

    let memory_pressure = if window.memory_usage.is_empty() {
        0.5
    } else {
        window.memory_usage.iter().sum::<f32>() / window.memory_usage.len() as f32
    };

    let gpu_load = if window.gpu_usage.is_empty() {
        0.5
    } else {
        window.gpu_usage.iter().sum::<f32>() / window.gpu_usage.len() as f32
    };

    // Calculate overall performance score
    let frame_rate_score = calculate_frame_rate_score(avg_frame_time);
    let stability_score = calculate_stability_score(variance);
    let resource_score = calculate_resource_score(cpu_load, memory_pressure, gpu_load);

    let performance_score = (frame_rate_score + stability_score + resource_score) / 3.0;

    // Determine recommended action
    let recommended_action = determine_performance_action(
        performance_score,
        cpu_load,
        memory_pressure,
        gpu_load,
        config,
    );

    PerformanceAssessment {
        avg_frame_time,
        frame_time_variance: variance,
        cpu_load,
        memory_pressure,
        gpu_load,
        performance_score,
        recommended_action,
    }
}

/// Calculate frame rate performance score (0.0-1.0)
fn calculate_frame_rate_score(avg_frame_time: f32) -> f32 {
    let target_frame_time = 1.0 / 60.0; // 16.67ms for 60 FPS
    let max_acceptable_frame_time = 1.0 / 30.0; // 33.33ms for 30 FPS

    if avg_frame_time <= target_frame_time {
        1.0 // Excellent performance
    } else if avg_frame_time <= max_acceptable_frame_time {
        // Linear interpolation between excellent and acceptable
        1.0 - (avg_frame_time - target_frame_time) / (max_acceptable_frame_time - target_frame_time)
    } else {
        // Poor performance
        0.0
    }
}

/// Calculate frame time stability score (0.0-1.0)
fn calculate_stability_score(variance: f32) -> f32 {
    let max_acceptable_variance = 0.01; // 10ms variance threshold

    if variance <= max_acceptable_variance {
        1.0 - (variance / max_acceptable_variance)
    } else {
        0.0
    }
}

/// Calculate resource usage score (0.0-1.0)
fn calculate_resource_score(cpu_load: f32, memory_pressure: f32, gpu_load: f32) -> f32 {
    let cpu_score = (1.0 - cpu_load).max(0.0);
    let memory_score = (1.0 - memory_pressure).max(0.0);
    let gpu_score = (1.0 - gpu_load).max(0.0);

    (cpu_score + memory_score + gpu_score) / 3.0
}

/// Determine recommended performance action
pub fn determine_performance_action(
    performance_score: f32,
    cpu_load: f32,
    memory_pressure: f32,
    gpu_load: f32,
    config: &AdvancedSpawnBudgetConfig,
) -> PerformanceAction {
    // Emergency throttle conditions
    if cpu_load > 0.95 || memory_pressure > 0.95 || gpu_load > 0.95 {
        return PerformanceAction::EmergencyThrottle;
    }

    // Check individual thresholds
    if cpu_load > config.performance_feedback.cpu_threshold
        || memory_pressure > config.performance_feedback.memory_threshold
        || gpu_load > config.performance_feedback.gpu_threshold
    {
        return PerformanceAction::ReduceSpawns;
    }

    // Use overall performance score
    if performance_score < 0.4 {
        PerformanceAction::ReduceSpawns
    } else if performance_score > 0.8 {
        PerformanceAction::IncreaseSpawns
    } else {
        PerformanceAction::MaintainSpawns
    }
}

/// Apply performance-based adjustments to spawn policy
fn apply_performance_adjustments(
    policy: &mut SpawnBudgetPolicy,
    metrics: &mut AdvancedSpawnMetrics,
    assessment: &PerformanceAssessment,
    config: &AdvancedSpawnBudgetConfig,
) {
    match assessment.recommended_action {
        PerformanceAction::EmergencyThrottle => {
            // Severe performance degradation: emergency throttle
            let current_limit = policy.frame_limits.max_spawns_per_frame;
            let emergency_limit = (current_limit / 4).max(1); // 25% of current, minimum 1

            policy.frame_limits.max_spawns_per_frame = emergency_limit;
            policy.frame_limits.time_budget_ms *= 0.5; // Halve time budget

            metrics.performance_rejections += 10; // Severe penalty

            warn!(
                "Emergency spawn throttle: performance_score={:.2}, new_limit={}",
                assessment.performance_score, emergency_limit
            );
        }

        PerformanceAction::ReduceSpawns => {
            // Moderate performance issues: reduce spawns
            let current_limit = policy.frame_limits.max_spawns_per_frame;
            let reduced_limit = ((current_limit as f32 * 0.8).round() as u32).max(5);

            policy.frame_limits.max_spawns_per_frame = reduced_limit;
            policy.frame_limits.time_budget_ms *= 0.9; // Reduce time budget

            metrics.performance_rejections += 1;

            info!(
                "Reducing spawn budget: performance_score={:.2}, new_limit={}",
                assessment.performance_score, reduced_limit
            );
        }

        PerformanceAction::IncreaseSpawns => {
            // Good performance: can increase spawns
            let current_limit = policy.frame_limits.max_spawns_per_frame;
            let increased_limit = ((current_limit as f32 * 1.1).round() as u32).min(100);

            policy.frame_limits.max_spawns_per_frame = increased_limit;
            policy.frame_limits.time_budget_ms *= 1.05; // Slightly increase time budget

            debug!(
                "Increasing spawn budget: performance_score={:.2}, new_limit={}",
                assessment.performance_score, increased_limit
            );
        }

        PerformanceAction::MaintainSpawns => {
            // Stable performance: no changes needed
            debug!(
                "Maintaining spawn budget: performance_score={:.2}",
                assessment.performance_score
            );
        }
    }
}

/// Estimate CPU usage from available diagnostics
fn estimate_cpu_usage(diagnostics: &DiagnosticsStore) -> f32 {
    // Simplified CPU estimation based on frame time consistency
    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diag| diag.smoothed())
    {
        let target_frame_time = 1.0 / 60.0; // 16.67ms
        let cpu_estimate = (frame_time as f32 / target_frame_time).clamp(0.0, 1.0);
        cpu_estimate
    } else {
        0.5 // Default estimate
    }
}

/// Estimate memory usage (placeholder implementation)
fn estimate_memory_usage() -> f32 {
    // TODO: Implement actual memory monitoring
    // For now, return a placeholder value
    0.6 // 60% memory usage estimate
}

/// Estimate GPU usage from rendering diagnostics
fn estimate_gpu_usage(diagnostics: &DiagnosticsStore) -> f32 {
    // TODO: Implement GPU usage estimation
    // This could be based on render pipeline diagnostics
    0.4 // 40% GPU usage estimate
}

impl Default for PerformanceAssessment {
    fn default() -> Self {
        Self {
            avg_frame_time: 1.0 / 60.0,
            frame_time_variance: 0.0,
            cpu_load: 0.5,
            memory_pressure: 0.5,
            gpu_load: 0.5,
            performance_score: 0.7,
            recommended_action: PerformanceAction::MaintainSpawns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_rate_score_calculation() {
        // Excellent performance (60+ FPS)
        let excellent_score = calculate_frame_rate_score(1.0 / 60.0);
        assert!((excellent_score - 1.0).abs() < 0.01);

        // Good performance (45 FPS)
        let good_score = calculate_frame_rate_score(1.0 / 45.0);
        assert!(good_score > 0.5 && good_score < 1.0);

        // Poor performance (20 FPS)
        let poor_score = calculate_frame_rate_score(1.0 / 20.0);
        assert!((poor_score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_performance_action_determination() {
        let config = super::super::AdvancedSpawnBudgetConfig::default();

        // Test emergency throttle
        let emergency_action = determine_performance_action(0.2, 0.98, 0.5, 0.5, &config);
        assert_eq!(emergency_action, PerformanceAction::EmergencyThrottle);

        // Test reduce spawns
        let reduce_action = determine_performance_action(0.3, 0.85, 0.5, 0.5, &config);
        assert_eq!(reduce_action, PerformanceAction::ReduceSpawns);

        // Test increase spawns
        let increase_action = determine_performance_action(0.9, 0.3, 0.3, 0.3, &config);
        assert_eq!(increase_action, PerformanceAction::IncreaseSpawns);

        // Test maintain spawns
        let maintain_action = determine_performance_action(0.6, 0.5, 0.5, 0.5, &config);
        assert_eq!(maintain_action, PerformanceAction::MaintainSpawns);
    }

    #[test]
    fn test_stability_score_calculation() {
        // Very stable
        let stable_score = calculate_stability_score(0.001);
        assert!(stable_score > 0.9);

        // Unstable
        let unstable_score = calculate_stability_score(0.02);
        assert!((unstable_score - 0.0).abs() < 0.01);
    }
}
