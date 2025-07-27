//! Road system resources for maintaining road entity mapping
//!
//! This module provides resources to track road_id → Entity mapping
//! as specified by Oracle's event-based architecture.

use bevy::prelude::*;
use std::collections::HashMap;

/// Resource maintaining mapping between road IDs and their entities
///
/// This enables the event system to attach meshes to the correct road entities
/// after async mesh generation completes.
#[derive(Resource, Default, Debug)]
pub struct RoadEntityMap {
    /// Maps road_id to the Entity containing the RoadEntity component
    road_entities: HashMap<u32, Entity>,
    /// Maps intersection_id to the Entity containing the IntersectionEntity component  
    intersection_entities: HashMap<u32, Entity>,
    /// Statistics for debugging
    stats: RoadEntityMapStats,
}

/// Statistics for the road entity map
#[derive(Debug, Default)]
pub struct RoadEntityMapStats {
    /// Total roads currently tracked
    pub total_roads: usize,
    /// Total intersections currently tracked
    pub total_intersections: usize,
    /// Number of map insertions
    pub insertions: u64,
    /// Number of map removals
    pub removals: u64,
    /// Number of failed lookups
    pub lookup_failures: u64,
}

impl RoadEntityMap {
    /// Create a new empty road entity map
    pub fn new() -> Self {
        Self {
            road_entities: HashMap::new(),
            intersection_entities: HashMap::new(),
            stats: RoadEntityMapStats::default(),
        }
    }

    /// Create a road entity map with initial capacity
    pub fn with_capacity(roads: usize, intersections: usize) -> Self {
        Self {
            road_entities: HashMap::with_capacity(roads),
            intersection_entities: HashMap::with_capacity(intersections),
            stats: RoadEntityMapStats::default(),
        }
    }

    /// Insert a road entity mapping
    pub fn insert_road(&mut self, road_id: u32, entity: Entity) {
        self.road_entities.insert(road_id, entity);
        self.stats.total_roads = self.road_entities.len();
        self.stats.insertions += 1;

        trace!("Inserted road {} -> {:?}", road_id, entity);
    }

    /// Insert an intersection entity mapping
    pub fn insert_intersection(&mut self, intersection_id: u32, entity: Entity) {
        self.intersection_entities.insert(intersection_id, entity);
        self.stats.total_intersections = self.intersection_entities.len();
        self.stats.insertions += 1;

        trace!("Inserted intersection {} -> {:?}", intersection_id, entity);
    }

    /// Get the entity for a road ID
    pub fn get_road_entity(&mut self, road_id: u32) -> Option<Entity> {
        match self.road_entities.get(&road_id) {
            Some(&entity) => {
                trace!("Found road {} -> {:?}", road_id, entity);
                Some(entity)
            }
            None => {
                self.stats.lookup_failures += 1;
                debug!("Road entity not found for road_id: {}", road_id);
                None
            }
        }
    }

    /// Get the entity for an intersection ID
    pub fn get_intersection_entity(&mut self, intersection_id: u32) -> Option<Entity> {
        match self.intersection_entities.get(&intersection_id) {
            Some(&entity) => {
                trace!("Found intersection {} -> {:?}", intersection_id, entity);
                Some(entity)
            }
            None => {
                self.stats.lookup_failures += 1;
                debug!(
                    "Intersection entity not found for intersection_id: {}",
                    intersection_id
                );
                None
            }
        }
    }

    /// Remove a road entity mapping
    pub fn remove_road(&mut self, road_id: u32) -> Option<Entity> {
        let result = self.road_entities.remove(&road_id);
        if result.is_some() {
            self.stats.total_roads = self.road_entities.len();
            self.stats.removals += 1;
            trace!("Removed road {} -> {:?}", road_id, result);
        }
        result
    }

    /// Remove an intersection entity mapping
    pub fn remove_intersection(&mut self, intersection_id: u32) -> Option<Entity> {
        let result = self.intersection_entities.remove(&intersection_id);
        if result.is_some() {
            self.stats.total_intersections = self.intersection_entities.len();
            self.stats.removals += 1;
            trace!("Removed intersection {} -> {:?}", intersection_id, result);
        }
        result
    }

