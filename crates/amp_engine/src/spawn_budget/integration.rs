//! Integration with Hierarchical World Generation System
//!
//! **Oracle's Integration Wisdom**: Advanced spawn budget must harmonize with world generation
//!
//! This module provides integration points between the advanced spawn budget system
//! and the hierarchical world generation system already ported.

use crate::spawn_budget::{AdvancedSpawnBudgetConfig, AdvancedSpawnMetrics};
use amp_gameplay::spawn_budget_policy::{BiomeType, EntityType, SpawnBudgetPolicy};
use bevy::prelude::*;

/// Integration system for hierarchical world generation
#[cfg(feature = "unstable_hierarchical_world")]
pub fn hierarchical_world_spawn_integration_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    mut metrics: ResMut<AdvancedSpawnMetrics>,
    config: Res<AdvancedSpawnBudgetConfig>,
    // TODO: Add hierarchical world query when available
) {
    // This system integrates advanced spawn budget with hierarchical world generation
    // It provides biome-aware adaptive spawning based on the hierarchical world structure

    debug!("Hierarchical world spawn integration running");

    // TODO: Implement integration with hierarchical world generation
    // This will:
    // 1. Query hierarchical world chunks and their biome types
    // 2. Apply biome-specific spawn budget adjustments
    // 3. Consider world generation priorities in spawn decisions
    // 4. Coordinate with world streaming for optimal performance
}

/// Biome-aware spawn budget adjustment system
pub fn biome_aware_budget_adjustment_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    config: Res<AdvancedSpawnBudgetConfig>,
    // TODO: Add biome detection components
) {
    // Apply biome-specific budget multipliers based on current environment
    // This system adjusts spawn budgets dynamically based on detected biome types

    debug!("Biome-aware budget adjustment running");

    // TODO: Implement biome-specific adjustments
    // Example logic:
    // - Urban biomes: Increase building budget, reduce tree budget
    // - Rural biomes: Increase tree budget, reduce NPC budget
    // - Industrial biomes: Increase particle budget for effects
}

/// World streaming coordination system
pub fn world_streaming_coordination_system(
    mut policy: ResMut<SpawnBudgetPolicy>,
    metrics: Res<AdvancedSpawnMetrics>,
    config: Res<AdvancedSpawnBudgetConfig>,
    // TODO: Add world streaming query
) {
    // Coordinate spawn budget with world streaming system
    // Ensures spawn decisions align with streaming priorities

    if metrics.current_fps < config.frame_rate_adaptation.low_fps_threshold {
        // Low FPS: prioritize essential spawns only
        apply_essential_spawn_priority(&mut policy);
    }

    debug!("World streaming coordination running");
}

/// Apply essential spawn priority during low performance
fn apply_essential_spawn_priority(policy: &mut SpawnBudgetPolicy) {
    // Reduce non-essential spawns during performance constraints
    // Priority order: Buildings > Vehicles > Trees > NPCs > Particles

    // This is a simplified implementation
    // In a full implementation, this would modify the spawn queue priorities
    debug!("Applying essential spawn priority due to low performance");
}

/// Integration helper functions
pub struct HierarchicalWorldIntegration;

impl HierarchicalWorldIntegration {
    /// Get biome-adjusted spawn limit for entity type
    pub fn get_biome_spawn_limit(
        entity_type: EntityType,
        biome: BiomeType,
        base_limit: u32,
        performance_factor: f32,
    ) -> u32 {
        let biome_multiplier = match (entity_type, biome) {
            // Urban biome adjustments
            (EntityType::Building, BiomeType::Urban) => 1.5,
            (EntityType::Vehicle, BiomeType::Urban) => 1.3,
            (EntityType::Npc, BiomeType::Urban) => 2.0,
            (EntityType::Tree, BiomeType::Urban) => 0.5,
            (EntityType::Particle, BiomeType::Urban) => 1.2,

            // Suburban biome adjustments
            (EntityType::Building, BiomeType::Suburban) => 0.8,
            (EntityType::Vehicle, BiomeType::Suburban) => 1.1,
            (EntityType::Npc, BiomeType::Suburban) => 1.2,
            (EntityType::Tree, BiomeType::Suburban) => 1.8,
            (EntityType::Particle, BiomeType::Suburban) => 0.7,

            // Rural biome adjustments
            (EntityType::Building, BiomeType::Rural) => 0.3,
            (EntityType::Vehicle, BiomeType::Rural) => 0.6,
            (EntityType::Npc, BiomeType::Rural) => 0.4,
            (EntityType::Tree, BiomeType::Rural) => 2.5,
            (EntityType::Particle, BiomeType::Rural) => 0.4,

            // Industrial biome adjustments
            (EntityType::Building, BiomeType::Industrial) => 1.2,
            (EntityType::Vehicle, BiomeType::Industrial) => 1.8,
            (EntityType::Npc, BiomeType::Industrial) => 0.6,
            (EntityType::Tree, BiomeType::Industrial) => 0.2,
            (EntityType::Particle, BiomeType::Industrial) => 3.0,
        };

        let adjusted_limit =
            (base_limit as f32 * biome_multiplier * performance_factor).round() as u32;
        adjusted_limit.max(1) // Ensure at least 1 spawn is allowed
    }

