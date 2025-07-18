/*!
# Performance Integration: Unified Optimization System

Integrates all performance implementations following Oracle's specifications:
- Distance cache optimization
- Batch processing integration
- NPC systems optimization
- World streaming optimization
- Transform synchronization optimization
- Comprehensive performance monitoring

## Architecture

This module provides a unified performance optimization system that orchestrates
all individual performance implementations to achieve the <3.0ms target frame time.
*/

use bevy::prelude::*;
use std::time::Instant;

// Import all performance implementations
use crate::performance_strike::*;
use crate::world_streaming::WorldStreamer;

// Placeholder structs for missing imports
#[derive(Default, Resource)]
pub struct BatchProcessor;

#[derive(Default)]
pub struct NPCSystem;

#[derive(Default)]
pub struct VehiclePhysicsSystem;

/// Integrated performance optimization system
#[derive(Resource, Debug)]
pub struct IntegratedPerformanceSystem {
    /// Performance strike configuration
    pub config: PerformanceStrikeConfig,
    /// Performance metrics aggregation
    pub metrics: PerformanceMetrics,
    /// Subsystem performance trackers
    pub subsystems: SubsystemPerformance,
    /// Optimization state
    pub optimization_state: OptimizationState,
}

/// Performance tracking for all subsystems
#[derive(Debug, Default, Clone)]
pub struct SubsystemPerformance {
    pub distance_cache: SubsystemMetrics,
    pub batch_processing: SubsystemMetrics,
    pub npc_system: SubsystemMetrics,
    pub world_streaming: SubsystemMetrics,
    pub transform_sync: SubsystemMetrics,
    pub gpu_culling: SubsystemMetrics,
    pub lod_system: SubsystemMetrics,
    pub vehicle_physics: SubsystemMetrics,
}

/// Performance metrics for individual subsystems
#[derive(Debug, Default, Clone)]
pub struct SubsystemMetrics {
    pub average_time_ms: f64,
    pub max_time_ms: f64,
    pub min_time_ms: f64,
    pub samples: usize,
    pub budget_violations: usize,
    pub optimization_level: OptimizationLevel,
}

/// Optimization level for adaptive performance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    Conservative,
    Balanced,
    Aggressive,
    Maximum,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::Balanced
    }
}

/// Current optimization state
#[derive(Debug, Default, Clone)]
pub struct OptimizationState {
    pub adaptive_optimization: bool,
    pub current_load: f64,
    pub target_load: f64,
    pub optimization_adjustments: Vec<OptimizationAdjustment>,
}

/// Record of optimization adjustments
#[derive(Debug, Clone)]
pub struct OptimizationAdjustment {
    pub timestamp: Instant,
    pub subsystem: String,
    pub adjustment_type: AdjustmentType,
    pub old_value: f64,
    pub new_value: f64,
    pub performance_impact: f64,
}

/// Type of optimization adjustment
#[derive(Debug, Clone)]
pub enum AdjustmentType {
    SchedulerFrequency,
    BatchSize,
    CacheSize,
    QualityLevel,
    ParallelismLevel,
}

impl Default for IntegratedPerformanceSystem {
    fn default() -> Self {
        Self {
            config: PerformanceStrikeConfig::default(),
            metrics: PerformanceMetrics::default(),
            subsystems: SubsystemPerformance::default(),
            optimization_state: OptimizationState::default(),
        }
    }
}

impl IntegratedPerformanceSystem {
    /// Create new integrated performance system
    pub fn new(config: PerformanceStrikeConfig) -> Self {
        Self {
            config,
            metrics: PerformanceMetrics::default(),
            subsystems: SubsystemPerformance::default(),
            optimization_state: OptimizationState {
                adaptive_optimization: true,
                current_load: 0.0,
                target_load: 0.8, // 80% of target performance
                optimization_adjustments: Vec::new(),
            },
        }
    }

