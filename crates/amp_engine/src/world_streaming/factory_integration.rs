use super::{ChunkKey, WorldStreamer};
#[cfg(feature = "bevy16")]
use crate::batch::{BatchController, BatchJob, BatchType};
#[cfg(feature = "bevy16")]
use bevy::ecs::system::SystemId;
#[cfg(feature = "bevy16")]
use bevy::prelude::*;

#[cfg(feature = "entity_debug")]
use tracing::debug;

#[cfg(feature = "bevy16")]
#[derive(Component)]
pub struct ChunkContentGenerator {
    pub chunk_key: ChunkKey,
    pub generation_type: ContentGenerationType,
}

#[cfg(feature = "bevy16")]
#[derive(Debug, Clone, Copy)]
pub enum ContentGenerationType {
    Buildings,
    Vehicles,
    NPCs,
    Environment,
}

#[cfg(feature = "bevy16")]
impl ContentGenerationType {
    fn to_batch_type(&self) -> BatchType {
        match self {
            ContentGenerationType::Buildings => BatchType::Buildings,
            ContentGenerationType::Vehicles => BatchType::Vehicles,
            ContentGenerationType::NPCs => BatchType::NPCs,
            ContentGenerationType::Environment => BatchType::Environment,
        }
    }
}

#[cfg(feature = "bevy16")]
pub fn start_chunk_generation(mut commands: Commands, mut streamer: ResMut<WorldStreamer>) {
    // Collect chunks that need content generation
    let chunks_to_generate: Vec<ChunkKey> = streamer
        .loaded_chunks
        .iter()
        .filter_map(|(chunk_key, chunk_data)| {
            if !chunk_data.content_generated {
                Some(*chunk_key)
            } else {
                None
            }
        })
        .collect();

    // Start generation for chunks that need content
    for chunk_key in chunks_to_generate {
        // Mark content as generated to prevent future spawning
        if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
            chunk_data.content_generated = true;

            // Spawn content generator for buildings
            commands.spawn(ChunkContentGenerator {
                chunk_key,
                generation_type: ContentGenerationType::Buildings,
            });
        }
    }
}

#[cfg(feature = "bevy16")]
pub fn generate_chunk_content(
    mut commands: Commands,
    mut batch_controller: ResMut<BatchController>,
    mut generators: Query<(Entity, &mut ChunkContentGenerator)>,
    mut streamer: ResMut<WorldStreamer>,
) {
    const MAX_GENERATIONS_PER_FRAME: usize = 2;

    let mut processed = 0;
    for (entity, generator) in generators.iter_mut() {
        if processed >= MAX_GENERATIONS_PER_FRAME {
            break;
        }

        // Create batch job for content generation
        let batch_job = BatchJob::new(
            SystemId::from_entity(Entity::from_raw(1)), // Mock system ID
            generator.generation_type.to_batch_type(),
            10,  // Mock entity count
            0.5, // Mock weight cost
        );

        // Submit to batch controller
        batch_controller.submit_job(batch_job);

        // Generate content based on type
        match generator.generation_type {
            ContentGenerationType::Buildings => {
                generate_buildings(&mut commands, generator.chunk_key, &mut streamer);
            }
            ContentGenerationType::Vehicles => {
                generate_vehicles(&mut commands, generator.chunk_key, &mut streamer);
            }
            ContentGenerationType::NPCs => {
                generate_npcs(&mut commands, generator.chunk_key, &mut streamer);
            }
            ContentGenerationType::Environment => {
                generate_environment(&mut commands, generator.chunk_key, &mut streamer);
            }
        }

        // Remove the generator
        commands.entity(entity).despawn();
        processed += 1;
    }
}

