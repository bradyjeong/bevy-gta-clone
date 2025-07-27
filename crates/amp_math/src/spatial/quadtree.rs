//! Quadtree spatial partitioning for hierarchical world generation
//!
//! Provides efficient spatial indexing and LOD management for large-scale world generation.
//! Part of the hierarchical world generation system ported from f430bc6-reference.

use glam::Vec3;
use std::collections::HashMap;

/// Hierarchical coordinate system for multi-scale world generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldCoord {
    pub level: LODLevel,
    pub x: i32,
    pub z: i32,
}

/// Level-of-detail levels for hierarchical world generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LODLevel {
    Macro = 0,  // 10km regions - procedural generation only
    Region = 1, // 2km regions - basic terrain and roads
    Local = 2,  // 400m chunks - buildings and major features
    Detail = 3, // 100m chunks - detailed objects and NPCs
    Micro = 4,  // 25m chunks - full detail with physics
}

/// World scale constants from reference implementation
pub const MACRO_REGION_SIZE: f32 = 10000.0; // 10km macro regions
pub const REGION_SIZE: f32 = 2000.0; // 2km regions
pub const LOCAL_CHUNK_SIZE: f32 = 400.0; // 400m local chunks
pub const DETAIL_CHUNK_SIZE: f32 = 100.0; // 100m detail chunks
pub const MICRO_CHUNK_SIZE: f32 = 25.0; // 25m micro chunks

/// Streaming distances for each LOD level
pub const MACRO_STREAMING_RADIUS: f32 = 50000.0; // 50km macro visibility
pub const REGION_STREAMING_RADIUS: f32 = 20000.0; // 20km region visibility
pub const LOCAL_STREAMING_RADIUS: f32 = 5000.0; // 5km local visibility
pub const DETAIL_STREAMING_RADIUS: f32 = 2000.0; // 2km detail visibility
pub const MICRO_STREAMING_RADIUS: f32 = 500.0; // 500m micro visibility

impl WorldCoord {
    pub fn new(level: LODLevel, x: i32, z: i32) -> Self {
        Self { level, x, z }
    }

    /// Create world coordinate from world position and LOD level
    pub fn from_world_pos(world_pos: Vec3, level: LODLevel) -> Self {
        let chunk_size = match level {
            LODLevel::Macro => MACRO_REGION_SIZE,
            LODLevel::Region => REGION_SIZE,
            LODLevel::Local => LOCAL_CHUNK_SIZE,
            LODLevel::Detail => DETAIL_CHUNK_SIZE,
            LODLevel::Micro => MICRO_CHUNK_SIZE,
        };

        Self {
            level,
            x: (world_pos.x / chunk_size).floor() as i32,
            z: (world_pos.z / chunk_size).floor() as i32,
        }
    }

    /// Convert world coordinate to world position (center of chunk/region)
    pub fn to_world_pos(&self) -> Vec3 {
        let chunk_size = match self.level {
            LODLevel::Macro => MACRO_REGION_SIZE,
            LODLevel::Region => REGION_SIZE,
            LODLevel::Local => LOCAL_CHUNK_SIZE,
            LODLevel::Detail => DETAIL_CHUNK_SIZE,
            LODLevel::Micro => MICRO_CHUNK_SIZE,
        };

        Vec3::new(
            self.x as f32 * chunk_size + chunk_size * 0.5,
            0.0,
            self.z as f32 * chunk_size + chunk_size * 0.5,
        )
    }

    /// Get streaming radius for this LOD level
    pub fn get_streaming_radius(&self) -> f32 {
        match self.level {
            LODLevel::Macro => MACRO_STREAMING_RADIUS,
            LODLevel::Region => REGION_STREAMING_RADIUS,
            LODLevel::Local => LOCAL_STREAMING_RADIUS,
            LODLevel::Detail => DETAIL_STREAMING_RADIUS,
            LODLevel::Micro => MICRO_STREAMING_RADIUS,
        }
    }

    /// Get parent coordinate at next higher LOD level
    pub fn get_parent(&self) -> Option<WorldCoord> {
        match self.level {
            LODLevel::Macro => None,
            LODLevel::Region => Some(WorldCoord::new(LODLevel::Macro, self.x / 5, self.z / 5)),
            LODLevel::Local => Some(WorldCoord::new(LODLevel::Region, self.x / 5, self.z / 5)),
            LODLevel::Detail => Some(WorldCoord::new(LODLevel::Local, self.x / 4, self.z / 4)),
            LODLevel::Micro => Some(WorldCoord::new(LODLevel::Detail, self.x / 4, self.z / 4)),
        }
    }

