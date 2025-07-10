//! Hierarchical LOD clipmap implementation
//!
//! This module provides constants and structures for hierarchical level-of-detail
//! management using clipmaps, enabling efficient rendering of large-scale open worlds.

use crate::spatial::region::RegionId;
use glam::Vec2;

/// Maximum number of LOD levels supported by the clipmap
pub const MAX_LOD_LEVELS: u8 = 8;

/// Base size for the finest LOD level (in world units)
pub const BASE_LOD_SIZE: f32 = 100.0;

/// Size multiplier between LOD levels
pub const LOD_SIZE_MULTIPLIER: f32 = 2.0;

/// Number of rings in each clipmap level
pub const CLIPMAP_RINGS: u8 = 4;

/// Size of each clipmap ring (in regions)
pub const RING_SIZE: u32 = 16;

/// Distance at which to trigger LOD transitions
pub const LOD_TRANSITION_DISTANCE: f32 = 0.7;

/// Hysteresis factor for LOD transitions to prevent flickering
pub const LOD_HYSTERESIS: f32 = 0.1;

/// Configuration for clipmap LOD system
#[derive(Debug, Clone)]
pub struct ClipmapConfig {
    /// Maximum number of LOD levels
    pub max_levels: u8,
    /// Base size for the finest LOD level
    pub base_size: f32,
    /// Size multiplier between levels
    pub size_multiplier: f32,
    /// Number of rings per level
    pub rings: u8,
    /// Size of each ring
    pub ring_size: u32,
    /// Distance threshold for LOD transitions
    pub transition_distance: f32,
    /// Hysteresis factor for smooth transitions
    pub hysteresis: f32,
}

impl Default for ClipmapConfig {
    fn default() -> Self {
        Self {
            max_levels: MAX_LOD_LEVELS,
            base_size: BASE_LOD_SIZE,
            size_multiplier: LOD_SIZE_MULTIPLIER,
            rings: CLIPMAP_RINGS,
            ring_size: RING_SIZE,
            transition_distance: LOD_TRANSITION_DISTANCE,
            hysteresis: LOD_HYSTERESIS,
        }
    }
}

/// Hierarchical clipmap for managing LOD regions
///
/// This structure manages a hierarchical set of clipmaps, each representing
/// a different level of detail. It provides efficient spatial queries and
/// LOD management for large-scale open world rendering.
#[derive(Debug)]
pub struct HierarchicalClipmap {
    /// Configuration for the clipmap system
    config: ClipmapConfig,
    /// Current center position of the clipmap
    center: Vec2,
    /// Active regions at each LOD level
    active_regions: Vec<Vec<RegionId>>,
    /// Previous center position for change detection
    prev_center: Vec2,
}

impl HierarchicalClipmap {
    /// Create a new hierarchical clipmap
    ///
    /// # Arguments
    /// * `config` - Configuration for the clipmap system
    /// * `center` - Initial center position
    pub fn new(config: ClipmapConfig, center: Vec2) -> Self {
        let mut active_regions = Vec::with_capacity(config.max_levels as usize);
        for _ in 0..config.max_levels {
            active_regions.push(Vec::new());
        }

        let mut clipmap = Self {
            config,
            center,
            active_regions,
            prev_center: center,
        };

        // Initialize the active regions
        clipmap.update_active_regions();

        clipmap
    }

    /// Create a new hierarchical clipmap with default configuration
    ///
    /// # Arguments
    /// * `center` - Initial center position
    pub fn new_default(center: Vec2) -> Self {
        Self::new(ClipmapConfig::default(), center)
    }

    /// Update the clipmap center position
    ///
    /// This method should be called when the player or camera moves to update
    /// the active regions based on the new position.
    ///
    /// # Arguments
    /// * `new_center` - The new center position
    ///
    /// # Returns
    /// * `true` if the clipmap was updated, `false` if no update was needed
    pub fn update_center(&mut self, new_center: Vec2) -> bool {
        let distance = (new_center - self.center).length();
        let threshold = self.config.base_size * self.config.transition_distance;

        if distance > threshold {
            self.prev_center = self.center;
            self.center = new_center;
            self.update_active_regions();
            true
        } else {
            false
        }
    }

