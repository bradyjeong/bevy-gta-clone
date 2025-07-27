//! Biome generation recipes for different LOD levels and biome types
//!
//! This module defines specific generation patterns and rules for each biome type
//! at different levels of detail in the hierarchical world system.

use super::*;
use bevy::prelude::*;

/// Generation recipe for a specific biome and LOD level
#[derive(Debug, Clone)]
pub struct GenerationRecipe {
    pub biome_type: BiomeType,
    pub lod_level: amp_math::spatial::LODLevel,
    pub building_rules: BuildingGenerationRules,
    pub vegetation_rules: VegetationGenerationRules,
    pub vehicle_rules: VehicleGenerationRules,
    pub npc_rules: NpcGenerationRules,
    pub infrastructure_rules: InfrastructureGenerationRules,
}

/// Building generation rules for a biome/LOD combination
#[derive(Debug, Clone)]
pub struct BuildingGenerationRules {
    pub density_multiplier: f32,
    pub min_spacing: f32,
    pub max_buildings_per_chunk: usize,
    pub building_types: Vec<BuildingType>,
    pub height_variance: f32,
    pub clustering_factor: f32,
}

/// Vegetation generation rules
#[derive(Debug, Clone)]
pub struct VegetationGenerationRules {
    pub density_multiplier: f32,
    pub vegetation_types: Vec<VegetationType>,
    pub clustering_probability: f32,
    pub edge_preference: f32, // Preference for spawning near roads/buildings
}

/// Vehicle generation rules
#[derive(Debug, Clone)]
pub struct VehicleGenerationRules {
    pub density_multiplier: f32,
    pub vehicle_types: Vec<VehicleType>,
    pub road_preference: f32, // How much vehicles prefer to spawn near roads
    pub parking_probability: f32,
}

/// NPC generation rules
#[derive(Debug, Clone)]
pub struct NpcGenerationRules {
    pub density_multiplier: f32,
    pub npc_types: Vec<NpcType>,
    pub activity_patterns: Vec<ActivityPattern>,
    pub building_proximity_preference: f32,
}

/// Infrastructure generation rules (roads, utilities, etc.)
#[derive(Debug, Clone)]
pub struct InfrastructureGenerationRules {
    pub road_density: f32,
    pub utility_density: f32,
    pub connectivity_requirement: f32,
}

/// Building type definitions
#[derive(Debug, Clone)]
pub struct BuildingType {
    pub name: String,
    pub probability: f32,
    pub min_size: Vec3,
    pub max_size: Vec3,
    pub typical_height: f32,
}

/// Vegetation type definitions
#[derive(Debug, Clone)]
pub struct VegetationType {
    pub name: String,
    pub probability: f32,
    pub size_range: (f32, f32),
    pub clustering_size: usize,
}

/// Vehicle type definitions
#[derive(Debug, Clone)]
pub struct VehicleType {
    pub name: String,
    pub probability: f32,
    pub length: f32,
    pub width: f32,
}

/// NPC type definitions
#[derive(Debug, Clone)]
pub struct NpcType {
    pub name: String,
    pub probability: f32,
    pub activity_radius: f32,
}

/// Activity pattern for NPCs
#[derive(Debug, Clone)]
pub struct ActivityPattern {
    pub name: String,
    pub duration_range: (f32, f32),
    pub movement_speed: f32,
}

/// Recipe registry containing all generation recipes
#[derive(Resource)]
pub struct RecipeRegistry {
    pub recipes:
        std::collections::HashMap<(BiomeType, amp_math::spatial::LODLevel), GenerationRecipe>,
}

impl RecipeRegistry {
    /// Create a new recipe registry with default recipes
    pub fn new() -> Self {
        let mut recipes = std::collections::HashMap::new();

        // Generate recipes for all biome/LOD combinations
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
            for lod_level in [
                amp_math::spatial::LODLevel::Macro,
                amp_math::spatial::LODLevel::Region,
                amp_math::spatial::LODLevel::Local,
                amp_math::spatial::LODLevel::Detail,
                amp_math::spatial::LODLevel::Micro,
            ] {
                let recipe = Self::create_recipe(biome_type, lod_level);
                recipes.insert((biome_type, lod_level), recipe);
            }
        }

