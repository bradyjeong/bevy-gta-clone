//! Spawn Budget Policy Integration - Phase 2: Oracle's Disciplined Spawning
//!
//! **Oracle's Mandate**: "Every spawn site must respect the budget authority"
//!
//! This module provides integration helpers to connect SBP with all spawn sites
//! across factory systems, world streaming, and direct spawn calls.
//!
//! ## Integration Strategy
//!
//! 1. **Factory Integration**: Wrap spawn methods with SBP checks
//! 2. **World Streaming**: Add SBP guards to generation functions
//! 3. **Queue Processing**: Connect SBP queue to actual entity spawning
//! 4. **Despawn Hooks**: Track entity destruction to release tokens

use crate::spawn_budget_policy::{
    BiomeType, EntityType, SpawnBudgetPolicy, SpawnData, SpawnPriority, SpawnResult,
};
use bevy::prelude::*;

/// Spawn request context for budget-aware spawning
#[derive(Debug, Clone)]
pub struct SpawnContext {
    pub entity_type: EntityType,
    pub biome: BiomeType,
    pub priority: SpawnPriority,
    pub position: Vec3,
}

/// Budget-aware spawn helper trait for factory integration
pub trait BudgetAwareSpawn {
    /// Attempt to spawn with budget enforcement
    fn spawn_with_budget(
        &self,
        commands: &mut Commands,
        policy: &mut ResMut<SpawnBudgetPolicy>,
        context: SpawnContext,
        spawn_data: SpawnData,
        time: &Res<Time>,
    ) -> SpawnResult;
}

/// Budget enforcement system for processing queued spawns
pub fn process_budget_queue_spawns(
    mut commands: Commands,
    mut policy: ResMut<SpawnBudgetPolicy>,
    time: Res<Time>,
) {
    let game_time = time.elapsed_secs();
    let spawned = policy.process_spawn_queue(game_time);

    for (entity_type, spawn_data) in spawned {
        spawn_queued_entity(&mut commands, entity_type, spawn_data);
    }
}

/// Actually spawn entity from queue data
fn spawn_queued_entity(commands: &mut Commands, entity_type: EntityType, spawn_data: SpawnData) {
    match spawn_data {
        SpawnData::Building {
            position,
            building_type,
        } => {
            commands.spawn((
                Name::new(format!("Building_{}", building_type)),
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                BuildingTag { building_type },
            ));
        }
        SpawnData::Vehicle {
            position,
            vehicle_type,
        } => {
            commands.spawn((
                Name::new(format!("Vehicle_{}", vehicle_type)),
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                VehicleTag { vehicle_type },
            ));
        }
        SpawnData::Npc { position, npc_type } => {
            commands.spawn((
                Name::new(format!("NPC_{}", npc_type)),
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                NpcTag { npc_type },
            ));
        }
        SpawnData::Tree {
            position,
            tree_type,
        } => {
            commands.spawn((
                Name::new(format!("Tree_{}", tree_type)),
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                TreeTag { tree_type },
            ));
        }
        SpawnData::Particle {
            position,
            effect_type,
        } => {
            commands.spawn((
                Name::new(format!("Particle_{}", effect_type)),
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                ParticleTag { effect_type },
            ));
        }
    }
}

/// Component tags for tracking entity types (used for despawn hooks)
#[derive(Component, Debug)]
pub struct BuildingTag {
    pub building_type: String,
}

#[derive(Component, Debug)]
pub struct VehicleTag {
    pub vehicle_type: String,
}

#[derive(Component, Debug)]
pub struct NpcTag {
    pub npc_type: String,
}

#[derive(Component, Debug)]
pub struct TreeTag {
    pub tree_type: String,
}

#[derive(Component, Debug)]
pub struct ParticleTag {
    pub effect_type: String,
}

/// Despawn tracking system - releases budget tokens when entities are destroyed
pub fn track_entity_despawns(
    mut removed_buildings: RemovedComponents<BuildingTag>,
    mut removed_vehicles: RemovedComponents<VehicleTag>,
    mut removed_npcs: RemovedComponents<NpcTag>,
    mut removed_trees: RemovedComponents<TreeTag>,
    mut removed_particles: RemovedComponents<ParticleTag>,
    mut policy: ResMut<SpawnBudgetPolicy>,
) {
    // Track building despawns
    for _entity in removed_buildings.read() {
        policy.record_despawn(EntityType::Building);
    }

    // Track vehicle despawns
    for _entity in removed_vehicles.read() {
        policy.record_despawn(EntityType::Vehicle);
    }

    // Track NPC despawns
    for _entity in removed_npcs.read() {
        policy.record_despawn(EntityType::Npc);
    }

    // Track tree despawns
    for _entity in removed_trees.read() {
        policy.record_despawn(EntityType::Tree);
    }

    // Track particle despawns
    for _entity in removed_particles.read() {
        policy.record_despawn(EntityType::Particle);
    }
}

/// Biome detection helper - determines biome type from position
pub fn detect_biome_from_position(position: Vec3) -> BiomeType {
    #[cfg(feature = "unstable_hierarchical_world")]
    {
        // Use the enhanced biome detection if hierarchical world is enabled
        use crate::biome::{enhanced_detect_biome_from_position, BiomeDetector};
        let detector = BiomeDetector::default();
        enhanced_detect_biome_from_position(position, &detector)
    }
    #[cfg(not(feature = "unstable_hierarchical_world"))]
    {
        // Fallback: Simple position-based biome detection
        let distance_from_origin = position.length();
        if distance_from_origin < 1000.0 {
            BiomeType::Urban
        } else if distance_from_origin < 3000.0 {
            BiomeType::Suburban
        } else if distance_from_origin < 8000.0 {
            BiomeType::Rural
        } else {
            BiomeType::Industrial
        }
    }
}

/// Entity type detection helper
pub fn detect_entity_type_from_name(name: &str) -> Option<EntityType> {
    if name.contains("Building") || name.contains("building") {
        Some(EntityType::Building)
    } else if name.contains("Vehicle") || name.contains("vehicle") || name.contains("Car") {
        Some(EntityType::Vehicle)
    } else if name.contains("NPC") || name.contains("npc") || name.contains("pedestrian") {
        Some(EntityType::Npc)
    } else if name.contains("Tree") || name.contains("tree") || name.contains("vegetation") {
        Some(EntityType::Tree)
    } else if name.contains("Particle") || name.contains("particle") || name.contains("effect") {
        Some(EntityType::Particle)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_detection() {
        let biome = detect_biome_from_position(Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(biome, BiomeType::Urban);
    }

    #[test]
    fn test_entity_type_detection() {
        assert_eq!(
            detect_entity_type_from_name("Building_house"),
            Some(EntityType::Building)
        );
        assert_eq!(
            detect_entity_type_from_name("Vehicle_car"),
            Some(EntityType::Vehicle)
        );
        assert_eq!(
            detect_entity_type_from_name("NPC_pedestrian"),
            Some(EntityType::Npc)
        );
        assert_eq!(
            detect_entity_type_from_name("Tree_oak"),
            Some(EntityType::Tree)
        );
        assert_eq!(
            detect_entity_type_from_name("Particle_smoke"),
            Some(EntityType::Particle)
        );
        assert_eq!(detect_entity_type_from_name("unknown"), None);
    }
}