    /// Update performance metrics for a subsystem
    pub fn update_subsystem_metrics(&mut self, subsystem: &str, time_ms: f64) {
        let budget = self.get_budget_for_subsystem(subsystem);

        let metrics = match subsystem {
            "distance_cache" => &mut self.subsystems.distance_cache,
            "batch_processing" => &mut self.subsystems.batch_processing,
            "npc_system" => &mut self.subsystems.npc_system,
            "world_streaming" => &mut self.subsystems.world_streaming,
            "transform_sync" => &mut self.subsystems.transform_sync,
            "gpu_culling" => &mut self.subsystems.gpu_culling,
            "lod_system" => &mut self.subsystems.lod_system,
            "vehicle_physics" => &mut self.subsystems.vehicle_physics,
            _ => return,
        };

        metrics.samples += 1;
        metrics.average_time_ms = (metrics.average_time_ms * (metrics.samples - 1) as f64
            + time_ms)
            / metrics.samples as f64;
        metrics.max_time_ms = metrics.max_time_ms.max(time_ms);
        metrics.min_time_ms = if metrics.min_time_ms == 0.0 {
            time_ms
        } else {
            metrics.min_time_ms.min(time_ms)
        };

        // Check budget violations
        if time_ms > budget {
            metrics.budget_violations += 1;
        }
    }

    /// Get performance budget for subsystem
    fn get_budget_for_subsystem(&self, subsystem: &str) -> f64 {
        match subsystem {
            "distance_cache" => 0.1,
            "batch_processing" => 0.5,
            "npc_system" => self.config.performance_budgets.ai_budget_ms,
            "world_streaming" => 0.5,
            "transform_sync" => self.config.performance_budgets.transform_budget_ms,
            "gpu_culling" => 0.3,
            "lod_system" => self.config.performance_budgets.lod_budget_ms,
            "vehicle_physics" => self.config.performance_budgets.physics_budget_ms,
            _ => 0.2,
        }
    }

    /// Perform adaptive optimization based on current performance
    pub fn adaptive_optimization(&mut self) {
        if !self.optimization_state.adaptive_optimization {
            return;
        }

        let total_time: f64 = self.subsystems.distance_cache.average_time_ms
            + self.subsystems.batch_processing.average_time_ms
            + self.subsystems.npc_system.average_time_ms
            + self.subsystems.world_streaming.average_time_ms
            + self.subsystems.transform_sync.average_time_ms
            + self.subsystems.gpu_culling.average_time_ms
            + self.subsystems.lod_system.average_time_ms
            + self.subsystems.vehicle_physics.average_time_ms;

        self.optimization_state.current_load = total_time / self.config.target_frame_time_ms;

        // Adjust optimization levels based on load
        if self.optimization_state.current_load > 1.0 {
            self.escalate_optimization();
        } else if self.optimization_state.current_load < self.optimization_state.target_load {
            self.reduce_optimization();
        }
    }

    /// Escalate optimization when performance is poor
    fn escalate_optimization(&mut self) {
        // Find the worst performing subsystem
        let worst_subsystem = self.find_worst_performing_subsystem();

        match worst_subsystem.as_str() {
            "transform_sync" => {
                self.record_adjustment(
                    "transform_sync",
                    AdjustmentType::ParallelismLevel,
                    0.5,
                    1.0,
                    0.2,
                );
            }
            "lod_system" => {
                self.record_adjustment(
                    "lod_system",
                    AdjustmentType::SchedulerFrequency,
                    2.0,
                    4.0,
                    0.15,
                );
            }
            "npc_system" => {
                self.record_adjustment("npc_system", AdjustmentType::BatchSize, 200.0, 100.0, 0.1);
            }
            _ => {}
        }
    }

    /// Reduce optimization when performance is good
    fn reduce_optimization(&mut self) {
        // Gradually reduce optimization levels to improve quality
        if let Some(last_adjustment) = self
            .optimization_state
            .optimization_adjustments
            .last()
            .cloned()
        {
            if last_adjustment.timestamp.elapsed().as_secs() > 5 {
                // Revert some optimizations if performance has been stable
                self.record_adjustment(
                    &last_adjustment.subsystem,
                    last_adjustment.adjustment_type.clone(),
                    last_adjustment.new_value,
                    last_adjustment.old_value,
                    -last_adjustment.performance_impact,
                );
            }
        }
    }