        Self { recipes }
    }

    /// Get a recipe for a specific biome and LOD level
    pub fn get_recipe(
        &self,
        biome_type: BiomeType,
        lod_level: amp_math::spatial::LODLevel,
    ) -> Option<&GenerationRecipe> {
        self.recipes.get(&(biome_type, lod_level))
    }

    /// Create a recipe for a biome/LOD combination
    fn create_recipe(
        biome_type: BiomeType,
        lod_level: amp_math::spatial::LODLevel,
    ) -> GenerationRecipe {
        let base_config = BiomeConfig::for_biome(biome_type);

        GenerationRecipe {
            biome_type,
            lod_level,
            building_rules: Self::create_building_rules(&base_config, lod_level),
            vegetation_rules: Self::create_vegetation_rules(&base_config, lod_level),
            vehicle_rules: Self::create_vehicle_rules(&base_config, lod_level),
            npc_rules: Self::create_npc_rules(&base_config, lod_level),
            infrastructure_rules: Self::create_infrastructure_rules(&base_config, lod_level),
        }
    }

    fn create_building_rules(
        config: &BiomeConfig,
        lod_level: amp_math::spatial::LODLevel,
    ) -> BuildingGenerationRules {
        let (density_mult, max_buildings, min_spacing) = match lod_level {
            amp_math::spatial::LODLevel::Macro => (0.0, 0, 1000.0), // No buildings at macro level
            amp_math::spatial::LODLevel::Region => (0.1, 2, 500.0), // Very few, large buildings
            amp_math::spatial::LODLevel::Local => (0.5, 8, 100.0),  // Moderate density
            amp_math::spatial::LODLevel::Detail => (0.8, 15, 50.0), // High density
            amp_math::spatial::LODLevel::Micro => (1.0, 25, 20.0),  // Full density
        };

        let building_types = match config.biome_type {
            BiomeType::Urban => vec![
                BuildingType {
                    name: "skyscraper".to_string(),
                    probability: 0.2,
                    min_size: Vec3::new(20.0, 80.0, 20.0),
                    max_size: Vec3::new(50.0, 200.0, 50.0),
                    typical_height: 120.0,
                },
                BuildingType {
                    name: "office_building".to_string(),
                    probability: 0.3,
                    min_size: Vec3::new(15.0, 30.0, 15.0),
                    max_size: Vec3::new(30.0, 80.0, 30.0),
                    typical_height: 50.0,
                },
                BuildingType {
                    name: "apartment".to_string(),
                    probability: 0.3,
                    min_size: Vec3::new(10.0, 20.0, 10.0),
                    max_size: Vec3::new(25.0, 60.0, 25.0),
                    typical_height: 35.0,
                },
                BuildingType {
                    name: "shop".to_string(),
                    probability: 0.2,
                    min_size: Vec3::new(8.0, 4.0, 8.0),
                    max_size: Vec3::new(15.0, 8.0, 15.0),
                    typical_height: 6.0,
                },
            ],
            BiomeType::Suburban => vec![
                BuildingType {
                    name: "house".to_string(),
                    probability: 0.6,
                    min_size: Vec3::new(8.0, 6.0, 8.0),
                    max_size: Vec3::new(15.0, 12.0, 15.0),
                    typical_height: 8.0,
                },
                BuildingType {
                    name: "townhouse".to_string(),
                    probability: 0.3,
                    min_size: Vec3::new(6.0, 8.0, 12.0),
                    max_size: Vec3::new(10.0, 15.0, 20.0),
                    typical_height: 10.0,
                },
                BuildingType {
                    name: "small_shop".to_string(),
                    probability: 0.1,
                    min_size: Vec3::new(10.0, 4.0, 10.0),
                    max_size: Vec3::new(20.0, 6.0, 20.0),
                    typical_height: 5.0,
                },
            ],
            BiomeType::Rural => vec![
                BuildingType {
                    name: "farmhouse".to_string(),
                    probability: 0.4,
                    min_size: Vec3::new(8.0, 6.0, 12.0),
                    max_size: Vec3::new(15.0, 10.0, 20.0),
                    typical_height: 8.0,
                },
                BuildingType {
                    name: "barn".to_string(),
                    probability: 0.4,
                    min_size: Vec3::new(15.0, 8.0, 20.0),
                    max_size: Vec3::new(30.0, 15.0, 40.0),
                    typical_height: 12.0,
                },
                BuildingType {
                    name: "silo".to_string(),
                    probability: 0.2,
                    min_size: Vec3::new(5.0, 15.0, 5.0),
                    max_size: Vec3::new(8.0, 25.0, 8.0),
                    typical_height: 20.0,
                },
            ],
            BiomeType::Industrial => vec![
                BuildingType {
                    name: "factory".to_string(),
                    probability: 0.4,
                    min_size: Vec3::new(30.0, 12.0, 50.0),
                    max_size: Vec3::new(80.0, 25.0, 120.0),
                    typical_height: 18.0,
                },
                BuildingType {
                    name: "warehouse".to_string(),
                    probability: 0.4,
                    min_size: Vec3::new(20.0, 8.0, 30.0),
                    max_size: Vec3::new(60.0, 15.0, 80.0),
                    typical_height: 12.0,
                },
                BuildingType {
                    name: "power_plant".to_string(),
                    probability: 0.1,
                    min_size: Vec3::new(40.0, 20.0, 60.0),
                    max_size: Vec3::new(100.0, 50.0, 150.0),
                    typical_height: 35.0,
                },
                BuildingType {
                    name: "refinery".to_string(),
                    probability: 0.1,
                    min_size: Vec3::new(50.0, 15.0, 80.0),
                    max_size: Vec3::new(120.0, 40.0, 200.0),
                    typical_height: 30.0,
                },
            ],
            _ => vec![BuildingType {
                name: "generic_building".to_string(),
                probability: 1.0,
                min_size: Vec3::new(8.0, 4.0, 8.0),
                max_size: Vec3::new(15.0, 8.0, 15.0),
                typical_height: 6.0,
            }],
        };

        BuildingGenerationRules {
            density_multiplier: config.building_density * density_mult,
            min_spacing,
            max_buildings_per_chunk: max_buildings,
            building_types,
            height_variance: 0.2,
            clustering_factor: match config.biome_type {
                BiomeType::Urban => 0.8,
                BiomeType::Suburban => 0.4,
                BiomeType::Rural => 0.1,
                BiomeType::Industrial => 0.6,
                _ => 0.3,
            },
        }
    }

    fn create_vegetation_rules(
        config: &BiomeConfig,
        lod_level: amp_math::spatial::LODLevel,
    ) -> VegetationGenerationRules {
        let density_mult = match lod_level {
            amp_math::spatial::LODLevel::Macro => 0.0,
            amp_math::spatial::LODLevel::Region => 0.2,
            amp_math::spatial::LODLevel::Local => 0.6,
            amp_math::spatial::LODLevel::Detail => 0.9,
            amp_math::spatial::LODLevel::Micro => 1.0,
        };

        let vegetation_types = match config.biome_type {
            BiomeType::Urban => vec![
                VegetationType {
                    name: "street_tree".to_string(),
                    probability: 0.4,
                    size_range: (8.0, 15.0),
                    clustering_size: 3,
                },
                VegetationType {
                    name: "bush".to_string(),
                    probability: 0.6,
                    size_range: (1.5, 3.0),
                    clustering_size: 5,
                },
            ],
            BiomeType::Forest => vec![
                VegetationType {
                    name: "pine_tree".to_string(),
                    probability: 0.4,
                    size_range: (15.0, 40.0),
                    clustering_size: 8,
                },
                VegetationType {
                    name: "deciduous_tree".to_string(),
                    probability: 0.3,
                    size_range: (12.0, 30.0),
                    clustering_size: 6,
                },
                VegetationType {
                    name: "fern".to_string(),
                    probability: 0.3,
                    size_range: (0.5, 2.0),
                    clustering_size: 12,
                },
            ],
            BiomeType::Desert => vec![
                VegetationType {
                    name: "cactus".to_string(),
                    probability: 0.6,
                    size_range: (2.0, 8.0),
                    clustering_size: 2,
                },
                VegetationType {
                    name: "desert_shrub".to_string(),
                    probability: 0.4,
                    size_range: (1.0, 3.0),
                    clustering_size: 4,
                },
            ],
            _ => vec![
                VegetationType {
                    name: "generic_tree".to_string(),
                    probability: 0.7,
                    size_range: (8.0, 20.0),
                    clustering_size: 4,
                },
                VegetationType {
                    name: "generic_bush".to_string(),
                    probability: 0.3,
                    size_range: (1.0, 3.0),
                    clustering_size: 6,
                },
            ],
        };

        VegetationGenerationRules {
            density_multiplier: config.vegetation_density * density_mult,
            vegetation_types,
            clustering_probability: match config.biome_type {
                BiomeType::Forest => 0.8,
                BiomeType::Rural => 0.6,
                BiomeType::Urban => 0.2,
                _ => 0.4,
            },
            edge_preference: match config.biome_type {
                BiomeType::Urban => 0.8,
                BiomeType::Suburban => 0.6,
                _ => 0.2,
            },
        }
    }

    fn create_vehicle_rules(
        config: &BiomeConfig,
        lod_level: amp_math::spatial::LODLevel,
    ) -> VehicleGenerationRules {
        let density_mult = match lod_level {
            amp_math::spatial::LODLevel::Macro => 0.0,
            amp_math::spatial::LODLevel::Region => 0.0,
            amp_math::spatial::LODLevel::Local => 0.3,
            amp_math::spatial::LODLevel::Detail => 0.7,
            amp_math::spatial::LODLevel::Micro => 1.0,
        };

        let vehicle_types = match config.biome_type {
            BiomeType::Urban => vec![
                VehicleType {
                    name: "sedan".to_string(),
                    probability: 0.4,
                    length: 4.5,
                    width: 1.8,
                },
                VehicleType {
                    name: "suv".to_string(),
                    probability: 0.3,
                    length: 5.0,
                    width: 2.0,
                },
                VehicleType {
                    name: "taxi".to_string(),
                    probability: 0.2,
                    length: 4.5,
                    width: 1.8,
                },
                VehicleType {
                    name: "bus".to_string(),
                    probability: 0.1,
                    length: 12.0,
                    width: 2.5,
                },
            ],
            BiomeType::Rural => vec![
                VehicleType {
                    name: "pickup_truck".to_string(),
                    probability: 0.5,
                    length: 5.5,
                    width: 2.0,
                },
                VehicleType {
                    name: "tractor".to_string(),
                    probability: 0.3,
                    length: 6.0,
                    width: 2.5,
                },
                VehicleType {
                    name: "suv".to_string(),
                    probability: 0.2,
                    length: 5.0,
                    width: 2.0,
                },
            ],
            BiomeType::Industrial => vec![
                VehicleType {
                    name: "truck".to_string(),
                    probability: 0.4,
                    length: 8.0,
                    width: 2.5,
                },
                VehicleType {
                    name: "van".to_string(),
                    probability: 0.3,
                    length: 6.0,
                    width: 2.0,
                },
                VehicleType {
                    name: "forklift".to_string(),
                    probability: 0.2,
                    length: 3.0,
                    width: 1.5,
                },
                VehicleType {
                    name: "semi_truck".to_string(),
                    probability: 0.1,
                    length: 16.0,
                    width: 2.5,
                },
            ],
            _ => vec![VehicleType {
                name: "sedan".to_string(),
                probability: 1.0,
                length: 4.5,
                width: 1.8,
            }],
        };

        VehicleGenerationRules {
            density_multiplier: config.vehicle_density * density_mult,
            vehicle_types,
            road_preference: match config.biome_type {
                BiomeType::Urban => 0.9,
                BiomeType::Suburban => 0.8,
                BiomeType::Industrial => 0.7,
                _ => 0.5,
            },
            parking_probability: match config.biome_type {
                BiomeType::Urban => 0.8,
                BiomeType::Suburban => 0.9,
                BiomeType::Industrial => 0.6,
                _ => 0.4,
            },
        }
    }

    fn create_npc_rules(
        config: &BiomeConfig,
        lod_level: amp_math::spatial::LODLevel,
    ) -> NpcGenerationRules {
        let density_mult = match lod_level {
            amp_math::spatial::LODLevel::Macro => 0.0,
            amp_math::spatial::LODLevel::Region => 0.0,
            amp_math::spatial::LODLevel::Local => 0.0,
            amp_math::spatial::LODLevel::Detail => 0.5,
            amp_math::spatial::LODLevel::Micro => 1.0,
        };

        let npc_types = match config.biome_type {
            BiomeType::Urban => vec![
                NpcType {
                    name: "pedestrian".to_string(),
                    probability: 0.6,
                    activity_radius: 50.0,
                },
                NpcType {
                    name: "shopper".to_string(),
                    probability: 0.2,
                    activity_radius: 100.0,
                },
                NpcType {
                    name: "worker".to_string(),
                    probability: 0.2,
                    activity_radius: 200.0,
                },
            ],
            BiomeType::Rural => vec![
                NpcType {
                    name: "farmer".to_string(),
                    probability: 0.7,
                    activity_radius: 500.0,
                },
                NpcType {
                    name: "resident".to_string(),
                    probability: 0.3,
                    activity_radius: 100.0,
                },
            ],
            _ => vec![NpcType {
                name: "generic_npc".to_string(),
                probability: 1.0,
                activity_radius: 100.0,
            }],
        };

        let activity_patterns = vec![
            ActivityPattern {
                name: "walking".to_string(),
                duration_range: (30.0, 300.0),
                movement_speed: 1.4,
            },
            ActivityPattern {
                name: "standing".to_string(),
                duration_range: (10.0, 60.0),
                movement_speed: 0.0,
            },
            ActivityPattern {
                name: "sitting".to_string(),
                duration_range: (60.0, 600.0),
                movement_speed: 0.0,
            },
        ];

        NpcGenerationRules {
            density_multiplier: config.npc_density * density_mult,
            npc_types,
            activity_patterns,
            building_proximity_preference: match config.biome_type {
                BiomeType::Urban => 0.8,
                BiomeType::Suburban => 0.6,
                BiomeType::Industrial => 0.4,
                _ => 0.3,
            },
        }
    }

    fn create_infrastructure_rules(
        config: &BiomeConfig,
        lod_level: amp_math::spatial::LODLevel,
    ) -> InfrastructureGenerationRules {
        let road_mult = match lod_level {
            amp_math::spatial::LODLevel::Macro => 0.0,
            amp_math::spatial::LODLevel::Region => 0.3,
            amp_math::spatial::LODLevel::Local => 0.7,
            amp_math::spatial::LODLevel::Detail => 1.0,
            amp_math::spatial::LODLevel::Micro => 1.0,
        };

        InfrastructureGenerationRules {
            road_density: config.road_density * road_mult,
            utility_density: match config.biome_type {
                BiomeType::Urban => 0.9,
                BiomeType::Suburban => 0.7,
                BiomeType::Industrial => 0.8,
                BiomeType::Rural => 0.3,
                _ => 0.2,
            },
            connectivity_requirement: match config.biome_type {
                BiomeType::Urban => 0.9,
                BiomeType::Suburban => 0.7,
                BiomeType::Industrial => 0.8,
                _ => 0.4,
            },
        }
    }
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_registry_creation() {
        let registry = RecipeRegistry::new();

        // Should have recipes for all biome/LOD combinations
        assert!(registry
            .get_recipe(BiomeType::Urban, amp_math::spatial::LODLevel::Local)
            .is_some());
        assert!(registry
            .get_recipe(BiomeType::Forest, amp_math::spatial::LODLevel::Detail)
            .is_some());
        assert!(registry
            .get_recipe(BiomeType::Desert, amp_math::spatial::LODLevel::Region)
            .is_some());
    }

    #[test]
    fn test_urban_building_rules() {
        let registry = RecipeRegistry::new();
        let recipe = registry
            .get_recipe(BiomeType::Urban, amp_math::spatial::LODLevel::Local)
            .unwrap();

        assert!(recipe.building_rules.density_multiplier > 0.0);
        assert!(!recipe.building_rules.building_types.is_empty());
    }

    #[test]
    fn test_forest_vegetation_rules() {
        let registry = RecipeRegistry::new();
        let recipe = registry
            .get_recipe(BiomeType::Forest, amp_math::spatial::LODLevel::Detail)
            .unwrap();

        assert!(recipe.vegetation_rules.density_multiplier > 0.5);
        assert!(recipe.vegetation_rules.clustering_probability > 0.5);
    }

    #[test]
    fn test_lod_scaling() {
        let registry = RecipeRegistry::new();

        let macro_recipe = registry
            .get_recipe(BiomeType::Urban, amp_math::spatial::LODLevel::Macro)
            .unwrap();
        let micro_recipe = registry
            .get_recipe(BiomeType::Urban, amp_math::spatial::LODLevel::Micro)
            .unwrap();

        // Micro level should have higher density than macro
        assert!(
            micro_recipe.building_rules.density_multiplier
                > macro_recipe.building_rules.density_multiplier
        );
        assert!(
            micro_recipe.vegetation_rules.density_multiplier
                > macro_recipe.vegetation_rules.density_multiplier
        );
    }

    #[test]
    fn test_biome_specific_content() {
        let registry = RecipeRegistry::new();

        let urban_recipe = registry
            .get_recipe(BiomeType::Urban, amp_math::spatial::LODLevel::Local)
            .unwrap();
        let rural_recipe = registry
            .get_recipe(BiomeType::Rural, amp_math::spatial::LODLevel::Local)
            .unwrap();

        // Urban should have higher building density
        assert!(
            urban_recipe.building_rules.density_multiplier
                > rural_recipe.building_rules.density_multiplier
        );

        // Rural should have higher vegetation density
        assert!(
            rural_recipe.vegetation_rules.density_multiplier
                > urban_recipe.vegetation_rules.density_multiplier
        );
    }
}
