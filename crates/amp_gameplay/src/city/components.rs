//! City components for building, street, and infrastructure entities

use bevy::prelude::*;

/// Building component with type classification
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Building {
    /// Type of building (residential, commercial, industrial, etc.)
    pub building_type: BuildingType,
    /// Height of the building in meters
    pub height: f32,
    /// Base size of the building (width, depth)
    pub size: Vec2,
    /// Unique identifier for the building
    pub id: u32,
}

/// Building type classification for varied urban landscape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum BuildingType {
    /// Residential apartment buildings
    Residential,
    /// Commercial office buildings
    Commercial,
    /// Industrial warehouses and factories
    Industrial,
    /// Infrastructure buildings (police, fire, hospital)
    Infrastructure,
    /// Skyscrapers for city center
    Skyscraper,
    /// Small shops and businesses
    Shop,
}

/// Street component for road segments
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Street {
    /// Street type (main road, side street, alley)
    pub street_type: StreetType,
    /// Width of the street in meters
    pub width: f32,
    /// Direction of the street (for traffic flow)
    pub direction: Vec3,
    /// Whether this street has sidewalks
    pub has_sidewalks: bool,
}

/// Street type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum StreetType {
    /// Main arterial roads
    Main,
    /// Secondary streets
    Secondary,
    /// Small side streets
    Side,
    /// Narrow alleys
    Alley,
}

/// Intersection component for road crossings
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Intersection {
    /// Type of intersection
    pub intersection_type: IntersectionType,
    /// Size of the intersection
    pub size: f32,
    /// Whether it has traffic lights
    pub has_traffic_lights: bool,
}

/// Intersection type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum IntersectionType {
    /// Four-way intersection
    FourWay,
    /// Three-way T-intersection
    ThreeWay,
    /// Simple corner
    Corner,
}

/// City tile component for grid-based city generation
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CityTile {
    /// Grid position (x, z)
    pub grid_pos: IVec2,
    /// Tile type
    pub tile_type: TileType,
    /// Whether this tile is occupied
    pub occupied: bool,
}

/// City tile type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TileType {
    /// Empty ground
    Empty,
    /// Building tile
    Building,
    /// Street tile
    Street,
    /// Intersection tile
    Intersection,
    /// Park or green space
    Park,
    /// Water feature
    Water,
}

/// Marker component for collider entities
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ColliderMarker {
    /// Type of collider
    pub collider_type: ColliderType,
    /// Parent entity this collider belongs to
    pub parent: Option<Entity>,
}

/// Collider type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ColliderType {
    /// Building collider
    Building,
    /// Street collider
    Street,
    /// Sidewalk collider
    Sidewalk,
    /// Infrastructure collider
    Infrastructure,
}

impl Default for Building {
    fn default() -> Self {
        Self {
            building_type: BuildingType::Residential,
            height: 10.0,
            size: Vec2::new(10.0, 10.0),
            id: 0,
        }
    }
}

impl Default for Street {
    fn default() -> Self {
        Self {
            street_type: StreetType::Side,
            width: 8.0,
            direction: Vec3::Z,
            has_sidewalks: true,
        }
    }
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            intersection_type: IntersectionType::FourWay,
            size: 12.0,
            has_traffic_lights: false,
        }
    }
}

impl Default for CityTile {
    fn default() -> Self {
        Self {
            grid_pos: IVec2::ZERO,
            tile_type: TileType::Empty,
            occupied: false,
        }
    }
}

impl Default for ColliderMarker {
    fn default() -> Self {
        Self {
            collider_type: ColliderType::Building,
            parent: None,
        }
    }
}

/// Deferred light component - stores light data without activating the light
/// Used for the light budget system to manage active lights dynamically
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct DeferredLight {
    /// Light intensity
    pub intensity: f32,
    /// Light color
    pub color: Color,
    /// Light range
    pub range: f32,
    /// Last distance from player (for priority sorting)
    pub last_distance: f32,
    /// Whether this light is currently active (has PointLight component)
    pub is_active: bool,
    /// Light type for categorization
    pub light_type: LightType,
}

/// Light type classification for light budget system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum LightType {
    /// Street light at intersection
    StreetLight,
    /// Building window light
    WindowLight,
    /// Traffic light
    TrafficLight,
    /// Neon sign light
    NeonSign,
}

impl Default for DeferredLight {
    fn default() -> Self {
        Self {
            intensity: 5000.0,
            color: Color::srgb(1.0, 0.9, 0.7),
            range: 25.0,
            last_distance: f32::INFINITY,
            is_active: false,
            light_type: LightType::StreetLight,
        }
    }
}
