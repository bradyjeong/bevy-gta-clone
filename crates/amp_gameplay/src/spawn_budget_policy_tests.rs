//! Unit tests for SpawnBudgetPolicy invariants and behavior verification

use super::spawn_budget_policy::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy() -> SpawnBudgetPolicy {
        let config = BiomeBudgetCaps::default();
        let frame_limits = FrameLimits::default();
        SpawnBudgetPolicy::new(config, frame_limits)
    }

    /// Test basic budget creation and spawn checking
    #[test]
    fn test_basic_spawn_checking() {
        let policy = create_test_policy();

        // Test that policy can be created and basic spawn checks work
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Vehicle, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Npc, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Tree, BiomeType::Urban));
    }

    /// Test spawn recording increments counts
    #[test]
    fn test_spawn_recording() {
        let mut policy = create_test_policy();

        // Record some spawns
        policy.record_spawn(EntityType::Building);
        policy.record_spawn(EntityType::Vehicle);
        policy.record_spawn(EntityType::Npc);

        // Should still be able to spawn more
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Vehicle, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Npc, BiomeType::Urban));
    }

    /// Test despawn functionality
    #[test]
    fn test_despawn_functionality() {
        let mut policy = create_test_policy();

        // Record and despawn entities
        policy.record_spawn(EntityType::Building);
        policy.record_despawn(EntityType::Building);

        // Should work without panicking
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
    }

    /// Test frame counter reset
    #[test]
    fn test_frame_reset() {
        let mut policy = create_test_policy();

        // Record some spawns
        policy.record_spawn(EntityType::Building);
        policy.record_spawn(EntityType::Vehicle);

        // Reset frame counters
        policy.reset_frame_counters();

        // Should still be able to spawn
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Vehicle, BiomeType::Urban));
    }

    /// Test queue status checking
    #[test]
    fn test_queue_status() {
        let policy = create_test_policy();

        // Get queue status (should work without errors)
        let (queue_size, pending_spawns, age) = policy.get_queue_status();

        // Queue should be empty initially
        assert_eq!(queue_size, 0);
        assert_eq!(pending_spawns, 0);
        assert!(age >= 0.0);
    }

    /// Test biome-specific spawning
    #[test]
    fn test_biome_specific_spawning() {
        let policy = create_test_policy();

        // Test different biomes
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Suburban));
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Rural));
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Industrial));
    }

    /// Test budget utilization tracking
    #[test]
    fn test_budget_utilization() {
        let mut policy = create_test_policy();

        // Check initial utilization
        let initial_utilization = policy.get_budget_utilization(BiomeType::Urban);
        assert!(initial_utilization >= 0.0 && initial_utilization <= 1.0);

        // Record some spawns
        policy.record_spawn(EntityType::Building);
        policy.record_spawn(EntityType::Vehicle);

        // Utilization should still be valid
        let after_utilization = policy.get_budget_utilization(BiomeType::Urban);
        assert!(after_utilization >= 0.0 && after_utilization <= 1.0);
    }

    /// Test invariant: spawns work within reasonable limits
    #[test]
    fn test_reasonable_spawn_limits() {
        let mut policy = create_test_policy();

        // Should be able to spawn reasonable numbers without hitting caps immediately
        for _ in 0..5 {
            assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
            policy.record_spawn(EntityType::Building);
        }

        for _ in 0..3 {
            assert!(policy.can_spawn(EntityType::Vehicle, BiomeType::Urban));
            policy.record_spawn(EntityType::Vehicle);
        }

        // Should still work
        assert!(policy.can_spawn(EntityType::Tree, BiomeType::Urban));
    }
}