    /// Get the current center position
    pub fn center(&self) -> Vec2 {
        self.center
    }

    /// Get the configuration
    pub fn config(&self) -> &ClipmapConfig {
        &self.config
    }

    /// Get the active regions for a specific LOD level
    ///
    /// # Arguments
    /// * `level` - The LOD level (0 = finest detail)
    ///
    /// # Returns
    /// * A slice of active region IDs for the specified level
    pub fn get_active_regions(&self, level: u8) -> &[RegionId] {
        if level < self.config.max_levels {
            &self.active_regions[level as usize]
        } else {
            &[]
        }
    }

    /// Get all active regions across all LOD levels
    ///
    /// # Returns
    /// * A vector of tuples containing (level, region_id) for all active regions
    pub fn get_all_active_regions(&self) -> Vec<(u8, RegionId)> {
        let mut all_regions = Vec::new();
        for (level, regions) in self.active_regions.iter().enumerate() {
            for &region_id in regions {
                all_regions.push((level as u8, region_id));
            }
        }
        all_regions
    }

    /// Calculate the appropriate LOD level for a given distance
    ///
    /// # Arguments
    /// * `distance` - Distance from the center
    ///
    /// # Returns
    /// * The appropriate LOD level (0 = finest detail)
    pub fn calculate_lod_level(&self, distance: f32) -> u8 {
        if distance <= self.config.base_size {
            return 0;
        }

        let normalized_distance = distance / self.config.base_size;
        let level = (normalized_distance.log2() / self.config.size_multiplier.log2()).floor() as u8;
        level.min(self.config.max_levels - 1)
    }

    /// Get the size of regions at a specific LOD level
    ///
    /// # Arguments
    /// * `level` - The LOD level
    ///
    /// # Returns
    /// * The size of regions at the specified level
    pub fn get_level_size(&self, level: u8) -> f32 {
        self.config.base_size * self.config.size_multiplier.powi(level as i32)
    }

    /// Check if a region should be loaded at a specific LOD level
    ///
    /// # Arguments
    /// * `region_id` - The region to check
    /// * `level` - The LOD level
    ///
    /// # Returns
    /// * `true` if the region should be loaded, `false` otherwise
    pub fn should_load_region(&self, region_id: RegionId, level: u8) -> bool {
        if level >= self.config.max_levels {
            return false;
        }

        // TODO: Implement proper region loading logic based on distance and priority
        // For now, just check if the region is in the active list
        self.active_regions[level as usize].contains(&region_id)
    }

