//! Coordinate conversion utilities
//!
//! This module consolidates all grid-to-world coordinate conversion functions
//! that were previously duplicated across multiple files.

use glam::{IVec2, Vec2, Vec3, Vec3Swizzles};

/// Coordinate conversion utilities for grid-based systems
pub struct CoordinateConversion;

impl CoordinateConversion {
    /// Convert grid coordinates to world position
    ///
    /// This is the canonical implementation that replaces all duplicated
    /// grid_to_world functions throughout the codebase.
    pub fn grid_to_world(grid_pos: IVec2, tile_size: f32) -> Vec2 {
        Vec2::new(grid_pos.x as f32 * tile_size, grid_pos.y as f32 * tile_size)
    }

    /// Convert world position to grid coordinates
    ///
    /// This is the canonical implementation that replaces all duplicated
    /// world_to_grid functions throughout the codebase.
    pub fn world_to_grid(world_pos: Vec2, tile_size: f32) -> IVec2 {
        IVec2::new(
            (world_pos.x / tile_size).round() as i32,
            (world_pos.y / tile_size).round() as i32,
        )
    }

    /// Convert 3D world position to 2D grid coordinates (ignores Y axis)
    pub fn world_to_grid_3d(world_pos: Vec3, tile_size: f32) -> IVec2 {
        Self::world_to_grid(world_pos.xz(), tile_size)
    }

    /// Convert grid coordinates to 3D world position with specified Y height
    pub fn grid_to_world_3d(grid_pos: IVec2, tile_size: f32, y_height: f32) -> Vec3 {
        let world_2d = Self::grid_to_world(grid_pos, tile_size);
        Vec3::new(world_2d.x, y_height, world_2d.y)
    }

    /// Convert world position to sector coordinates
    pub fn world_to_sector(world_pos: Vec3, sector_size: f32) -> IVec2 {
        IVec2::new(
            (world_pos.x / sector_size).floor() as i32,
            (world_pos.z / sector_size).floor() as i32,
        )
    }

    /// Convert sector coordinates to world position (center of sector)
    pub fn sector_to_world(sector_pos: IVec2, sector_size: f32) -> Vec3 {
        Vec3::new(
            (sector_pos.x as f32 + 0.5) * sector_size,
            0.0,
            (sector_pos.y as f32 + 0.5) * sector_size,
        )
    }

    /// Convert world position to chunk coordinates
    pub fn world_to_chunk(world_pos: Vec3, chunk_size: f32) -> IVec2 {
        IVec2::new(
            (world_pos.x / chunk_size).floor() as i32,
            (world_pos.z / chunk_size).floor() as i32,
        )
    }

    /// Convert chunk coordinates to world position (center of chunk)
    pub fn chunk_to_world(chunk_pos: IVec2, chunk_size: f32) -> Vec3 {
        Vec3::new(
            (chunk_pos.x as f32 + 0.5) * chunk_size,
            0.0,
            (chunk_pos.y as f32 + 0.5) * chunk_size,
        )
    }

    /// Convert world position to region coordinates for spatial partitioning
    pub fn world_to_region(world_pos: Vec3, region_size: f32) -> IVec2 {
        IVec2::new(
            (world_pos.x / region_size).floor() as i32,
            (world_pos.z / region_size).floor() as i32,
        )
    }

    /// Convert region coordinates to world position (center of region)
    pub fn region_to_world(region_pos: IVec2, region_size: f32) -> Vec3 {
        Vec3::new(
            (region_pos.x as f32 + 0.5) * region_size,
            0.0,
            (region_pos.y as f32 + 0.5) * region_size,
        )
    }

    /// Calculate distance between two grid positions
    pub fn grid_distance(pos1: IVec2, pos2: IVec2) -> f32 {
        let diff = pos1 - pos2;
        ((diff.x.pow(2) + diff.y.pow(2)) as f32).sqrt()
    }

    /// Calculate Manhattan distance between two grid positions
    pub fn grid_manhattan_distance(pos1: IVec2, pos2: IVec2) -> i32 {
        let diff = pos1 - pos2;
        diff.x.abs() + diff.y.abs()
    }

    /// Check if a grid position is within a certain radius of another position
    pub fn is_within_grid_radius(pos: IVec2, center: IVec2, radius: f32) -> bool {
        Self::grid_distance(pos, center) <= radius
    }

    /// Get all grid positions within a radius of a center position
    pub fn get_grid_positions_in_radius(center: IVec2, radius: f32) -> Vec<IVec2> {
        let mut positions = Vec::new();
        let radius_int = radius.ceil() as i32;

        for x in -radius_int..=radius_int {
            for y in -radius_int..=radius_int {
                let pos = center + IVec2::new(x, y);
                if Self::is_within_grid_radius(pos, center, radius) {
                    positions.push(pos);
                }
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_world_conversion() {
        let grid_pos = IVec2::new(10, 20);
        let tile_size = 5.0;
        let world_pos = CoordinateConversion::grid_to_world(grid_pos, tile_size);

        assert_eq!(world_pos.x, 50.0);
        assert_eq!(world_pos.y, 100.0);
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let world_pos = Vec2::new(50.0, 100.0);
        let tile_size = 5.0;
        let grid_pos = CoordinateConversion::world_to_grid(world_pos, tile_size);

        assert_eq!(grid_pos.x, 10);
        assert_eq!(grid_pos.y, 20);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_grid = IVec2::new(15, 25);
        let tile_size = 3.0;

        let world_pos = CoordinateConversion::grid_to_world(original_grid, tile_size);
        let converted_grid = CoordinateConversion::world_to_grid(world_pos, tile_size);

        assert_eq!(original_grid, converted_grid);
    }

    #[test]
    fn test_grid_distance() {
        let pos1 = IVec2::new(0, 0);
        let pos2 = IVec2::new(3, 4);
        let distance = CoordinateConversion::grid_distance(pos1, pos2);

        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_grid_manhattan_distance() {
        let pos1 = IVec2::new(0, 0);
        let pos2 = IVec2::new(3, 4);
        let distance = CoordinateConversion::grid_manhattan_distance(pos1, pos2);

        assert_eq!(distance, 7); // 3 + 4
    }

    #[test]
    fn test_within_grid_radius() {
        let center = IVec2::new(0, 0);
        let pos_inside = IVec2::new(2, 2);
        let pos_outside = IVec2::new(5, 5);
        let radius = 3.0;

        assert!(CoordinateConversion::is_within_grid_radius(
            pos_inside, center, radius
        ));
        assert!(!CoordinateConversion::is_within_grid_radius(
            pos_outside,
            center,
            radius
        ));
    }

    #[test]
    fn test_sector_conversion() {
        let world_pos = Vec3::new(128.0, 0.0, 256.0);
        let sector_size = 64.0;
        let sector_pos = CoordinateConversion::world_to_sector(world_pos, sector_size);

        assert_eq!(sector_pos.x, 2);
        assert_eq!(sector_pos.y, 4);

        let world_center = CoordinateConversion::sector_to_world(sector_pos, sector_size);
        assert_eq!(world_center.x, 160.0); // (2 + 0.5) * 64
        assert_eq!(world_center.z, 288.0); // (4 + 0.5) * 64
    }
}
