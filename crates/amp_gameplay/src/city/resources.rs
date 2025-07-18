//! City resources for configuration and layout management

use crate::city::components::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// City generation configuration
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct CityConfig {
    /// Grid size for city generation
    pub grid_size: IVec2,
    /// Tile size in world units
    pub tile_size: f32,
    /// Seed for deterministic city generation
    pub seed: u64,
    /// Building density (0.0 to 1.0)
    pub building_density: f32,
    /// Street width in meters
    pub street_width: f32,
    /// Building height range
    pub building_height_range: (f32, f32),
    /// Building size range
    pub building_size_range: (f32, f32),
    /// Enable varied building heights like f430bc6
    pub varied_heights: bool,
    /// City center position for skyscrapers
    pub city_center: Vec2,
    /// City center radius for dense area
    pub city_center_radius: f32,
    /// Intersection frequency (every N tiles)
    pub intersection_frequency: u32,
}

/// City layout data loaded from configuration files
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct CityLayout {
    /// Grid layout data
    pub grid: HashMap<IVec2, TileType>,
    /// Building specifications
    pub buildings: HashMap<IVec2, BuildingSpec>,
    /// Street specifications
    pub streets: HashMap<IVec2, StreetSpec>,
    /// Intersection specifications
    pub intersections: HashMap<IVec2, IntersectionSpec>,
}

/// Building specification for city layout
#[derive(Debug, Clone, Reflect)]
pub struct BuildingSpec {
    pub building_type: BuildingType,
    pub height: f32,
    pub size: Vec2,
    pub rotation: f32,
    pub prefab_name: String,
}

/// Street specification for city layout
#[derive(Debug, Clone, Reflect)]
pub struct StreetSpec {
    pub street_type: StreetType,
    pub width: f32,
    pub direction: Vec3,
    pub has_sidewalks: bool,
    pub prefab_name: String,
}

/// Intersection specification for city layout
#[derive(Debug, Clone, Reflect)]
pub struct IntersectionSpec {
    pub intersection_type: IntersectionType,
    pub size: f32,
    pub has_traffic_lights: bool,
    pub prefab_name: String,
}

/// City prefab registry
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct CityPrefabs {
    /// Building prefab names mapped to PrefabIds
    pub buildings: HashMap<String, u32>,
    /// Street prefab names mapped to PrefabIds
    pub streets: HashMap<String, u32>,
    /// Intersection prefab names mapped to PrefabIds
    pub intersections: HashMap<String, u32>,
    /// Next available prefab ID
    pub next_id: u32,
}

impl Default for CityConfig {
    fn default() -> Self {
        Self {
            grid_size: IVec2::new(100, 100), // 100x100 grid for massive city
            tile_size: 20.0,                 // 20 meter tiles
            seed: 42,
            building_density: 0.6, // 60% of tiles have buildings
            street_width: 8.0,
            building_height_range: (10.0, 80.0), // 10m to 80m buildings
            building_size_range: (8.0, 20.0),    // 8m to 20m building base size
            varied_heights: true,                // Enable f430bc6-style varied heights
            city_center: Vec2::new(50.0, 50.0),  // Center of the grid
            city_center_radius: 20.0,            // Dense area around center
            intersection_frequency: 5,           // Intersection every 5 tiles
        }
    }
}

impl Default for CityLayout {
    fn default() -> Self {
        Self {
            grid: HashMap::new(),
            buildings: HashMap::new(),
            streets: HashMap::new(),
            intersections: HashMap::new(),
        }
    }
}

impl CityLayout {
    /// Get all buildings within a sector bounds for streaming
    pub fn get_buildings_in_sector(
        &self,
        sector_min: IVec2,
        sector_max: IVec2,
    ) -> Vec<(IVec2, &BuildingSpec)> {
        self.buildings
            .iter()
            .filter(|(pos, _)| {
                pos.x >= sector_min.x
                    && pos.x <= sector_max.x
                    && pos.y >= sector_min.y
                    && pos.y <= sector_max.y
            })
            .map(|(pos, spec)| (*pos, spec))
            .collect()
    }

