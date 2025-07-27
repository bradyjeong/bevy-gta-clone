//! Advanced Spawn Budget Plugin
//!
//! **Oracle's Integration Vision**: Unite all advanced spawn budget systems
//!
//! This plugin orchestrates the advanced spawn budget features with proper
//! system scheduling and feature flag integration.

use crate::spawn_budget::{
    adaptive::*, gpu_integration::*, performance_feedback::*, AdvancedSpawnBudgetConfig,
    AdvancedSpawnMetrics,
};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

/// Advanced Spawn Budget Plugin
pub struct AdvancedSpawnBudgetPlugin;

impl Plugin for AdvancedSpawnBudgetPlugin {
    fn build(&self, app: &mut App) {
        // Ensure frame time diagnostics are available
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        // Add resources
        app.init_resource::<AdvancedSpawnBudgetConfig>()
            .init_resource::<AdvancedSpawnMetrics>()
            .init_resource::<PerformanceWindow>();

        // Add core adaptive systems
        app.add_systems(
            Update,
            (
                // Phase 1: Performance monitoring and feedback
                performance_feedback_system,
                // Phase 2: Adaptive budget adjustment
                adaptive_spawn_budget_system,
                performance_budget_scaling_system,
                // Phase 3: GPU integration systems
                #[cfg(feature = "unstable_spawn_budget")]
                occlusion_aware_spawn_system,
                #[cfg(feature = "unstable_spawn_budget")]
                distance_culling_spawn_system,
                #[cfg(feature = "unstable_spawn_budget")]
                gpu_culling_status_system,
                // Phase 4: Advanced culling integration
                #[cfg(feature = "unstable_spawn_budget")]
                frustum_culling_spawn_system,
                #[cfg(feature = "unstable_spawn_budget")]
                hierarchical_occlusion_spawn_system,
                // Phase 5: Biome-aware adaptation (when hierarchical world is available)
                #[cfg(all(
                    feature = "unstable_spawn_budget",
                    feature = "unstable_hierarchical_world"
                ))]
                biome_adaptive_spawning_system,
                // Phase 6: Integration with hierarchical world generation
                #[cfg(all(
                    feature = "unstable_spawn_budget",
                    feature = "unstable_hierarchical_world"
                ))]
                super::integration::hierarchical_world_spawn_integration_system,
                #[cfg(feature = "unstable_spawn_budget")]
                super::integration::biome_aware_budget_adjustment_system,
                #[cfg(feature = "unstable_spawn_budget")]
                super::integration::world_streaming_coordination_system,
                #[cfg(feature = "unstable_spawn_budget")]
                super::integration::integration_performance_monitoring_system,
            )
                .chain()
                .in_set(AdvancedSpawnBudgetSystemSet),
        );

        // Schedule advanced systems in Update schedule
        // Note: System ordering will be handled by system dependencies

        // Add debug systems in development
        #[cfg(debug_assertions)]
        app.add_systems(Update, debug_spawn_budget_metrics);

        info!("Advanced Spawn Budget Plugin initialized with Oracle's discipline");
    }
}

/// System set for advanced spawn budget operations
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AdvancedSpawnBudgetSystemSet;

/// Debug system for monitoring spawn budget metrics
#[cfg(debug_assertions)]
fn debug_spawn_budget_metrics(
    metrics: Res<AdvancedSpawnMetrics>,
    config: Res<AdvancedSpawnBudgetConfig>,
    mut last_debug_time: Local<f32>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Debug output every 5 seconds
    if current_time - *last_debug_time < 5.0 {
        return;
    }
    *last_debug_time = current_time;

    debug!(
        "Advanced Spawn Budget Metrics:\n\
        - Current FPS: {:.1}\n\
        - FPS Adaptation Factor: {:.2}\n\
        - Occlusion Rejections: {}\n\
        - Distance Rejections: {}\n\
        - Performance Rejections: {}\n\
        - Avg Spawn Processing Time: {:.2}ms\n\
        - GPU Culling Active: {}",
        metrics.current_fps,
        metrics.fps_adaptation_factor,
        metrics.occlusion_rejections,
        metrics.distance_rejections,
        metrics.performance_rejections,
        metrics.avg_spawn_processing_time,
        metrics.gpu_culling_active
    );

    if config.frame_rate_adaptation.target_fps > 0.0 {
        let fps_efficiency = metrics.current_fps / config.frame_rate_adaptation.target_fps;
        if fps_efficiency < 0.8 {
            warn!(
                "Frame rate below target: {:.1} FPS (target: {:.1} FPS, efficiency: {:.1}%)",
                metrics.current_fps,
                config.frame_rate_adaptation.target_fps,
                fps_efficiency * 100.0
            );
        }
    }
}

