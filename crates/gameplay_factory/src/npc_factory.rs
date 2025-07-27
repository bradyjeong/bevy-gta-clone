//! NPC factory for spawning complete NPC entities with behavior systems.
//!
//! This module provides the NpcFactory struct that converts NpcConfig
//! into proper Bevy entities with AI components and behavior systems.

use amp_core::Error;
use amp_gameplay::npc::{
    LastUpdateFrame, NpcBrainHandle, NpcBundle, NpcConfig, NpcState, NpcType, NPC,
};
use amp_gameplay::spawn_budget_integration::{detect_biome_from_position, NpcTag};
use amp_gameplay::spawn_budget_policy::{
    EntityType, SpawnBudgetPolicy, SpawnData, SpawnPriority, SpawnResult,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Factory for creating NPC entities from NpcConfig.
///
/// This factory handles the complete spawning of an NPC entity with:
/// - NPC component with behavior properties
/// - NpcState for finite state machine
/// - NpcBrainHandle for batch processing
/// - LastUpdateFrame for timing
/// - Transform and visibility components
#[derive(Default, Debug, PartialEq)]
pub struct NpcFactory;

impl NpcFactory {
    /// Create a new NpcFactory instance.
    pub fn new() -> Self {
        Self
    }

    /// Spawn a complete NPC entity from NpcConfig.
    ///
    /// This method creates:
    /// 1. An NPC entity with NPC, NpcState, NpcBrainHandle, and LastUpdateFrame components
    /// 2. Transform and visibility components for rendering
    /// 3. Configurable NPC type and behavior properties
    ///
    /// Returns the Entity ID of the NPC entity.
    /// Spawn a single NPC entity with budget enforcement
    pub fn spawn_npc_with_budget(
        &self,
        commands: &mut Commands,
        policy: &mut ResMut<SpawnBudgetPolicy>,
        config: &NpcConfig,
        npc_type: NpcType,
        position: Vec3,
        npc_id: u32,
        priority: SpawnPriority,
        time: &Res<Time>,
    ) -> Result<SpawnResult, Error> {
        let biome = detect_biome_from_position(position);
        let game_time = time.elapsed_secs();

        let spawn_data = SpawnData::Npc {
            position,
            npc_type: format!("{:?}", npc_type),
        };

        let result = policy.request_spawn(
            EntityType::Npc,
            biome,
            priority,
            spawn_data.clone(),
            game_time,
        );

        match result {
            SpawnResult::Approved => {
                // Immediate spawn
                let _entity =
                    self.spawn_npc_immediate(commands, config, npc_type, position, npc_id)?;
                Ok(SpawnResult::Approved)
            }
            SpawnResult::Queued => Ok(SpawnResult::Queued),
            SpawnResult::Rejected(reason) => Ok(SpawnResult::Rejected(reason)),
        }
    }

    /// Spawn a single NPC entity (original method, now internal)
    pub fn spawn_npc(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        npc_type: NpcType,
        position: Vec3,
        npc_id: u32,
    ) -> Result<Entity, Error> {
        self.spawn_npc_immediate(commands, config, npc_type, position, npc_id)
    }

    /// Internal immediate spawn method
    fn spawn_npc_immediate(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        npc_type: NpcType,
        position: Vec3,
        npc_id: u32,
    ) -> Result<Entity, Error> {
        // Create NPC component from config
        let npc = NPC {
            id: npc_id,
            npc_type,
            speed: config.npc_behavior.movement.walk_speed,
            health: 100.0,
            max_health: 100.0,
            energy: config.npc_behavior.emotions.energy_levels.max_energy,
            stress: 0.0,
            last_decision_time: 0.0,
            target: None,
            memory_duration: config.npc_behavior.ai.memory_duration,
        };

        // Create NpcState component with default idle state
        let npc_state = NpcState::default();

        // Create NpcBrainHandle for batch processing
        let brain_handle = NpcBrainHandle {
            batch_handle: npc_id,
            priority: 4, // BatchType::AI priority
            cost: 1.0,
            distance_to_player: f32::MAX,
            frames_since_update: 0,
            update_interval: 1, // Every frame initially
        };

        // Create LastUpdateFrame component
        let last_update_frame = LastUpdateFrame::default();

        // Create transform at specified position
        let transform = Transform::from_translation(position);

        // Create the NPC bundle
        let npc_bundle = NpcBundle {
            npc,
            state: npc_state,
            brain_handle,
            last_update_frame,
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        };

        // Spawn the NPC entity with budget tracking tag
        let entity = commands
            .spawn((
                npc_bundle,
                NpcTag {
                    npc_type: format!("{:?}", npc_type),
                },
            ))
            .id();

        Ok(entity)
    }

    /// Spawn multiple NPCs in a batch with budget enforcement
    pub fn spawn_npcs_batch_with_budget(
        &self,
        commands: &mut Commands,
        policy: &mut ResMut<SpawnBudgetPolicy>,
        config: &NpcConfig,
        spawn_requests: &[NpcSpawnRequest],
        priority: SpawnPriority,
        time: &Res<Time>,
    ) -> Result<Vec<SpawnResult>, Error> {
        let mut results = Vec::with_capacity(spawn_requests.len());

        for request in spawn_requests {
            let result = self.spawn_npc_with_budget(
                commands,
                policy,
                config,
                request.npc_type,
                request.position,
                request.npc_id,
                priority,
                time,
            )?;
            results.push(result);
        }

        Ok(results)
    }

    /// Spawn multiple NPCs in a batch (original method)
    pub fn spawn_npcs_batch(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        spawn_requests: &[NpcSpawnRequest],
    ) -> Result<Vec<Entity>, Error> {
        let mut entities = Vec::with_capacity(spawn_requests.len());

        for request in spawn_requests {
            let entity = self.spawn_npc(
                commands,
                config,
                request.npc_type,
                request.position,
                request.npc_id,
            )?;
            entities.push(entity);
        }

        Ok(entities)
    }

    /// Create a random NPC configuration for variety
    pub fn create_random_npc_config(
        &self,
        base_config: &NpcConfig,
        npc_type: NpcType,
    ) -> NpcConfigVariant {
        let appearance = &base_config.npc_behavior.appearance;

        // Select random appearance elements
        let skin_tone = appearance
            .skin_tones
            .get((rand::random::<f32>() * appearance.skin_tones.len() as f32) as usize)
            .cloned()
            .unwrap_or(Color::WHITE);

        let hair_color = appearance
            .hair_colors
            .get((rand::random::<f32>() * appearance.hair_colors.len() as f32) as usize)
            .cloned()
            .unwrap_or(Color::BLACK);

        let clothing_color = appearance
            .clothing_colors
            .get((rand::random::<f32>() * appearance.clothing_colors.len() as f32) as usize)
            .cloned()
            .unwrap_or(Color::srgb(0.0, 0.0, 1.0));

        // Random height and build variation
        let height_range = appearance.height_variation;
        let height = height_range.0 + (rand::random::<f32>() * (height_range.1 - height_range.0));

        let build_range = appearance.build_variation;
        let build = build_range.0 + (rand::random::<f32>() * (build_range.1 - build_range.0));

        // Speed variation based on NPC type
        let speed_multiplier = match npc_type {
            NpcType::Civilian => 1.0,
            NpcType::Police => 1.2,
            NpcType::Security => 1.1,
            NpcType::Vendor => 0.8,
        };

        NpcConfigVariant {
            npc_type,
            skin_tone,
            hair_color,
            clothing_color,
            height,
            build,
            speed_multiplier,
        }
    }
}

/// Request structure for spawning NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawnRequest {
    /// Unique NPC identifier
    pub npc_id: u32,
    /// Type of NPC to spawn
    pub npc_type: NpcType,
    /// Position to spawn at
    pub position: Vec3,
}

