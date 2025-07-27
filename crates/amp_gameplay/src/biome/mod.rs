//! Biome generation system for hierarchical world generation
//!
//! This module implements biome-based world generation recipes for different environmental types,
//! supporting the hierarchical world streaming system.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub mod generation;
pub mod recipes;

pub use generation::*;
pub use recipes::*;

/// Enhanced biome types with specific generation parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Urban,
    Suburban,
    Rural,
    Industrial,
    Forest,
    Desert,
    Coastal,
    Mountain,
}

/// Biome generation parameters that control entity spawning and terrain features
#[derive(Debug, Clone)]
pub struct BiomeConfig {
    pub biome_type: BiomeType,
    pub building_density: f32,
    pub vegetation_density: f32,
    pub vehicle_density: f32,
    pub npc_density: f32,
    pub road_density: f32,
    pub water_probability: f32,
    pub terrain_roughness: f32,
    pub preferred_building_types: Vec<String>,
    pub preferred_vegetation_types: Vec<String>,
}

impl BiomeConfig {
    /// Get default configuration for a biome type
    pub fn for_biome(biome_type: BiomeType) -> Self {
        match biome_type {
            BiomeType::Urban => Self {
                biome_type,
                building_density: 0.8,
                vegetation_density: 0.2,
                vehicle_density: 0.6,
                npc_density: 0.7,
                road_density: 0.9,
                water_probability: 0.1,
                terrain_roughness: 0.2,
                preferred_building_types: vec![
                    "skyscraper".to_string(),
                    "office_building".to_string(),
                    "apartment".to_string(),
                    "shop".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "street_tree".to_string(),
                    "park_tree".to_string(),
                    "bush".to_string(),
                ],
            },
            BiomeType::Suburban => Self {
                biome_type,
                building_density: 0.4,
                vegetation_density: 0.6,
                vehicle_density: 0.4,
                npc_density: 0.3,
                road_density: 0.6,
                water_probability: 0.15,
                terrain_roughness: 0.3,
                preferred_building_types: vec![
                    "house".to_string(),
                    "townhouse".to_string(),
                    "small_shop".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "oak_tree".to_string(),
                    "maple_tree".to_string(),
                    "hedge".to_string(),
                    "lawn".to_string(),
                ],
            },
            BiomeType::Rural => Self {
                biome_type,
                building_density: 0.1,
                vegetation_density: 0.8,
                vehicle_density: 0.2,
                npc_density: 0.1,
                road_density: 0.3,
                water_probability: 0.25,
                terrain_roughness: 0.5,
                preferred_building_types: vec![
                    "farmhouse".to_string(),
                    "barn".to_string(),
                    "silo".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "crop_field".to_string(),
                    "wildflower".to_string(),
                    "tall_grass".to_string(),
                    "pine_tree".to_string(),
                ],
            },
            BiomeType::Industrial => Self {
                biome_type,
                building_density: 0.6,
                vegetation_density: 0.1,
                vehicle_density: 0.8,
                npc_density: 0.4,
                road_density: 0.8,
                water_probability: 0.05,
                terrain_roughness: 0.2,
                preferred_building_types: vec![
                    "factory".to_string(),
                    "warehouse".to_string(),
                    "power_plant".to_string(),
                    "refinery".to_string(),
                ],
                preferred_vegetation_types: vec!["weed".to_string(), "scrub_bush".to_string()],
            },
            BiomeType::Forest => Self {
                biome_type,
                building_density: 0.05,
                vegetation_density: 0.95,
                vehicle_density: 0.05,
                npc_density: 0.02,
                road_density: 0.1,
                water_probability: 0.3,
                terrain_roughness: 0.8,
                preferred_building_types: vec!["cabin".to_string(), "ranger_station".to_string()],
                preferred_vegetation_types: vec![
                    "pine_tree".to_string(),
                    "fir_tree".to_string(),
                    "deciduous_tree".to_string(),
                    "fern".to_string(),
                    "moss".to_string(),
                ],
            },
            BiomeType::Desert => Self {
                biome_type,
                building_density: 0.1,
                vegetation_density: 0.15,
                vehicle_density: 0.1,
                npc_density: 0.05,
                road_density: 0.2,
                water_probability: 0.02,
                terrain_roughness: 0.6,
                preferred_building_types: vec![
                    "adobe_house".to_string(),
                    "gas_station".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "cactus".to_string(),
                    "desert_shrub".to_string(),
                    "palm_tree".to_string(),
                ],
            },
            BiomeType::Coastal => Self {
                biome_type,
                building_density: 0.3,
                vegetation_density: 0.4,
                vehicle_density: 0.3,
                npc_density: 0.4,
                road_density: 0.5,
                water_probability: 0.6,
                terrain_roughness: 0.4,
                preferred_building_types: vec![
                    "beach_house".to_string(),
                    "pier".to_string(),
                    "lighthouse".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "palm_tree".to_string(),
                    "sea_grass".to_string(),
                    "dune_grass".to_string(),
                ],
            },
            BiomeType::Mountain => Self {
                biome_type,
                building_density: 0.05,
                vegetation_density: 0.6,
                vehicle_density: 0.1,
                npc_density: 0.02,
                road_density: 0.2,
                water_probability: 0.2,
                terrain_roughness: 0.9,
                preferred_building_types: vec![
                    "mountain_cabin".to_string(),
                    "ski_lodge".to_string(),
                ],
                preferred_vegetation_types: vec![
                    "alpine_tree".to_string(),
                    "mountain_shrub".to_string(),
                    "moss".to_string(),
                ],
            },
        }
    }
}