    /// Check if spawn should be deferred based on world generation state
    pub fn should_defer_spawn(
        entity_type: EntityType,
        position: Vec3,
        // TODO: Add world generation state parameters
    ) -> bool {
        // Check if the world chunk at this position is currently being generated
        // If so, defer spawn to avoid conflicts

        // Simplified implementation
        let distance_from_origin = position.length();

        // Defer spawns at extreme distances where world generation might be active
        distance_from_origin > 5000.0
    }

    /// Get priority adjustment based on world generation priorities
    pub fn get_world_generation_priority_adjustment(
        entity_type: EntityType,
        biome: BiomeType,
    ) -> f32 {
        // Adjust spawn priorities based on world generation needs
        match (entity_type, biome) {
            // Infrastructure first in urban areas
            (EntityType::Building, BiomeType::Urban) => 1.0,
            (EntityType::Vehicle, BiomeType::Urban) => 0.8,

            // Nature first in rural areas
            (EntityType::Tree, BiomeType::Rural) => 1.0,
            (EntityType::Building, BiomeType::Rural) => 0.6,

            // Industrial elements first in industrial areas
            (EntityType::Particle, BiomeType::Industrial) => 1.0,
            (EntityType::Building, BiomeType::Industrial) => 0.9,

            // Default priorities
            _ => 0.7,
        }
    }
}

/// Performance monitoring for integration systems
pub fn integration_performance_monitoring_system(
    mut metrics: ResMut<AdvancedSpawnMetrics>,
    time: Res<Time>,
    // TODO: Add world generation performance queries
) {
    // Monitor performance impact of integration systems
    // This helps identify bottlenecks in the spawn budget integration

    let current_time = time.elapsed_secs();

    // TODO: Implement actual performance measurement
    // For now, just log periodically
    if (current_time % 10.0) < time.delta_secs() {
        debug!(
            "Integration performance: spawn_processing_time={:.2}ms, integration_overhead=estimated",
            metrics.avg_spawn_processing_time
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_spawn_limit_calculation() {
        // Test urban biome building spawn limit
        let urban_building_limit = HierarchicalWorldIntegration::get_biome_spawn_limit(
            EntityType::Building,
            BiomeType::Urban,
            100,
            1.0,
        );
        assert!(urban_building_limit > 100); // Should be increased for urban buildings

        // Test rural biome tree spawn limit
        let rural_tree_limit = HierarchicalWorldIntegration::get_biome_spawn_limit(
            EntityType::Tree,
            BiomeType::Rural,
            100,
            1.0,
        );
        assert!(rural_tree_limit > 200); // Should be significantly increased for rural trees

        // Test performance factor impact
        let low_perf_limit = HierarchicalWorldIntegration::get_biome_spawn_limit(
            EntityType::Building,
            BiomeType::Urban,
            100,
            0.5, // Low performance
        );
        assert!(low_perf_limit < urban_building_limit); // Should be reduced for low performance
    }

    #[test]
    fn test_spawn_deferral_logic() {
        // Test close position (should not defer)
        let close_defer = HierarchicalWorldIntegration::should_defer_spawn(
            EntityType::Building,
            Vec3::new(100.0, 0.0, 100.0),
        );
        assert!(!close_defer);

        // Test far position (should defer)
        let far_defer = HierarchicalWorldIntegration::should_defer_spawn(
            EntityType::Building,
            Vec3::new(6000.0, 0.0, 6000.0),
        );
        assert!(far_defer);
    }

    #[test]
    fn test_priority_adjustments() {
        // Test urban building priority (should be high)
        let urban_building_priority =
            HierarchicalWorldIntegration::get_world_generation_priority_adjustment(
                EntityType::Building,
                BiomeType::Urban,
            );
        assert!(urban_building_priority >= 1.0);

        // Test rural tree priority (should be high)
        let rural_tree_priority =
            HierarchicalWorldIntegration::get_world_generation_priority_adjustment(
                EntityType::Tree,
                BiomeType::Rural,
            );
        assert!(rural_tree_priority >= 1.0);
    }
}