/// Variant configuration for NPC appearance and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcConfigVariant {
    /// Type of NPC
    pub npc_type: NpcType,
    /// Skin tone color
    pub skin_tone: Color,
    /// Hair color
    pub hair_color: Color,
    /// Clothing color
    pub clothing_color: Color,
    /// Height multiplier
    pub height: f32,
    /// Build multiplier
    pub build: f32,
    /// Speed multiplier
    pub speed_multiplier: f32,
}

/// Unified Entity Factory extension for NPC spawning
pub trait UnifiedEntityFactoryExt {
    /// Spawn an NPC using the unified factory pattern
    fn spawn_npc(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        npc_type: NpcType,
        position: Vec3,
        npc_id: u32,
    ) -> Result<Entity, Error>;

    /// Spawn multiple NPCs in a batch
    fn spawn_npcs_batch(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        spawn_requests: &[NpcSpawnRequest],
    ) -> Result<Vec<Entity>, Error>;
}

/// Implementation of UnifiedEntityFactoryExt for the Factory struct
impl UnifiedEntityFactoryExt for crate::Factory {
    fn spawn_npc(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        npc_type: NpcType,
        position: Vec3,
        npc_id: u32,
    ) -> Result<Entity, Error> {
        let factory = NpcFactory::new();
        factory.spawn_npc(commands, config, npc_type, position, npc_id)
    }

    fn spawn_npcs_batch(
        &self,
        commands: &mut Commands,
        config: &NpcConfig,
        spawn_requests: &[NpcSpawnRequest],
    ) -> Result<Vec<Entity>, Error> {
        let factory = NpcFactory::new();
        factory.spawn_npcs_batch(commands, config, spawn_requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amp_gameplay::npc::NpcBehaviorConfig;

    #[test]
    fn test_npc_factory_creation() {
        let factory = NpcFactory::new();
        assert_eq!(factory, NpcFactory::default());
    }

    #[test]
    fn test_npc_spawn_request_creation() {
        let request = NpcSpawnRequest {
            npc_id: 42,
            npc_type: NpcType::Civilian,
            position: Vec3::new(1.0, 2.0, 3.0),
        };

        assert_eq!(request.npc_id, 42);
        assert_eq!(request.npc_type, NpcType::Civilian);
        assert_eq!(request.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_random_npc_config_generation() {
        let config = NpcConfig::default();
        let factory = NpcFactory::new();

        let variant = factory.create_random_npc_config(&config, NpcType::Police);

        assert_eq!(variant.npc_type, NpcType::Police);
        assert_eq!(variant.speed_multiplier, 1.2);
        assert!(variant.height >= 0.8 && variant.height <= 1.2);
        assert!(variant.build >= 0.7 && variant.build <= 1.3);
    }

    #[test]
    fn test_npc_type_speed_multipliers() {
        let config = NpcConfig::default();
        let factory = NpcFactory::new();

        let civilian = factory.create_random_npc_config(&config, NpcType::Civilian);
        let police = factory.create_random_npc_config(&config, NpcType::Police);
        let security = factory.create_random_npc_config(&config, NpcType::Security);
        let vendor = factory.create_random_npc_config(&config, NpcType::Vendor);

        assert_eq!(civilian.speed_multiplier, 1.0);
        assert_eq!(police.speed_multiplier, 1.2);
        assert_eq!(security.speed_multiplier, 1.1);
        assert_eq!(vendor.speed_multiplier, 0.8);
    }
}
