//! Region management using Morton codes for spatial partitioning
//!
//! This module provides efficient spatial region identification and management
//! using Morton codes (Z-order curves) for hierarchical spatial partitioning.

use amp_math::morton::{morton_decode_2d, morton_encode_2d};
use glam::Vec2;
use std::fmt;

/// Unique identifier for spatial regions using Morton codes
///
/// RegionId uses Morton codes (Z-order curves) to provide efficient
/// spatial locality and hierarchical organization of regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionId(pub u64);

impl RegionId {
    /// Create a new RegionId from Morton code
    pub fn new(morton_code: u64) -> Self {
        Self(morton_code)
    }

    /// Create a RegionId from 2D coordinates
    pub fn from_coords(x: u32, y: u32) -> Self {
        Self(morton_encode_2d(x, y))
    }

    /// Get the underlying Morton code
    pub fn morton_code(&self) -> u64 {
        self.0
    }

    /// Decode Morton code back to 2D coordinates
    pub fn to_coords(&self) -> (u32, u32) {
        morton_decode_2d(self.0)
    }

    /// Get the parent region at a higher level in the hierarchy
    pub fn parent(&self) -> Self {
        Self(self.0 >> 2) // Shift right by 2 bits (1 level up)
    }

    /// Get child regions at a lower level in the hierarchy
    pub fn children(&self) -> [Self; 4] {
        let base = self.0 << 2; // Shift left by 2 bits
        [Self(base), Self(base + 1), Self(base + 2), Self(base + 3)]
    }

    /// Get the level of this region in the hierarchy
    pub fn level(&self) -> u8 {
        ((64 - self.0.leading_zeros()) / 2) as u8
    }

    /// Get neighboring regions at the same level
    pub fn neighbors(&self) -> Vec<Self> {
        let (x, y) = self.to_coords();
        let mut neighbors = Vec::new();

        // Add all 8 neighboring regions
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 {
                    neighbors.push(Self::from_coords(nx as u32, ny as u32));
                }
            }
        }

        neighbors
    }
}

impl fmt::Display for RegionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.to_coords();
        write!(f, "Region({x}, {y})")
    }
}

/// Spatial region with bounds and metadata
#[derive(Debug, Clone)]
pub struct Region {
    /// Unique identifier for this region
    pub id: RegionId,
    /// Spatial bounds of this region
    pub bounds: RegionBounds,
    /// LOD level of this region
    pub level: u8,
}

/// Axis-aligned bounding box for a region
#[derive(Debug, Clone)]
pub struct RegionBounds {
    /// Minimum corner coordinates
    pub min: Vec2,
    /// Maximum corner coordinates
    pub max: Vec2,
}

impl RegionBounds {
    /// Create new region bounds
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Get the center point of the region
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the region
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Check if a point is inside the region
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if this region intersects with another
    pub fn intersects(&self, other: &RegionBounds) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

impl Region {
    /// Create a new region
    pub fn new(id: RegionId, bounds: RegionBounds, level: u8) -> Self {
        Self { id, bounds, level }
    }

    /// Create a region from world coordinates and level
    pub fn from_world_coords(world_pos: Vec2, level: u8, region_size: f32) -> Self {
        let scale = region_size * (1 << level) as f32;
        let grid_x = (world_pos.x / scale).floor() as u32;
        let grid_y = (world_pos.y / scale).floor() as u32;

        let id = RegionId::from_coords(grid_x, grid_y);
        let min = Vec2::new(grid_x as f32 * scale, grid_y as f32 * scale);
        let max = min + Vec2::splat(scale);
        let bounds = RegionBounds::new(min, max);

        Self::new(id, bounds, level)
    }

