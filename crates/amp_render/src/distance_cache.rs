//! Distance caching system for optimizing spatial queries.
//!
//! This module provides a Bevy plugin and systems for caching distance calculations
//! to reduce expensive Vec3::distance_squared calls in culling and LOD systems.

use amp_math::spatial::{CacheStats, DistanceCache};
use bevy::{prelude::*, transform::components::Transform};

/// Bevy plugin for the distance caching system.
pub struct DistanceCachePlugin;

impl Plugin for DistanceCachePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DistanceCacheResource::default())
            .insert_resource(FrameCounter::default())
            .add_systems(
                PostUpdate,
                (
                    increment_frame_counter,
                    prefill_distance_cache,
                    cleanup_expired_cache_entries,
                )
                    .chain()
                    .in_set(DistanceCacheSet),
            );
    }
}

/// System set for distance cache operations.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct DistanceCacheSet;

/// Resource wrapper for the distance cache.
#[derive(Resource)]
pub struct DistanceCacheResource {
    pub cache: DistanceCache,
}

impl Default for DistanceCacheResource {
    fn default() -> Self {
        Self {
            cache: DistanceCache::new(),
        }
    }
}

impl DistanceCacheResource {
    /// Get cached distance or compute and cache it.
    pub fn distance(
        &mut self,
        camera_pos: Vec3,
        entity_pos: Vec3,
        entity_id: Entity,
        frame: u32,
    ) -> f32 {
        // Convert Entity to u32 using the entity index
        let entity_key = entity_id.index();
        self.cache
            .get_or_compute_distance_with_frame(camera_pos, entity_pos, entity_key, frame)
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Frame counter for TTL tracking.
#[derive(Resource, Default)]
pub struct FrameCounter {
    pub frame: u32,
}

/// Component marker for entities with dirty transforms.
#[derive(Component)]
pub struct DirtyTransform;

/// System to increment the frame counter.
fn increment_frame_counter(mut frame_counter: ResMut<FrameCounter>) {
    frame_counter.frame = frame_counter.frame.wrapping_add(1);
}

/// System to prefill the distance cache with entities that have dirty transforms.
/// This system runs early in PostUpdate to ensure cache is populated before culling systems.
fn prefill_distance_cache(
    mut distance_cache: ResMut<DistanceCacheResource>,
    frame_counter: Res<FrameCounter>,
    query: Query<(Entity, &Transform), (With<DirtyTransform>, Without<Camera>)>,
    camera_query: Query<&Transform, (With<Camera>, Without<DirtyTransform>)>,
    mut commands: Commands,
) {
    // Get camera position (assume single camera for now)
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    // Prefill cache for dirty entities
    for (entity, transform) in query.iter() {
        let entity_pos = transform.translation;
        distance_cache.distance(camera_pos, entity_pos, entity, frame_counter.frame);

        // Remove dirty marker after processing
        commands.entity(entity).remove::<DirtyTransform>();
    }
}

/// System to cleanup expired cache entries periodically.
fn cleanup_expired_cache_entries(
    mut distance_cache: ResMut<DistanceCacheResource>,
    frame_counter: Res<FrameCounter>,
) {
    // Cleanup every 60 frames (approximately once per second at 60 FPS)
    if frame_counter.frame % 60 == 0 {
        distance_cache.cache.cleanup_expired(frame_counter.frame);
    }
}

/// Helper function to get cached distance between camera and entity.
/// This is the main API for other systems to use.
pub fn get_cached_distance(
    distance_cache: &mut DistanceCacheResource,
    frame_counter: &FrameCounter,
    camera_pos: Vec3,
    entity_pos: Vec3,
    entity: Entity,
) -> f32 {
    distance_cache.distance(camera_pos, entity_pos, entity, frame_counter.frame)
}

/// Helper function to mark an entity as having a dirty transform.
pub fn mark_transform_dirty(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(DirtyTransform);
}

/// Extension trait for easier access to distance cache functionality.
pub trait DistanceCacheExt {
    /// Get cached distance to an entity.
    fn cached_distance_to(
        &mut self,
        camera_pos: Vec3,
        entity_pos: Vec3,
        entity: Entity,
        frame: u32,
    ) -> f32;

    /// Get cache statistics.
    fn distance_cache_stats(&self) -> CacheStats;
}

impl DistanceCacheExt for DistanceCacheResource {
    fn cached_distance_to(
        &mut self,
        camera_pos: Vec3,
        entity_pos: Vec3,
        entity: Entity,
        frame: u32,
    ) -> f32 {
        self.distance(camera_pos, entity_pos, entity, frame)
    }

    fn distance_cache_stats(&self) -> CacheStats {
        self.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_distance_cache_resource_creation() {
        let resource = DistanceCacheResource::default();
        assert!(resource.cache.is_empty());
    }

    #[test]
    fn test_distance_cache_resource_distance() {
        let mut resource = DistanceCacheResource::default();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        let distance = resource.distance(camera_pos, entity_pos, entity, 0);
        assert_eq!(distance, 10.0);
    }

    #[test]
    fn test_get_cached_distance_helper() {
        let mut resource = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        let distance = get_cached_distance(
            &mut resource,
            &frame_counter,
            camera_pos,
            entity_pos,
            entity,
        );
        assert_eq!(distance, 10.0);
    }

    #[test]
    fn test_frame_counter_increment() {
        let mut frame_counter = FrameCounter::default();
        assert_eq!(frame_counter.frame, 0);

        frame_counter.frame = frame_counter.frame.wrapping_add(1);
        assert_eq!(frame_counter.frame, 1);
    }

    #[test]
    fn test_distance_cache_ext_trait() {
        let mut resource = DistanceCacheResource::default();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        let distance = resource.cached_distance_to(camera_pos, entity_pos, entity, 0);
        assert_eq!(distance, 10.0);

        let stats = resource.distance_cache_stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_dirty_transform_component() {
        let mut world = World::new();
        let entity = world.spawn(DirtyTransform).id();

        assert!(world.get::<DirtyTransform>(entity).is_some());
    }
}
