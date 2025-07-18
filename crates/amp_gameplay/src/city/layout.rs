//! City layout generation and loading systems

use crate::city::components::*;
use crate::city::resources::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// City layout generator for procedural city creation
pub struct CityLayoutGenerator {
    pub config: CityConfig,
    pub rng_seed: u64,
}

impl CityLayoutGenerator {
    /// Create a new city layout generator
    pub fn new(config: CityConfig) -> Self {
        Self {
            rng_seed: config.seed,
            config,
        }
    }

    /// Generate a complete city layout based on configuration
    pub fn generate_layout(&self) -> CityLayout {
        let mut layout = CityLayout::default();

        // Generate street grid first
        self.generate_street_grid(&mut layout);

        // Generate intersections
        self.generate_intersections(&mut layout);

        // Generate buildings in remaining spaces
        self.generate_buildings(&mut layout);

        // Add parks and green spaces
        self.generate_parks(&mut layout);

        layout
    }

    /// Generate street grid with main roads and side streets
    fn generate_street_grid(&self, layout: &mut CityLayout) {
        let grid_size = self.config.grid_size;

        // Generate main roads (every N tiles)
        for x in (0..grid_size.x).step_by(self.config.intersection_frequency as usize) {
            for z in 0..grid_size.y {
                let pos = IVec2::new(x, z);
                layout.grid.insert(pos, TileType::Street);
                layout.streets.insert(
                    pos,
                    StreetSpec {
                        street_type: StreetType::Main,
                        width: self.config.street_width * 1.5,
                        direction: Vec3::Z,
                        has_sidewalks: true,
                        prefab_name: "street_main".to_string(),
                    },
                );
            }
        }

        // Generate perpendicular main roads
        for z in (0..grid_size.y).step_by(self.config.intersection_frequency as usize) {
            for x in 0..grid_size.x {
                let pos = IVec2::new(x, z);
                layout.grid.insert(pos, TileType::Street);
                layout.streets.insert(
                    pos,
                    StreetSpec {
                        street_type: StreetType::Main,
                        width: self.config.street_width * 1.5,
                        direction: Vec3::X,
                        has_sidewalks: true,
                        prefab_name: "street_main".to_string(),
                    },
                );
            }
        }

        // Generate side streets for accessibility
        for x in (2..grid_size.x).step_by(3) {
            for z in 0..grid_size.y {
                let pos = IVec2::new(x, z);
                if !layout.grid.contains_key(&pos) {
                    layout.grid.insert(pos, TileType::Street);
                    layout.streets.insert(
                        pos,
                        StreetSpec {
                            street_type: StreetType::Side,
                            width: self.config.street_width,
                            direction: Vec3::Z,
                            has_sidewalks: true,
                            prefab_name: "street_side".to_string(),
                        },
                    );
                }
            }
        }
    }

    /// Generate intersections at street crossings
    fn generate_intersections(&self, layout: &mut CityLayout) {
        let grid_size = self.config.grid_size;

        for x in (0..grid_size.x).step_by(self.config.intersection_frequency as usize) {
            for z in (0..grid_size.y).step_by(self.config.intersection_frequency as usize) {
                let pos = IVec2::new(x, z);
                layout.grid.insert(pos, TileType::Intersection);
                layout.intersections.insert(
                    pos,
                    IntersectionSpec {
                        intersection_type: IntersectionType::FourWay,
                        size: self.config.street_width * 2.0,
                        has_traffic_lights: self.config.is_city_center(pos),
                        prefab_name: "intersection_fourway".to_string(),
                    },
                );
            }
        }
    }

    /// Generate buildings using density and type distribution
    fn generate_buildings(&self, layout: &mut CityLayout) {
        let grid_size = self.config.grid_size;

        // Simple LCG for deterministic randomness
        let mut rng_state = self.rng_seed;
        let mut next_random = || -> u32 {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            ((rng_state / 65536) % 32768) as u32
        };

        for x in 0..grid_size.x {
            for z in 0..grid_size.y {
                let pos = IVec2::new(x, z);

                // Skip if tile is already occupied
                if layout.grid.contains_key(&pos) {
                    continue;
                }

                // Check building density
                let density_roll = (next_random() as f32) / 32768.0;
                if density_roll > self.config.building_density {
                    continue;
                }

                // Determine building type based on position
                let building_type = self.determine_building_type(pos, &mut next_random);

                // Generate building height with variation
                let base_height = self.config.building_height_range.0
                    + (next_random() as f32 / 32768.0)
                        * (self.config.building_height_range.1
                            - self.config.building_height_range.0);

                let height = if self.config.varied_heights {
                    self.config.get_building_height(pos, base_height)
                } else {
                    base_height
                };

                // Generate building size
                let size_factor = (next_random() as f32) / 32768.0;
                let size = self.config.building_size_range.0
                    + size_factor
                        * (self.config.building_size_range.1 - self.config.building_size_range.0);

                layout.grid.insert(pos, TileType::Building);
                layout.buildings.insert(
                    pos,
                    BuildingSpec {
                        building_type,
                        height,
                        size: Vec2::new(size, size),
                        rotation: 0.0,
                        prefab_name: self.get_building_prefab_name(building_type),
                    },
                );
            }
        }
    }

    /// Determine building type based on position in city
    fn determine_building_type(&self, pos: IVec2, rng: &mut dyn FnMut() -> u32) -> BuildingType {
        if self.config.is_city_center(pos) {
            // City center has more commercial and skyscrapers
            match rng() % 100 {
                0..=40 => BuildingType::Commercial,
                41..=60 => BuildingType::Skyscraper,
                61..=80 => BuildingType::Residential,
                81..=90 => BuildingType::Shop,
                _ => BuildingType::Infrastructure,
            }
        } else {
            // Outer areas have more residential and industrial
            match rng() % 100 {
                0..=50 => BuildingType::Residential,
                51..=70 => BuildingType::Industrial,
                71..=85 => BuildingType::Commercial,
                86..=95 => BuildingType::Shop,
                _ => BuildingType::Infrastructure,
            }
        }
    }