    /// Get children coordinates at next lower LOD level
    pub fn get_children(&self) -> Vec<WorldCoord> {
        let mut children = Vec::new();
        let (subdivisions, child_level) = match self.level {
            LODLevel::Macro => (5, LODLevel::Region),
            LODLevel::Region => (5, LODLevel::Local),
            LODLevel::Local => (4, LODLevel::Detail),
            LODLevel::Detail => (4, LODLevel::Micro),
            LODLevel::Micro => return children, // No children for micro level
        };

        for dx in 0..subdivisions {
            for dz in 0..subdivisions {
                children.push(WorldCoord::new(
                    child_level,
                    self.x * subdivisions + dx,
                    self.z * subdivisions + dz,
                ));
            }
        }

        children
    }
}

/// Spatial node in the quadtree
#[derive(Debug, Clone)]
pub struct QuadtreeNode {
    pub coord: WorldCoord,
    pub bounds: (Vec3, Vec3), // min, max
    pub entities: Vec<u32>,   // Entity IDs stored in this node
    pub children: Option<[Box<QuadtreeNode>; 4]>,
}

impl QuadtreeNode {
    /// Create a new quadtree node
    pub fn new(coord: WorldCoord) -> Self {
        let center = coord.to_world_pos();
        let size = match coord.level {
            LODLevel::Macro => MACRO_REGION_SIZE,
            LODLevel::Region => REGION_SIZE,
            LODLevel::Local => LOCAL_CHUNK_SIZE,
            LODLevel::Detail => DETAIL_CHUNK_SIZE,
            LODLevel::Micro => MICRO_CHUNK_SIZE,
        };

        let half_size = size * 0.5;
        let min = Vec3::new(center.x - half_size, 0.0, center.z - half_size);
        let max = Vec3::new(center.x + half_size, 0.0, center.z + half_size);

        Self {
            coord,
            bounds: (min, max),
            entities: Vec::new(),
            children: None,
        }
    }

    /// Check if point is contained within this node's bounds
    pub fn contains_point(&self, point: Vec3) -> bool {
        let (min, max) = self.bounds;
        point.x >= min.x && point.x < max.x && point.z >= min.z && point.z < max.z
    }

    /// Insert an entity into this node or its children
    pub fn insert(&mut self, entity_id: u32, position: Vec3) -> bool {
        if !self.contains_point(position) {
            return false;
        }

        // If we're at the micro level or have no children, store entity here
        if matches!(self.coord.level, LODLevel::Micro) || self.children.is_none() {
            self.entities.push(entity_id);
            return true;
        }

        // Try to insert into children if they exist
        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                if child.insert(entity_id, position) {
                    return true;
                }
            }
        }

        // Fallback: store in this node
        self.entities.push(entity_id);
        true
    }

    /// Query entities within a radius from a point
    pub fn query_radius(&self, center: Vec3, radius: f32, results: &mut Vec<u32>) {
        // Check if this node's bounds intersect with the query radius
        if !self.bounds_intersect_circle(center, radius) {
            return;
        }

        // Add entities from this node
        for &entity_id in &self.entities {
            results.push(entity_id);
        }

        // Recurse into children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_radius(center, radius, results);
            }
        }
    }

    /// Check if node bounds intersect with a circle
    fn bounds_intersect_circle(&self, center: Vec3, radius: f32) -> bool {
        let (min, max) = self.bounds;

        // Find closest point on the AABB to the circle center
        let closest_x = center.x.clamp(min.x, max.x);
        let closest_z = center.z.clamp(min.z, max.z);

        // Calculate distance from circle center to this closest point
        let distance_sq = (center.x - closest_x).powi(2) + (center.z - closest_z).powi(2);

        distance_sq <= radius.powi(2)
    }
}

/// Hierarchical quadtree for spatial partitioning
#[derive(Debug)]
pub struct HierarchicalQuadtree {
    pub root_nodes: HashMap<WorldCoord, QuadtreeNode>,
    pub max_depth: usize,
}