/// System for runtime configuration updates
pub fn update_spawn_budget_config_system(
    mut config: ResMut<AdvancedSpawnBudgetConfig>,
    // TODO: Add asset loading for hot-reload capability
) {
    // TODO: Implement hot-reload from config files
    // This will allow runtime tuning of spawn budget parameters

    // For now, just validate configuration consistency
    validate_config_consistency(&mut config);
}

/// Validate configuration parameter consistency
pub fn validate_config_consistency(config: &mut AdvancedSpawnBudgetConfig) {
    // Ensure frame rate thresholds are sensible
    if config.frame_rate_adaptation.low_fps_threshold
        >= config.frame_rate_adaptation.high_fps_threshold
    {
        warn!("Invalid FPS thresholds, correcting...");
        config.frame_rate_adaptation.high_fps_threshold =
            config.frame_rate_adaptation.low_fps_threshold + 10.0;
    }

    // Ensure adaptation factors are within bounds
    config.frame_rate_adaptation.max_reduction_factor = config
        .frame_rate_adaptation
        .max_reduction_factor
        .clamp(0.1, 1.0);
    config.frame_rate_adaptation.max_increase_factor = config
        .frame_rate_adaptation
        .max_increase_factor
        .clamp(1.0, 3.0);

    // Ensure smoothing factor is reasonable
    config.frame_rate_adaptation.smoothing_factor = config
        .frame_rate_adaptation
        .smoothing_factor
        .clamp(0.01, 1.0);

    // Validate distance curves
    for curve in [
        &mut config.distance_curves.building_curve,
        &mut config.distance_curves.vehicle_curve,
        &mut config.distance_curves.npc_curve,
        &mut config.distance_curves.tree_curve,
        &mut config.distance_curves.particle_curve,
    ] {
        if curve.distances.len() != curve.probabilities.len() {
            warn!("Distance curve mismatch, using defaults");
            *curve = super::DistanceCurve {
                distances: vec![0.0, 100.0, 300.0],
                probabilities: vec![1.0, 0.5, 0.0],
            };
        }
    }
}

/// Integration helper for external systems
pub struct AdvancedSpawnBudgetIntegration;

impl AdvancedSpawnBudgetIntegration {
    /// Check if advanced spawn budget features are available
    pub fn is_available() -> bool {
        cfg!(feature = "unstable_spawn_budget")
    }

    /// Get recommended spawn limit based on current performance
    pub fn get_recommended_spawn_limit(metrics: &AdvancedSpawnMetrics, base_limit: u32) -> u32 {
        (base_limit as f32 * metrics.fps_adaptation_factor).round() as u32
    }

    /// Check if position is suitable for spawning based on distance curves
    pub fn should_spawn_at_distance(
        entity_type: amp_gameplay::spawn_budget_policy::EntityType,
        distance: f32,
        config: &AdvancedSpawnBudgetConfig,
    ) -> bool {
        use crate::spawn_budget::gpu_integration::calculate_distance_spawn_probability;

        let probability =
            calculate_distance_spawn_probability(entity_type, distance, &config.distance_curves);

        fastrand::f32() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_plugin_integration() {
        let mut app = App::new();
        app.add_plugins(AdvancedSpawnBudgetPlugin);

        // Verify resources are initialized
        assert!(app.world().contains_resource::<AdvancedSpawnBudgetConfig>());
        assert!(app.world().contains_resource::<AdvancedSpawnMetrics>());
        assert!(app.world().contains_resource::<PerformanceWindow>());
    }

    #[test]
    fn test_config_validation() {
        let mut config = AdvancedSpawnBudgetConfig::default();

        // Test invalid FPS thresholds
        config.frame_rate_adaptation.low_fps_threshold = 70.0;
        config.frame_rate_adaptation.high_fps_threshold = 60.0;

        validate_config_consistency(&mut config);

        assert!(
            config.frame_rate_adaptation.low_fps_threshold
                < config.frame_rate_adaptation.high_fps_threshold
        );
    }

    #[test]
    fn test_integration_helpers() {
        let metrics = AdvancedSpawnMetrics {
            fps_adaptation_factor: 0.8,
            ..Default::default()
        };

        let recommended_limit =
            AdvancedSpawnBudgetIntegration::get_recommended_spawn_limit(&metrics, 50);

        assert_eq!(recommended_limit, 40); // 50 * 0.8 = 40
    }
}