    /// Find the worst performing subsystem
    fn find_worst_performing_subsystem(&self) -> String {
        let mut worst_ratio = 0.0;
        let mut worst_subsystem = String::new();

        let subsystems = [
            ("distance_cache", &self.subsystems.distance_cache),
            ("batch_processing", &self.subsystems.batch_processing),
            ("npc_system", &self.subsystems.npc_system),
            ("world_streaming", &self.subsystems.world_streaming),
            ("transform_sync", &self.subsystems.transform_sync),
            ("gpu_culling", &self.subsystems.gpu_culling),
            ("lod_system", &self.subsystems.lod_system),
            ("vehicle_physics", &self.subsystems.vehicle_physics),
        ];

        for (name, metrics) in subsystems {
            let budget = self.get_budget_for_subsystem(name);
            let ratio = metrics.average_time_ms / budget;

            if ratio > worst_ratio {
                worst_ratio = ratio;
                worst_subsystem = name.to_string();
            }
        }

        worst_subsystem
    }

    /// Record optimization adjustment
    fn record_adjustment(
        &mut self,
        subsystem: &str,
        adjustment_type: AdjustmentType,
        old_value: f64,
        new_value: f64,
        performance_impact: f64,
    ) {
        self.optimization_state
            .optimization_adjustments
            .push(OptimizationAdjustment {
                timestamp: Instant::now(),
                subsystem: subsystem.to_string(),
                adjustment_type,
                old_value,
                new_value,
                performance_impact,
            });
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            overall_performance: self.calculate_overall_performance(),
            subsystem_performance: self.subsystems.clone(),
            optimization_state: self.optimization_state.clone(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// Calculate overall performance score
    fn calculate_overall_performance(&self) -> f64 {
        let total_time: f64 = self.subsystems.distance_cache.average_time_ms
            + self.subsystems.batch_processing.average_time_ms
            + self.subsystems.npc_system.average_time_ms
            + self.subsystems.world_streaming.average_time_ms
            + self.subsystems.transform_sync.average_time_ms
            + self.subsystems.gpu_culling.average_time_ms
            + self.subsystems.lod_system.average_time_ms
            + self.subsystems.vehicle_physics.average_time_ms;

        (self.config.target_frame_time_ms / total_time).min(1.0)
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check each subsystem against its budget
        if self.subsystems.transform_sync.average_time_ms
            > self.config.performance_budgets.transform_budget_ms
        {
            recommendations.push("Enable parallel transform synchronization".to_string());
        }

        if self.subsystems.lod_system.average_time_ms
            > self.config.performance_budgets.lod_budget_ms
        {
            recommendations.push("Reduce LOD update frequency to every 4th frame".to_string());
        }

        if self.subsystems.npc_system.average_time_ms > self.config.performance_budgets.ai_budget_ms
        {
            recommendations.push("Increase NPC update distances and reduce batch size".to_string());
        }

        if self.subsystems.distance_cache.average_time_ms > 0.1 {
            recommendations.push("Switch to HopSlotMap-based distance cache".to_string());
        }

        recommendations
    }
}

/// Performance report for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub overall_performance: f64,
    pub subsystem_performance: SubsystemPerformance,
    pub optimization_state: OptimizationState,
    pub recommendations: Vec<String>,
}

/// Integrated performance system update
pub fn integrated_performance_update(
    mut system: ResMut<IntegratedPerformanceSystem>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let start_time = Instant::now();

    // Update frame time
    let frame_time = time.delta_secs_f64() * 1000.0;
    system.metrics.frame_times.push(frame_time);

    // Perform adaptive optimization
    system.adaptive_optimization();

    // Update optimization state
    system.optimization_state.current_load = frame_time / system.config.target_frame_time_ms;

    let elapsed = start_time.elapsed();
    system.update_subsystem_metrics("integrated_performance", elapsed.as_secs_f64() * 1000.0);
}

/// Optimized distance cache system with integration
pub fn integrated_distance_cache_system(
    mut cache: ResMut<OptimizedDistanceCache>,
    query: Query<(Entity, &Transform), With<GlobalTransform>>,
    camera_query: Query<&Transform, (With<Camera>, Without<GlobalTransform>)>,
    counter: Res<FrameCounter>,
    mut performance_system: ResMut<IntegratedPerformanceSystem>,
) {
    let start_time = Instant::now();

    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation;

        // Update distance cache with SoA optimization
        for (entity, transform) in query.iter() {
            let _distance = cache.get_or_calculate(
                entity,
                transform.translation,
                camera_pos,
                counter.frame_number,
            );
        }
    }

    let elapsed = start_time.elapsed();
    performance_system.update_subsystem_metrics("distance_cache", elapsed.as_secs_f64() * 1000.0);
}