impl HierarchicalQuadtree {
    /// Create a new hierarchical quadtree
    pub fn new() -> Self {
        Self {
            root_nodes: HashMap::new(),
            max_depth: 5, // Supports all LOD levels
        }
    }

    /// Ensure a node exists for the given coordinate
    pub fn ensure_node(&mut self, coord: WorldCoord) -> &mut QuadtreeNode {
        self.root_nodes
            .entry(coord)
            .or_insert_with(|| QuadtreeNode::new(coord))
    }

    /// Insert an entity at a position
    pub fn insert_entity(&mut self, entity_id: u32, position: Vec3, lod_level: LODLevel) {
        let coord = WorldCoord::from_world_pos(position, lod_level);
        let node = self.ensure_node(coord);
        node.insert(entity_id, position);
    }

    /// Query entities within radius at a specific LOD level
    pub fn query_entities(&self, center: Vec3, radius: f32, lod_level: LODLevel) -> Vec<u32> {
        let mut results = Vec::new();

        // Find all nodes at this LOD level that might contain results
        let search_radius_chunks = (radius
            / match lod_level {
                LODLevel::Macro => MACRO_REGION_SIZE,
                LODLevel::Region => REGION_SIZE,
                LODLevel::Local => LOCAL_CHUNK_SIZE,
                LODLevel::Detail => DETAIL_CHUNK_SIZE,
                LODLevel::Micro => MICRO_CHUNK_SIZE,
            })
        .ceil() as i32;

        let center_coord = WorldCoord::from_world_pos(center, lod_level);

        for dx in -search_radius_chunks..=search_radius_chunks {
            for dz in -search_radius_chunks..=search_radius_chunks {
                let coord = WorldCoord::new(lod_level, center_coord.x + dx, center_coord.z + dz);

                if let Some(node) = self.root_nodes.get(&coord) {
                    node.query_radius(center, radius, &mut results);
                }
            }
        }

        results
    }

    /// Get all active coordinates for streaming
    pub fn get_streaming_coords(&self, center: Vec3) -> Vec<WorldCoord> {
        let mut coords = Vec::new();

        // Get coordinates for each LOD level within streaming radius
        for level in [
            LODLevel::Macro,
            LODLevel::Region,
            LODLevel::Local,
            LODLevel::Detail,
            LODLevel::Micro,
        ] {
            let active_coord = WorldCoord::from_world_pos(center, level);
            let streaming_radius = active_coord.get_streaming_radius();

            let chunk_size = match level {
                LODLevel::Macro => MACRO_REGION_SIZE,
                LODLevel::Region => REGION_SIZE,
                LODLevel::Local => LOCAL_CHUNK_SIZE,
                LODLevel::Detail => DETAIL_CHUNK_SIZE,
                LODLevel::Micro => MICRO_CHUNK_SIZE,
            };

            let chunk_radius = (streaming_radius / chunk_size).ceil() as i32;

            for dx in -chunk_radius..=chunk_radius {
                for dz in -chunk_radius..=chunk_radius {
                    let coord = WorldCoord::new(level, active_coord.x + dx, active_coord.z + dz);
                    let distance = center.distance(coord.to_world_pos());

                    if distance <= streaming_radius {
                        coords.push(coord);
                    }
                }
            }
        }

        coords
    }

    /// Remove a node and all its entities
    pub fn remove_node(&mut self, coord: WorldCoord) -> Option<Vec<u32>> {
        self.root_nodes.remove(&coord).map(|node| node.entities)
    }

    /// Get the number of nodes at each LOD level
    pub fn get_node_counts(&self) -> HashMap<LODLevel, usize> {
        let mut counts = HashMap::new();

        for coord in self.root_nodes.keys() {
            *counts.entry(coord.level).or_insert(0) += 1;
        }

        counts
    }
}