    /// Get all streets within a sector bounds for streaming
    pub fn get_streets_in_sector(
        &self,
        sector_min: IVec2,
        sector_max: IVec2,
    ) -> Vec<(IVec2, &StreetSpec)> {
        self.streets
            .iter()
            .filter(|(pos, _)| {
                pos.x >= sector_min.x
                    && pos.x <= sector_max.x
                    && pos.y >= sector_min.y
                    && pos.y <= sector_max.y
            })
            .map(|(pos, spec)| (*pos, spec))
            .collect()
    }

    /// Get all intersections within a sector bounds for streaming
    pub fn get_intersections_in_sector(
        &self,
        sector_min: IVec2,
        sector_max: IVec2,
    ) -> Vec<(IVec2, &IntersectionSpec)> {
        self.intersections
            .iter()
            .filter(|(pos, _)| {
                pos.x >= sector_min.x
                    && pos.x <= sector_max.x
                    && pos.y >= sector_min.y
                    && pos.y <= sector_max.y
            })
            .map(|(pos, spec)| (*pos, spec))
            .collect()
    }
}

impl Default for CityPrefabs {
    fn default() -> Self {
        Self {
            buildings: HashMap::new(),
            streets: HashMap::new(),
            intersections: HashMap::new(),
            next_id: 5000, // Start from 5000 to avoid conflicts
        }
    }
}

impl CityConfig {
    /// Create a new city configuration
    pub fn new(grid_size: IVec2, tile_size: f32) -> Self {
        Self {
            grid_size,
            tile_size,
            ..Default::default()
        }
    }

    /// Get world position for grid coordinate
    pub fn grid_to_world(&self, grid_pos: IVec2) -> Vec3 {
        Vec3::new(
            grid_pos.x as f32 * self.tile_size,
            0.0,
            grid_pos.y as f32 * self.tile_size,
        )
    }

    /// Get grid coordinate for world position
    pub fn world_to_grid(&self, world_pos: Vec3) -> IVec2 {
        IVec2::new(
            (world_pos.x / self.tile_size).round() as i32,
            (world_pos.z / self.tile_size).round() as i32,
        )
    }

    /// Check if position is in city center for skyscrapers
    pub fn is_city_center(&self, grid_pos: IVec2) -> bool {
        let center_pos = Vec2::new(grid_pos.x as f32, grid_pos.y as f32);
        center_pos.distance(self.city_center) <= self.city_center_radius
    }

    /// Get building height based on distance from city center
    pub fn get_building_height(&self, grid_pos: IVec2, base_height: f32) -> f32 {
        if self.is_city_center(grid_pos) {
            // Skyscrapers in city center
            base_height * 2.0 + 30.0
        } else {
            // Regular buildings with slight variation
            let distance_factor = 1.0
                - (Vec2::new(grid_pos.x as f32, grid_pos.y as f32).distance(self.city_center)
                    / (self.grid_size.x as f32 * 0.5));
            base_height * (1.0 + distance_factor * 0.5)
        }
    }
}

impl CityPrefabs {
    /// Register a new prefab and return its ID
    pub fn register_prefab(&mut self, category: &str, name: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        match category {
            "building" => self.buildings.insert(name, id),
            "street" => self.streets.insert(name, id),
            "intersection" => self.intersections.insert(name, id),
            _ => panic!("Unknown prefab category: {}", category),
        };

        id
    }

    /// Get prefab ID by name and category
    pub fn get_prefab_id(&self, category: &str, name: &str) -> Option<u32> {
        match category {
            "building" => self.buildings.get(name).copied(),
            "street" => self.streets.get(name).copied(),
            "intersection" => self.intersections.get(name).copied(),
            _ => None,
        }
    }
}