#[cfg(feature = "bevy16")]
fn generate_buildings(commands: &mut Commands, chunk_key: ChunkKey, streamer: &mut WorldStreamer) {
    let (chunk_x, chunk_z) = chunk_key.to_world_position(streamer.chunk_size);

    // Generate buildings in a grid pattern
    for i in 0..3 {
        for j in 0..3 {
            let building_x = chunk_x + (i as f32 * 60.0) - 90.0;
            let building_z = chunk_z + (j as f32 * 60.0) - 90.0;

            let entity = commands
                .spawn((
                    ChunkEntity { chunk_key },
                    MockBuilding,
                    Transform::from_translation(Vec3::new(building_x, 0.0, building_z)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            // Add entity to chunk with cap guard
            if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
                if chunk_data.can_add_entity(streamer.entity_limit_per_chunk) {
                    chunk_data.add_entity(entity);
                }
            }
        }
    }
}

#[cfg(feature = "bevy16")]
fn generate_vehicles(commands: &mut Commands, chunk_key: ChunkKey, streamer: &mut WorldStreamer) {
    let (chunk_x, chunk_z) = chunk_key.to_world_position(streamer.chunk_size);

    // Generate fewer vehicles scattered around
    for i in 0..2 {
        let vehicle_x = chunk_x + (i as f32 * 100.0) - 50.0;
        let vehicle_z = chunk_z + (i as f32 * 100.0) - 50.0;

        let entity = commands
            .spawn((
                ChunkEntity { chunk_key },
                MockVehicle,
                Transform::from_translation(Vec3::new(vehicle_x, 0.0, vehicle_z)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();

        // Add entity to chunk with cap guard
        if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
            if chunk_data.can_add_entity(streamer.entity_limit_per_chunk) {
                chunk_data.add_entity(entity);
            }
        }
    }
}

#[cfg(feature = "bevy16")]
fn generate_npcs(commands: &mut Commands, chunk_key: ChunkKey, streamer: &mut WorldStreamer) {
    let (chunk_x, chunk_z) = chunk_key.to_world_position(streamer.chunk_size);

    // Generate NPCs at random positions
    for i in 0..5 {
        let npc_x = chunk_x + (i as f32 * 40.0) - 80.0;
        let npc_z = chunk_z + (i as f32 * 40.0) - 80.0;

        let entity = commands
            .spawn((
                ChunkEntity { chunk_key },
                MockNPC,
                Transform::from_translation(Vec3::new(npc_x, 0.0, npc_z)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();

        // Add entity to chunk with cap guard
        if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
            if chunk_data.can_add_entity(streamer.entity_limit_per_chunk) {
                chunk_data.add_entity(entity);
            }
        }
    }
}

#[cfg(feature = "bevy16")]
fn generate_environment(
    commands: &mut Commands,
    chunk_key: ChunkKey,
    streamer: &mut WorldStreamer,
) {
    let (chunk_x, chunk_z) = chunk_key.to_world_position(streamer.chunk_size);

    // Generate trees and other environmental objects
    for i in 0..8 {
        let tree_x = chunk_x + (i as f32 * 25.0) - 87.5;
        let tree_z = chunk_z + (i as f32 * 25.0) - 87.5;

        let entity = commands
            .spawn((
                ChunkEntity { chunk_key },
                MockTree,
                Transform::from_translation(Vec3::new(tree_x, 0.0, tree_z)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();

        // Add entity to chunk with cap guard
        if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
            if chunk_data.can_add_entity(streamer.entity_limit_per_chunk) {
                chunk_data.add_entity(entity);
            }
        }
    }
}

/// Marker component for entities that belong to a chunk
#[cfg(feature = "bevy16")]
#[derive(Component, Debug, Clone)]
pub struct ChunkEntity {
    pub chunk_key: ChunkKey,
}

// Mock components for testing
#[cfg(feature = "bevy16")]
#[derive(Component)]
pub struct MockBuilding;

#[cfg(feature = "bevy16")]
#[derive(Component)]
pub struct MockVehicle;

#[cfg(feature = "bevy16")]
#[derive(Component)]
pub struct MockNPC;

#[cfg(feature = "bevy16")]
#[derive(Component)]
pub struct MockTree;

#[cfg(feature = "bevy16")]
pub fn track_chunk_entities(
    mut commands: Commands,
    streamer: Res<WorldStreamer>,
    chunk_entities: Query<(Entity, &ChunkEntity), Added<ChunkEntity>>,
) {
    for (entity, chunk_entity) in chunk_entities.iter() {
        // Track entities added to chunks
        #[cfg(feature = "entity_debug")]
        debug!(
            "Entity {:?} added to chunk {:?}",
            entity, chunk_entity.chunk_key
        );
    }
}

#[cfg(feature = "bevy16")]
pub fn cleanup_chunk_entities(
    mut streamer: ResMut<WorldStreamer>,
    mut removed_entities: RemovedComponents<ChunkEntity>,
    chunk_entities: Query<&ChunkEntity>,
) {
    for entity in removed_entities.read() {
        // Clean up tracking for removed entities
        #[cfg(feature = "entity_debug")]
        debug!("Entity {:?} removed from chunk tracking", entity);
    }
}