impl Default for HierarchicalQuadtree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_coord_creation() {
        let coord = WorldCoord::new(LODLevel::Local, 5, -3);
        assert_eq!(coord.level, LODLevel::Local);
        assert_eq!(coord.x, 5);
        assert_eq!(coord.z, -3);
    }

    #[test]
    fn test_world_coord_from_position() {
        let pos = Vec3::new(1200.0, 0.0, -800.0);
        let coord = WorldCoord::from_world_pos(pos, LODLevel::Local);

        // Local chunks are 400m, so 1200/400 = 3, -800/400 = -2
        assert_eq!(coord.x, 3);
        assert_eq!(coord.z, -2);
    }

    #[test]
    fn test_world_coord_to_position() {
        let coord = WorldCoord::new(LODLevel::Local, 2, -1);
        let pos = coord.to_world_pos();

        // Local chunks are 400m, center of chunk (2, -1) should be at (1000, 0, -200)
        assert_eq!(pos.x, 1000.0);
        assert_eq!(pos.z, -200.0);
    }

    #[test]
    fn test_parent_child_relationships() {
        let local_coord = WorldCoord::new(LODLevel::Local, 8, 12);
        let parent = local_coord.get_parent().unwrap();

        // Local to Region subdivision is 5, so 8/5 = 1, 12/5 = 2
        assert_eq!(parent.level, LODLevel::Region);
        assert_eq!(parent.x, 1);
        assert_eq!(parent.z, 2);

        let children = parent.get_children();
        assert_eq!(children.len(), 25); // 5x5 subdivision
        assert!(children.contains(&local_coord));
    }

    #[test]
    fn test_streaming_radius() {
        let macro_coord = WorldCoord::new(LODLevel::Macro, 0, 0);
        assert_eq!(macro_coord.get_streaming_radius(), MACRO_STREAMING_RADIUS);

        let micro_coord = WorldCoord::new(LODLevel::Micro, 0, 0);
        assert_eq!(micro_coord.get_streaming_radius(), MICRO_STREAMING_RADIUS);
    }

    #[test]
    fn test_quadtree_node_creation() {
        let coord = WorldCoord::new(LODLevel::Local, 0, 0);
        let node = QuadtreeNode::new(coord);

        assert_eq!(node.coord, coord);
        assert!(node.entities.is_empty());
        assert!(node.children.is_none());
    }

    #[test]
    fn test_quadtree_point_containment() {
        let coord = WorldCoord::new(LODLevel::Local, 0, 0);
        let node = QuadtreeNode::new(coord);

        // Center of chunk should be contained
        let center = coord.to_world_pos();
        assert!(node.contains_point(center));

        // Point just inside bounds should be contained
        assert!(node.contains_point(center + Vec3::new(150.0, 0.0, 150.0)));

        // Point outside bounds should not be contained
        assert!(!node.contains_point(center + Vec3::new(250.0, 0.0, 0.0)));
    }

    #[test]
    fn test_hierarchical_quadtree_insert() {
        let mut quadtree = HierarchicalQuadtree::new();

        let position = Vec3::new(100.0, 0.0, 200.0);
        quadtree.insert_entity(1, position, LODLevel::Local);

        assert_eq!(quadtree.root_nodes.len(), 1);

        let coord = WorldCoord::from_world_pos(position, LODLevel::Local);
        let node = quadtree.root_nodes.get(&coord).unwrap();
        assert_eq!(node.entities, vec![1]);
    }

    #[test]
    fn test_quadtree_query() {
        let mut quadtree = HierarchicalQuadtree::new();

        // Insert entities at different positions
        quadtree.insert_entity(1, Vec3::new(100.0, 0.0, 100.0), LODLevel::Local);
        quadtree.insert_entity(2, Vec3::new(200.0, 0.0, 200.0), LODLevel::Local);
        quadtree.insert_entity(3, Vec3::new(1000.0, 0.0, 1000.0), LODLevel::Local);

        // Query near the first two entities
        let results = quadtree.query_entities(Vec3::new(150.0, 0.0, 150.0), 300.0, LODLevel::Local);

        // Should find entities 1 and 2, but not 3 (too far)
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(!results.contains(&3));
    }

    #[test]
    fn test_streaming_coords_generation() {
        let quadtree = HierarchicalQuadtree::new();

        let center = Vec3::new(0.0, 0.0, 0.0);
        let coords = quadtree.get_streaming_coords(center);

        // Should generate coordinates for all LOD levels
        let mut level_counts = HashMap::new();
        for coord in coords {
            *level_counts.entry(coord.level).or_insert(0) += 1;
        }

        // All LOD levels should be represented
        assert!(level_counts.contains_key(&LODLevel::Macro));
        assert!(level_counts.contains_key(&LODLevel::Region));
        assert!(level_counts.contains_key(&LODLevel::Local));
        assert!(level_counts.contains_key(&LODLevel::Detail));
        assert!(level_counts.contains_key(&LODLevel::Micro));
    }
}