/// Biome detection resource for world generation
#[derive(Resource)]
pub struct BiomeDetector {
    /// Noise functions for biome generation
    pub temperature_noise: Box<dyn Fn(Vec3) -> f32 + Send + Sync>,
    pub humidity_noise: Box<dyn Fn(Vec3) -> f32 + Send + Sync>,
    pub elevation_noise: Box<dyn Fn(Vec3) -> f32 + Send + Sync>,
    pub urban_influence: Box<dyn Fn(Vec3) -> f32 + Send + Sync>,
}

impl Default for BiomeDetector {
    fn default() -> Self {
        Self {
            temperature_noise: Box::new(|pos| {
                // Simple Perlin-like noise for temperature
                let scale = 0.001;
                (pos.x * scale).sin() * (pos.z * scale).cos() * 0.5 + 0.5
            }),
            humidity_noise: Box::new(|pos| {
                // Simple noise for humidity
                let scale = 0.0015;
                (pos.x * scale * 1.3).cos() * (pos.z * scale * 0.7).sin() * 0.5 + 0.5
            }),
            elevation_noise: Box::new(|pos| {
                // Simple elevation noise
                let scale = 0.0005;
                (pos.x * scale * 2.1).sin() * (pos.z * scale * 1.7).cos() * 0.5 + 0.5
            }),
            urban_influence: Box::new(|pos| {
                // Distance from urban centers (simplified)
                let urban_centers = vec![Vec3::ZERO, Vec3::new(5000.0, 0.0, 5000.0)];
                let min_distance = urban_centers
                    .iter()
                    .map(|center| pos.distance(*center))
                    .fold(f32::INFINITY, f32::min);

                // Urban influence decreases with distance
                (1.0 - (min_distance / 10000.0).min(1.0)).max(0.0)
            }),
        }
    }
}

impl BiomeDetector {
    /// Detect biome type at a world position
    pub fn detect_biome(&self, position: Vec3) -> BiomeType {
        let temperature = (self.temperature_noise)(position);
        let humidity = (self.humidity_noise)(position);
        let elevation = (self.elevation_noise)(position);
        let urban_influence = (self.urban_influence)(position);

        // High urban influence = urban/suburban
        if urban_influence > 0.7 {
            return BiomeType::Urban;
        } else if urban_influence > 0.3 {
            return BiomeType::Suburban;
        }

        // Industrial areas near urban centers with low humidity
        if urban_influence > 0.1 && humidity < 0.3 && temperature > 0.4 {
            return BiomeType::Industrial;
        }

        // Coastal areas (high humidity, moderate elevation)
        if humidity > 0.7 && elevation < 0.3 {
            return BiomeType::Coastal;
        }

        // Mountain areas (high elevation)
        if elevation > 0.7 {
            return BiomeType::Mountain;
        }

        // Desert areas (low humidity, high temperature)
        if humidity < 0.2 && temperature > 0.6 {
            return BiomeType::Desert;
        }

        // Forest areas (high humidity, moderate temperature)
        if humidity > 0.6 && temperature > 0.3 && temperature < 0.7 {
            return BiomeType::Forest;
        }

        // Default to rural
        BiomeType::Rural
    }

    /// Get biome configuration for a position
    pub fn get_biome_config(&self, position: Vec3) -> BiomeConfig {
        let biome_type = self.detect_biome(position);
        BiomeConfig::for_biome(biome_type)
    }
}

/// Resource containing all biome configurations
#[derive(Resource)]
pub struct BiomeRegistry {
    pub configs: std::collections::HashMap<BiomeType, BiomeConfig>,
}