    /// Get prefab name for building type
    fn get_building_prefab_name(&self, building_type: BuildingType) -> String {
        match building_type {
            BuildingType::Residential => "building_residential".to_string(),
            BuildingType::Commercial => "building_commercial".to_string(),
            BuildingType::Industrial => "building_industrial".to_string(),
            BuildingType::Infrastructure => "building_infrastructure".to_string(),
            BuildingType::Skyscraper => "building_skyscraper".to_string(),
            BuildingType::Shop => "building_shop".to_string(),
        }
    }

    /// Generate parks and green spaces for urban planning
    fn generate_parks(&self, layout: &mut CityLayout) {
        let grid_size = self.config.grid_size;

        // Add parks in open areas (every 20x20 block)
        for x in (10..grid_size.x).step_by(20) {
            for z in (10..grid_size.y).step_by(20) {
                let park_size = 3; // 3x3 park
                for px in 0..park_size {
                    for pz in 0..park_size {
                        let pos = IVec2::new(x + px, z + pz);
                        if !layout.grid.contains_key(&pos) {
                            layout.grid.insert(pos, TileType::Park);
                        }
                    }
                }
            }
        }
    }
}

/// City layout serialization for CSV/JSON export
pub struct CityLayoutSerializer;

impl CityLayoutSerializer {
    /// Export city layout to CSV format
    pub fn export_to_csv(layout: &CityLayout, grid_size: IVec2) -> String {
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("x,z,tile_type,prefab_name,height,size_x,size_z\n");

        // Export grid data
        for z in 0..grid_size.y {
            for x in 0..grid_size.x {
                let pos = IVec2::new(x, z);
                let tile_type = layout.grid.get(&pos).unwrap_or(&TileType::Empty);

                let (prefab_name, height, size_x, size_z) = match tile_type {
                    TileType::Building => {
                        if let Some(building) = layout.buildings.get(&pos) {
                            (
                                building.prefab_name.clone(),
                                building.height,
                                building.size.x,
                                building.size.y,
                            )
                        } else {
                            ("".to_string(), 0.0, 0.0, 0.0)
                        }
                    }
                    TileType::Street => {
                        if let Some(street) = layout.streets.get(&pos) {
                            (street.prefab_name.clone(), 0.0, street.width, street.width)
                        } else {
                            ("".to_string(), 0.0, 0.0, 0.0)
                        }
                    }
                    TileType::Intersection => {
                        if let Some(intersection) = layout.intersections.get(&pos) {
                            (
                                intersection.prefab_name.clone(),
                                0.0,
                                intersection.size,
                                intersection.size,
                            )
                        } else {
                            ("".to_string(), 0.0, 0.0, 0.0)
                        }
                    }
                    _ => ("".to_string(), 0.0, 0.0, 0.0),
                };

                csv_content.push_str(&format!(
                    "{},{},{:?},{},{},{},{}\n",
                    x, z, tile_type, prefab_name, height, size_x, size_z
                ));
            }
        }

        csv_content
    }

    /// Import city layout from CSV format
    pub fn import_from_csv(csv_content: &str) -> Result<CityLayout, String> {
        let mut layout = CityLayout::default();

        for (line_num, line) in csv_content.lines().enumerate() {
            if line_num == 0 {
                continue; // Skip header
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 7 {
                return Err(format!("Invalid CSV line {}: {}", line_num + 1, line));
            }

            let x = parts[0]
                .parse::<i32>()
                .map_err(|_| format!("Invalid x coordinate on line {}", line_num + 1))?;
            let z = parts[1]
                .parse::<i32>()
                .map_err(|_| format!("Invalid z coordinate on line {}", line_num + 1))?;
            let pos = IVec2::new(x, z);

            // Parse tile type
            let tile_type = match parts[2] {
                "Building" => TileType::Building,
                "Street" => TileType::Street,
                "Intersection" => TileType::Intersection,
                "Park" => TileType::Park,
                "Water" => TileType::Water,
                _ => TileType::Empty,
            };

            if tile_type != TileType::Empty {
                layout.grid.insert(pos, tile_type);
            }

            // Parse specific data based on tile type
            match tile_type {
                TileType::Building => {
                    let height = parts[4].parse::<f32>().unwrap_or(10.0);
                    let size_x = parts[5].parse::<f32>().unwrap_or(10.0);
                    let size_z = parts[6].parse::<f32>().unwrap_or(10.0);

                    layout.buildings.insert(
                        pos,
                        BuildingSpec {
                            building_type: BuildingType::Residential, // Default
                            height,
                            size: Vec2::new(size_x, size_z),
                            rotation: 0.0,
                            prefab_name: parts[3].to_string(),
                        },
                    );
                }
                TileType::Street => {
                    let width = parts[5].parse::<f32>().unwrap_or(8.0);

                    layout.streets.insert(
                        pos,
                        StreetSpec {
                            street_type: StreetType::Main,
                            width,
                            direction: Vec3::Z,
                            has_sidewalks: true,
                            prefab_name: parts[3].to_string(),
                        },
                    );
                }
                TileType::Intersection => {
                    let size = parts[5].parse::<f32>().unwrap_or(12.0);

                    layout.intersections.insert(
                        pos,
                        IntersectionSpec {
                            intersection_type: IntersectionType::FourWay,
                            size,
                            has_traffic_lights: false,
                            prefab_name: parts[3].to_string(),
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(layout)
    }
}