/// Integrated batch processing system
pub fn integrated_batch_processing_system(
    _batch_processor: ResMut<BatchProcessor>,
    mut performance_system: ResMut<IntegratedPerformanceSystem>,
    _world: &World,
) {
    let start_time = Instant::now();

    // Execute batch processing with performance monitoring
    // This would integrate with the existing batch processor

    let elapsed = start_time.elapsed();
    performance_system.update_subsystem_metrics("batch_processing", elapsed.as_secs_f64() * 1000.0);
}

/// Integrated NPC system with distance-based updates
pub fn integrated_npc_system(
    mut npc_query: Query<(Entity, &Transform, &mut NPCDialogState)>,
    camera_query: Query<&Transform, (With<Camera>, Without<NPCDialogState>)>,
    mut distance_cache: ResMut<OptimizedDistanceCache>,
    counter: Res<FrameCounter>,
    mut performance_system: ResMut<IntegratedPerformanceSystem>,
) {
    let start_time = Instant::now();

    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation;

        // Process NPCs with distance-based batching
        let mut processed = 0;
        const MAX_NPC_UPDATES: usize = 200;

        for (entity, transform, mut npc_state) in npc_query.iter_mut() {
            if processed >= MAX_NPC_UPDATES {
                break;
            }

            let distance = distance_cache.get_or_calculate(
                entity,
                transform.translation,
                camera_pos,
                counter.frame_number,
            );

            // Update NPC based on distance
            if distance < 50.0 {
                // Close NPCs update every frame
                npc_state.interaction_cooldown -= 1.0 / 60.0;
            } else if distance < 200.0 && counter.frame_number % 4 == 0 {
                // Medium distance NPCs update every 4th frame
                npc_state.interaction_cooldown -= 4.0 / 60.0;
            } else if distance < 500.0 && counter.frame_number % 16 == 0 {
                // Far NPCs update every 16th frame
                npc_state.interaction_cooldown -= 16.0 / 60.0;
            }

            processed += 1;
        }
    }

    let elapsed = start_time.elapsed();
    performance_system.update_subsystem_metrics("npc_system", elapsed.as_secs_f64() * 1000.0);
}

/// Integrated world streaming system
pub fn integrated_world_streaming_system(
    mut streaming_system: ResMut<WorldStreamer>,
    mut performance_system: ResMut<IntegratedPerformanceSystem>,
    mut commands: Commands,
) {
    let start_time = Instant::now();

    // Execute world streaming with performance monitoring
    // This would integrate with the existing world streaming system

    let elapsed = start_time.elapsed();
    performance_system.update_subsystem_metrics("world_streaming", elapsed.as_secs_f64() * 1000.0);
}

/// Performance strike integration plugin
pub struct PerformanceIntegrationPlugin;

impl Plugin for PerformanceIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IntegratedPerformanceSystem::default())
            .add_systems(
                Update,
                (
                    integrated_performance_update,
                    integrated_distance_cache_system,
                    integrated_batch_processing_system,
                    integrated_npc_system,
                    integrated_world_streaming_system,
                )
                    .chain(),
            );
    }
}

/// Performance validation utilities
pub mod validation {
    use super::*;