    /// Check if a road ID is tracked
    pub fn contains_road(&self, road_id: u32) -> bool {
        self.road_entities.contains_key(&road_id)
    }

    /// Check if an intersection ID is tracked
    pub fn contains_intersection(&self, intersection_id: u32) -> bool {
        self.intersection_entities.contains_key(&intersection_id)
    }

    /// Get all tracked road IDs
    pub fn road_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.road_entities.keys().copied()
    }

    /// Get all tracked intersection IDs
    pub fn intersection_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.intersection_entities.keys().copied()
    }

    /// Get statistics about the entity map
    pub fn stats(&self) -> &RoadEntityMapStats {
        &self.stats
    }

    /// Clear all mappings (for debugging/cleanup)
    pub fn clear(&mut self) {
        let removed_roads = self.road_entities.len();
        let removed_intersections = self.intersection_entities.len();

        self.road_entities.clear();
        self.intersection_entities.clear();

        self.stats.total_roads = 0;
        self.stats.total_intersections = 0;
        self.stats.removals += (removed_roads + removed_intersections) as u64;

        info!(
            "Cleared entity map: {} roads, {} intersections",
            removed_roads, removed_intersections
        );
    }

    /// Clean up mappings for entities that no longer exist
    pub fn cleanup_invalid_entities(&mut self, world: &World) {
        let mut roads_to_remove = Vec::new();
        let mut intersections_to_remove = Vec::new();

        // Check road entities
        for (&road_id, &entity) in &self.road_entities {
            if world.get_entity(entity).is_err() {
                roads_to_remove.push(road_id);
            }
        }

        // Check intersection entities
        for (&intersection_id, &entity) in &self.intersection_entities {
            if world.get_entity(entity).is_err() {
                intersections_to_remove.push(intersection_id);
            }
        }

        // Remove invalid mappings
        for road_id in roads_to_remove {
            self.remove_road(road_id);
            debug!(
                "Cleaned up invalid road entity mapping for road {}",
                road_id
            );
        }

        for intersection_id in intersections_to_remove {
            self.remove_intersection(intersection_id);
            debug!(
                "Cleaned up invalid intersection entity mapping for intersection {}",
                intersection_id
            );
        }
    }

    /// Clean up mappings for entities that no longer exist using a set of valid entities
    pub fn cleanup_invalid_entities_with_set(
        &mut self,
        existing_entities: &std::collections::HashSet<Entity>,
    ) {
        let mut roads_to_remove = Vec::new();
        let mut intersections_to_remove = Vec::new();

        // Check road entities
        for (&road_id, &entity) in &self.road_entities {
            if !existing_entities.contains(&entity) {
                roads_to_remove.push(road_id);
            }
        }

        // Check intersection entities
        for (&intersection_id, &entity) in &self.intersection_entities {
            if !existing_entities.contains(&entity) {
                intersections_to_remove.push(intersection_id);
            }
        }

        // Remove invalid mappings
        for road_id in roads_to_remove {
            self.remove_road(road_id);
            debug!(
                "Cleaned up invalid road entity mapping for road {}",
                road_id
            );
        }

        for intersection_id in intersections_to_remove {
            self.remove_intersection(intersection_id);
            debug!(
                "Cleaned up invalid intersection entity mapping for intersection {}",
                intersection_id
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_entity_map_creation() {
        let map = RoadEntityMap::new();

        assert_eq!(map.stats.total_roads, 0);
        assert_eq!(map.stats.total_intersections, 0);
        assert_eq!(map.stats.insertions, 0);
        assert_eq!(map.stats.removals, 0);
    }

    #[test]
    fn test_road_entity_map_with_capacity() {
        let map = RoadEntityMap::with_capacity(100, 50);

        assert!(map.road_entities.capacity() >= 100);
        assert!(map.intersection_entities.capacity() >= 50);
    }

    #[test]
    fn test_road_insertion_and_lookup() {
        let mut map = RoadEntityMap::new();
        let entity = Entity::from_raw(42);

        // Insert road
        map.insert_road(123, entity);

        assert_eq!(map.stats.total_roads, 1);
        assert_eq!(map.stats.insertions, 1);
        assert!(map.contains_road(123));

        // Lookup road
        assert_eq!(map.get_road_entity(123), Some(entity));
        assert_eq!(map.get_road_entity(999), None);
        assert_eq!(map.stats.lookup_failures, 1);
    }

    #[test]
    fn test_intersection_insertion_and_lookup() {
        let mut map = RoadEntityMap::new();
        let entity = Entity::from_raw(24);

        // Insert intersection
        map.insert_intersection(456, entity);

        assert_eq!(map.stats.total_intersections, 1);
        assert_eq!(map.stats.insertions, 1);
        assert!(map.contains_intersection(456));

        // Lookup intersection
        assert_eq!(map.get_intersection_entity(456), Some(entity));
        assert_eq!(map.get_intersection_entity(888), None);
        assert_eq!(map.stats.lookup_failures, 1);
    }

    #[test]
    fn test_road_removal() {
        let mut map = RoadEntityMap::new();
        let entity = Entity::from_raw(99);

        // Insert and remove road
        map.insert_road(789, entity);
        assert_eq!(map.stats.total_roads, 1);

        let removed = map.remove_road(789);
        assert_eq!(removed, Some(entity));
        assert_eq!(map.stats.total_roads, 0);
        assert_eq!(map.stats.removals, 1);
        assert!(!map.contains_road(789));

        // Try to remove non-existent road
        let removed_again = map.remove_road(789);
        assert_eq!(removed_again, None);
    }

    #[test]
    fn test_intersection_removal() {
        let mut map = RoadEntityMap::new();
        let entity = Entity::from_raw(66);

        // Insert and remove intersection
        map.insert_intersection(321, entity);
        assert_eq!(map.stats.total_intersections, 1);

        let removed = map.remove_intersection(321);
        assert_eq!(removed, Some(entity));
        assert_eq!(map.stats.total_intersections, 0);
        assert_eq!(map.stats.removals, 1);
        assert!(!map.contains_intersection(321));
    }

    #[test]
    fn test_clear_all_mappings() {
        let mut map = RoadEntityMap::new();

        // Insert multiple roads and intersections
        map.insert_road(1, Entity::from_raw(10));
        map.insert_road(2, Entity::from_raw(20));
        map.insert_intersection(10, Entity::from_raw(100));
        map.insert_intersection(20, Entity::from_raw(200));

        assert_eq!(map.stats.total_roads, 2);
        assert_eq!(map.stats.total_intersections, 2);

        // Clear all
        map.clear();

        assert_eq!(map.stats.total_roads, 0);
        assert_eq!(map.stats.total_intersections, 0);
        assert_eq!(map.stats.removals, 4); // 2 roads + 2 intersections
    }

    #[test]
    fn test_id_iteration() {
        let mut map = RoadEntityMap::new();

        // Insert test data
        map.insert_road(1, Entity::from_raw(10));
        map.insert_road(2, Entity::from_raw(20));
        map.insert_road(3, Entity::from_raw(30));
        map.insert_intersection(10, Entity::from_raw(100));
        map.insert_intersection(20, Entity::from_raw(200));

        // Collect IDs
        let road_ids: Vec<u32> = map.road_ids().collect();
        let intersection_ids: Vec<u32> = map.intersection_ids().collect();

        assert_eq!(road_ids.len(), 3);
        assert_eq!(intersection_ids.len(), 2);

        // Check that all expected IDs are present
        assert!(road_ids.contains(&1));
        assert!(road_ids.contains(&2));
        assert!(road_ids.contains(&3));
        assert!(intersection_ids.contains(&10));
        assert!(intersection_ids.contains(&20));
    }
}