    /// Get all regions that intersect with a given area
    pub fn get_regions_in_area(area: &RegionBounds, level: u8, region_size: f32) -> Vec<Region> {
        let scale = region_size * (1 << level) as f32;

        let min_x = (area.min.x / scale).floor() as u32;
        let max_x = (area.max.x / scale).ceil() as u32;
        let min_y = (area.min.y / scale).floor() as u32;
        let max_y = (area.max.y / scale).ceil() as u32;

        let mut regions = Vec::new();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let id = RegionId::from_coords(x, y);
                let min = Vec2::new(x as f32 * scale, y as f32 * scale);
                let max = min + Vec2::splat(scale);
                let bounds = RegionBounds::new(min, max);
                regions.push(Region::new(id, bounds, level));
            }
        }

        regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_id_creation() {
        let id = RegionId::from_coords(10, 20);
        let (x, y) = id.to_coords();
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }

    #[test]
    fn test_region_id_hierarchy() {
        let id = RegionId::from_coords(4, 6);
        let parent = id.parent();
        let children = parent.children();

        // Children should contain the original region
        assert_eq!(children.len(), 4);
        assert!(children.contains(&id));

        // Test that parent-child relationship is consistent
        for child in &children {
            assert_eq!(child.parent(), parent);
        }
    }

    #[test]
    fn test_region_id_neighbors() {
        let id = RegionId::from_coords(5, 5);
        let neighbors = id.neighbors();

        // Should have 8 neighbors
        assert_eq!(neighbors.len(), 8);

        // Check some expected neighbors
        assert!(neighbors.contains(&RegionId::from_coords(4, 4)));
        assert!(neighbors.contains(&RegionId::from_coords(6, 6)));
        assert!(neighbors.contains(&RegionId::from_coords(5, 4)));
    }

    #[test]
    fn test_region_bounds_contains() {
        let bounds = RegionBounds::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        assert!(bounds.contains_point(Vec2::new(5.0, 5.0)));
        assert!(bounds.contains_point(Vec2::new(0.0, 0.0)));
        assert!(bounds.contains_point(Vec2::new(10.0, 10.0)));
        assert!(!bounds.contains_point(Vec2::new(-1.0, 5.0)));
        assert!(!bounds.contains_point(Vec2::new(11.0, 5.0)));
    }

    #[test]
    fn test_region_bounds_intersects() {
        let bounds1 = RegionBounds::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let bounds2 = RegionBounds::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let bounds3 = RegionBounds::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));

        assert!(bounds1.intersects(&bounds2));
        assert!(bounds2.intersects(&bounds1));
        assert!(!bounds1.intersects(&bounds3));
        assert!(!bounds3.intersects(&bounds1));
    }

    #[test]
    fn test_region_from_world_coords() {
        let region = Region::from_world_coords(Vec2::new(150.0, 250.0), 0, 100.0);

        assert_eq!(region.level, 0);
        assert_eq!(region.bounds.min, Vec2::new(100.0, 200.0));
        assert_eq!(region.bounds.max, Vec2::new(200.0, 300.0));
    }

    #[test]
    fn test_region_get_regions_in_area() {
        let area = RegionBounds::new(Vec2::new(50.0, 50.0), Vec2::new(150.0, 150.0));
        let regions = Region::get_regions_in_area(&area, 0, 100.0);

        // The area spans from (50, 50) to (150, 150) with region size 100
        // min_x = floor(50/100) = 0, max_x = ceil(150/100) = 2
        // min_y = floor(50/100) = 0, max_y = ceil(150/100) = 2
        // So we get x in [0, 1, 2] and y in [0, 1, 2] = 3x3 = 9 regions
        assert_eq!(regions.len(), 9);

        // Check that all regions are at the correct level
        for region in &regions {
            assert_eq!(region.level, 0);
        }
    }

    #[test]
    fn test_region_id_display() {
        let id = RegionId::from_coords(42, 84);
        let display = format!("{id}");
        assert_eq!(display, "Region(42, 84)");
    }

    #[test]
    fn test_region_bounds_center_and_size() {
        let bounds = RegionBounds::new(Vec2::new(10.0, 20.0), Vec2::new(30.0, 60.0));

        assert_eq!(bounds.center(), Vec2::new(20.0, 40.0));
        assert_eq!(bounds.size(), Vec2::new(20.0, 40.0));
    }
}