impl Default for BiomeRegistry {
    fn default() -> Self {
        let mut configs = std::collections::HashMap::new();

        // Initialize all biome configurations
        for biome_type in [
            BiomeType::Urban,
            BiomeType::Suburban,
            BiomeType::Rural,
            BiomeType::Industrial,
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Coastal,
            BiomeType::Mountain,
        ] {
            configs.insert(biome_type, BiomeConfig::for_biome(biome_type));
        }

        Self { configs }
    }
}

impl BiomeRegistry {
    /// Get configuration for a biome type
    pub fn get_config(&self, biome_type: BiomeType) -> Option<&BiomeConfig> {
        self.configs.get(&biome_type)
    }

    /// Update configuration for a biome type
    pub fn set_config(&mut self, biome_type: BiomeType, config: BiomeConfig) {
        self.configs.insert(biome_type, config);
    }
}

/// Component for entities that belong to a specific biome
#[derive(Component, Debug, Clone)]
pub struct BiomeEntity {
    pub biome_type: BiomeType,
    pub generation_seed: u64,
}

/// Plugin for biome generation system
#[derive(Default)]
pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeDetector>()
            .init_resource::<BiomeRegistry>();
    }
}

/// Helper function to create a seeded RNG for deterministic generation
pub fn create_biome_rng(position: Vec3, seed: u64) -> ChaCha8Rng {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    (position.x as i32, position.z as i32, seed).hash(&mut hasher);
    ChaCha8Rng::seed_from_u64(hasher.finish())
}

/// Enhanced biome detection that integrates with spawn budget system
pub fn enhanced_detect_biome_from_position(
    position: Vec3,
    detector: &BiomeDetector,
) -> crate::spawn_budget_policy::BiomeType {
    let biome_type = detector.detect_biome(position);

    // Map to spawn budget BiomeType
    match biome_type {
        BiomeType::Urban => crate::spawn_budget_policy::BiomeType::Urban,
        BiomeType::Suburban => crate::spawn_budget_policy::BiomeType::Suburban,
        BiomeType::Rural => crate::spawn_budget_policy::BiomeType::Rural,
        BiomeType::Industrial => crate::spawn_budget_policy::BiomeType::Industrial,
        // Map other biomes to closest spawn budget biome
        BiomeType::Forest => crate::spawn_budget_policy::BiomeType::Rural,
        BiomeType::Desert => crate::spawn_budget_policy::BiomeType::Rural,
        BiomeType::Coastal => crate::spawn_budget_policy::BiomeType::Suburban,
        BiomeType::Mountain => crate::spawn_budget_policy::BiomeType::Rural,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_config_creation() {
        let urban_config = BiomeConfig::for_biome(BiomeType::Urban);
        assert_eq!(urban_config.biome_type, BiomeType::Urban);
        assert!(urban_config.building_density > 0.5);
        assert!(urban_config.road_density > 0.5);
    }

    #[test]
    fn test_biome_detector() {
        let detector = BiomeDetector::default();

        // Test detection at origin
        let biome = detector.detect_biome(Vec3::ZERO);
        assert!(matches!(biome, BiomeType::Urban | BiomeType::Suburban));

        // Test far from urban centers
        let far_position = Vec3::new(50000.0, 0.0, 50000.0);
        let far_biome = detector.detect_biome(far_position);
        assert!(!matches!(far_biome, BiomeType::Urban));
    }

    #[test]
    fn test_biome_registry() {
        let registry = BiomeRegistry::default();

        // Should have all biome types
        assert!(registry.get_config(BiomeType::Urban).is_some());
        assert!(registry.get_config(BiomeType::Forest).is_some());
        assert!(registry.get_config(BiomeType::Desert).is_some());
    }

    #[test]
    fn test_seeded_rng() {
        let position = Vec3::new(100.0, 0.0, 200.0);
        let seed = 12345;

        let mut rng1 = create_biome_rng(position, seed);
        let mut rng2 = create_biome_rng(position, seed);

        // Same position and seed should produce same random numbers
        assert_eq!(rng1.gen::<f32>(), rng2.gen::<f32>());
    }

    #[test]
    fn test_enhanced_biome_detection() {
        let detector = BiomeDetector::default();
        let position = Vec3::ZERO;

        let budget_biome = enhanced_detect_biome_from_position(position, &detector);

        // Should return a valid spawn budget biome type
        assert!(matches!(
            budget_biome,
            crate::spawn_budget_policy::BiomeType::Urban
                | crate::spawn_budget_policy::BiomeType::Suburban
                | crate::spawn_budget_policy::BiomeType::Rural
                | crate::spawn_budget_policy::BiomeType::Industrial
        ));
    }
}