    /// Update the active regions based on the current center position
    fn update_active_regions(&mut self) {
        // TODO: Implement the actual clipmap region calculation
        // This is a stub implementation that will be expanded in future iterations

        for level in 0..self.config.max_levels {
            self.active_regions[level as usize].clear();

            // Calculate the size for this level
            let level_size = self.get_level_size(level);
            let region_size = level_size / self.config.ring_size as f32;

            // Calculate the grid coordinates around the center
            let center_x = (self.center.x / region_size).floor() as i32;
            let center_y = (self.center.y / region_size).floor() as i32;

            // Add regions in a square pattern around the center
            let radius = (self.config.rings as u32 * self.config.ring_size / 2) as i32;
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x >= 0 && y >= 0 {
                        let region_id = RegionId::from_coords(x as u32, y as u32);
                        self.active_regions[level as usize].push(region_id);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipmap_config_default() {
        let config = ClipmapConfig::default();
        assert_eq!(config.max_levels, MAX_LOD_LEVELS);
        assert_eq!(config.base_size, BASE_LOD_SIZE);
        assert_eq!(config.size_multiplier, LOD_SIZE_MULTIPLIER);
    }

    #[test]
    fn test_hierarchical_clipmap_creation() {
        let center = Vec2::new(1000.0, 2000.0);
        let clipmap = HierarchicalClipmap::new_default(center);

        assert_eq!(clipmap.center(), center);
        assert_eq!(clipmap.config().max_levels, MAX_LOD_LEVELS);
    }

    #[test]
    fn test_clipmap_update_center() {
        let mut clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);

        // Small movement shouldn't trigger update
        let small_movement = Vec2::new(10.0, 10.0);
        assert!(!clipmap.update_center(small_movement));
        assert_eq!(clipmap.center(), Vec2::ZERO);

        // Large movement should trigger update
        let large_movement = Vec2::new(1000.0, 1000.0);
        assert!(clipmap.update_center(large_movement));
        assert_eq!(clipmap.center(), large_movement);
    }

    #[test]
    fn test_lod_level_calculation() {
        let clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);

        // Test various distances
        assert_eq!(clipmap.calculate_lod_level(50.0), 0); // Close = finest detail
        assert_eq!(clipmap.calculate_lod_level(150.0), 0); // Still level 0 (distance <= base_size * multiplier)
        assert_eq!(clipmap.calculate_lod_level(250.0), 1); // Medium distance
        assert_eq!(clipmap.calculate_lod_level(500.0), 2); // Far distance

        // Test maximum level clamping
        assert_eq!(clipmap.calculate_lod_level(100000.0), MAX_LOD_LEVELS - 1);
    }

    #[test]
    fn test_level_size_calculation() {
        let clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);

        assert_eq!(clipmap.get_level_size(0), BASE_LOD_SIZE);
        assert_eq!(
            clipmap.get_level_size(1),
            BASE_LOD_SIZE * LOD_SIZE_MULTIPLIER
        );
        assert_eq!(
            clipmap.get_level_size(2),
            BASE_LOD_SIZE * LOD_SIZE_MULTIPLIER * LOD_SIZE_MULTIPLIER
        );
    }

    #[test]
    fn test_active_regions_initialization() {
        let clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);

        // All levels should have some active regions after initialization
        for level in 0..MAX_LOD_LEVELS {
            let regions = clipmap.get_active_regions(level);
            // Active regions should be populated after update
            if level < MAX_LOD_LEVELS {
                assert!(!regions.is_empty());
            }
        }
    }

    #[test]
    fn test_get_all_active_regions() {
        let clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);
        let all_regions = clipmap.get_all_active_regions();

        // Should have regions from multiple levels
        assert!(!all_regions.is_empty());

        // Check that we have regions from level 0
        let level_0_regions: Vec<_> = all_regions
            .iter()
            .filter(|(level, _)| *level == 0)
            .collect();
        assert!(!level_0_regions.is_empty());
    }

    #[test]
    fn test_should_load_region() {
        let clipmap = HierarchicalClipmap::new_default(Vec2::ZERO);

        // Invalid level should return false
        assert!(!clipmap.should_load_region(RegionId::from_coords(0, 0), 255));

        // Valid levels should work (though this depends on implementation)
        let _result = clipmap.should_load_region(RegionId::from_coords(0, 0), 0);
        // Result depends on whether this region is in the active list
        // We just test that the function call succeeds without panicking
    }

    #[test]
    fn test_custom_clipmap_config() {
        let config = ClipmapConfig {
            max_levels: 4,
            base_size: 200.0,
            size_multiplier: 3.0,
            rings: 2,
            ring_size: 8,
            transition_distance: 0.5,
            hysteresis: 0.05,
        };

        let clipmap = HierarchicalClipmap::new(config, Vec2::ZERO);
        assert_eq!(clipmap.config().max_levels, 4);
        assert_eq!(clipmap.config().base_size, 200.0);
        assert_eq!(clipmap.get_level_size(0), 200.0);
        assert_eq!(clipmap.get_level_size(1), 600.0); // 200 * 3
    }
}