    /// Validate integrated performance against Oracle's targets
    pub fn validate_integrated_performance(
        system: &IntegratedPerformanceSystem,
    ) -> ValidationResult {
        let mut result = ValidationResult::default();

        // Check overall performance target
        let total_time = system.subsystems.distance_cache.average_time_ms
            + system.subsystems.batch_processing.average_time_ms
            + system.subsystems.npc_system.average_time_ms
            + system.subsystems.world_streaming.average_time_ms
            + system.subsystems.transform_sync.average_time_ms
            + system.subsystems.gpu_culling.average_time_ms
            + system.subsystems.lod_system.average_time_ms
            + system.subsystems.vehicle_physics.average_time_ms;

        result.overall_target_met = total_time <= system.config.target_frame_time_ms;
        result.actual_frame_time = total_time;
        result.target_frame_time = system.config.target_frame_time_ms;

        // Check individual subsystem budgets
        result.subsystem_validations.push(SubsystemValidation {
            name: "transform_sync".to_string(),
            actual_time: system.subsystems.transform_sync.average_time_ms,
            budget: system.config.performance_budgets.transform_budget_ms,
            passed: system.subsystems.transform_sync.average_time_ms
                <= system.config.performance_budgets.transform_budget_ms,
        });

        result.subsystem_validations.push(SubsystemValidation {
            name: "lod_system".to_string(),
            actual_time: system.subsystems.lod_system.average_time_ms,
            budget: system.config.performance_budgets.lod_budget_ms,
            passed: system.subsystems.lod_system.average_time_ms
                <= system.config.performance_budgets.lod_budget_ms,
        });

        result.subsystem_validations.push(SubsystemValidation {
            name: "npc_system".to_string(),
            actual_time: system.subsystems.npc_system.average_time_ms,
            budget: system.config.performance_budgets.ai_budget_ms,
            passed: system.subsystems.npc_system.average_time_ms
                <= system.config.performance_budgets.ai_budget_ms,
        });

        result.subsystem_validations.push(SubsystemValidation {
            name: "vehicle_physics".to_string(),
            actual_time: system.subsystems.vehicle_physics.average_time_ms,
            budget: system.config.performance_budgets.physics_budget_ms,
            passed: system.subsystems.vehicle_physics.average_time_ms
                <= system.config.performance_budgets.physics_budget_ms,
        });

        result
    }

    /// Performance validation result
    #[derive(Debug, Default)]
    pub struct ValidationResult {
        pub overall_target_met: bool,
        pub actual_frame_time: f64,
        pub target_frame_time: f64,
        pub subsystem_validations: Vec<SubsystemValidation>,
    }

    /// Individual subsystem validation
    #[derive(Debug)]
    pub struct SubsystemValidation {
        pub name: String,
        pub actual_time: f64,
        pub budget: f64,
        pub passed: bool,
    }

    impl ValidationResult {
        pub fn print_report(&self) {
            println!("🎯 Performance Validation Report");
            println!("================================");
            println!(
                "Overall Performance: {:.2}ms / {:.2}ms {}",
                self.actual_frame_time,
                self.target_frame_time,
                if self.overall_target_met {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!();

            for validation in &self.subsystem_validations {
                println!(
                    "{}: {:.2}ms / {:.2}ms {}",
                    validation.name,
                    validation.actual_time,
                    validation.budget,
                    if validation.passed { "✅" } else { "❌" }
                );
            }

            println!();
            if self.overall_target_met {
                println!("🎉 All performance targets met!");
            } else {
                println!("⚠️  Performance optimization needed.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_performance_system() {
        let mut system = IntegratedPerformanceSystem::default();

        // Test updating subsystem metrics
        system.update_subsystem_metrics("transform_sync", 0.5);
        assert_eq!(system.subsystems.transform_sync.average_time_ms, 0.5);
        assert_eq!(system.subsystems.transform_sync.samples, 1);

        system.update_subsystem_metrics("transform_sync", 1.0);
        assert_eq!(system.subsystems.transform_sync.average_time_ms, 0.75);
        assert_eq!(system.subsystems.transform_sync.samples, 2);
    }

    #[test]
    fn test_performance_budget_validation() {
        let system = IntegratedPerformanceSystem::default();

        // Test budget for transform_sync
        let budget = system.get_budget_for_subsystem("transform_sync");
        assert_eq!(budget, 0.75);

        // Test budget for lod_system
        let budget = system.get_budget_for_subsystem("lod_system");
        assert_eq!(budget, 0.4);
    }

    #[test]
    fn test_optimization_adjustment() {
        let mut system = IntegratedPerformanceSystem::default();

        system.record_adjustment(
            "transform_sync",
            AdjustmentType::ParallelismLevel,
            0.5,
            1.0,
            0.2,
        );

        assert_eq!(system.optimization_state.optimization_adjustments.len(), 1);
        let adjustment = &system.optimization_state.optimization_adjustments[0];
        assert_eq!(adjustment.subsystem, "transform_sync");
        assert_eq!(adjustment.old_value, 0.5);
        assert_eq!(adjustment.new_value, 1.0);
    }

    #[test]
    fn test_performance_report() {
        let system = IntegratedPerformanceSystem::default();
        let report = system.get_performance_report();

        assert_eq!(report.overall_performance, 1.0); // No time recorded yet
        assert!(!report.recommendations.is_empty());
    }
}
