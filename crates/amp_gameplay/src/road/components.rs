/// Road system components for Bevy ECS
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Road entity marker component
#[derive(Component, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RoadEntity {
    /// Unique road identifier in the road network
    pub road_id: u32,
}

/// Intersection entity marker component
#[derive(Component, Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct IntersectionEntity {
    /// Unique intersection identifier
    pub intersection_id: u32,
}

/// Road type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum RoadType {
    /// Wide highway with 4+ lanes and barriers
    Highway,
    /// Medium main street with 2-4 lanes and intersections
    MainStreet,
    /// Narrow residential street with 2 lanes
    SideStreet,
    /// Very narrow alley with 1 lane behind buildings
    Alley,
}

impl Default for RoadType {
    fn default() -> Self {
        Self::SideStreet
    }
}

impl RoadType {
    /// Get the width of this road type in meters
    pub fn width(&self) -> f32 {
        match self {
            RoadType::Highway => 16.0,    // 4 lanes + shoulders
            RoadType::MainStreet => 12.0, // 3-4 lanes
            RoadType::SideStreet => 8.0,  // 2 lanes
            RoadType::Alley => 4.0,       // 1 lane
        }
    }

    /// Get the priority level for intersection management
    pub fn priority(&self) -> u8 {
        match self {
            RoadType::Highway => 4,
            RoadType::MainStreet => 3,
            RoadType::SideStreet => 2,
            RoadType::Alley => 1,
        }
    }

    /// Get the speed limit for this road type in m/s
    pub fn speed_limit(&self) -> f32 {
        match self {
            RoadType::Highway => 36.0,    // ~130 km/h
            RoadType::MainStreet => 22.0, // ~80 km/h
            RoadType::SideStreet => 14.0, // ~50 km/h
            RoadType::Alley => 8.0,       // ~30 km/h
        }
    }

    /// Get the number of lanes for this road type
    pub fn lane_count(&self) -> u32 {
        match self {
            RoadType::Highway => 4,
            RoadType::MainStreet => 3,
            RoadType::SideStreet => 2,
            RoadType::Alley => 1,
        }
    }

    /// Check if this road type supports lane markings
    pub fn has_lane_markings(&self) -> bool {
        match self {
            RoadType::Highway | RoadType::MainStreet => true,
            RoadType::SideStreet => true,
            RoadType::Alley => false,
        }
    }

    /// Check if this road type has barriers or curbs
    pub fn has_barriers(&self) -> bool {
        match self {
            RoadType::Highway => true,
            RoadType::MainStreet | RoadType::SideStreet | RoadType::Alley => false,
        }
    }
}

/// Intersection type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum IntersectionType {
    /// 4-way cross intersection
    Cross,
    /// 3-way T-junction
    TJunction,
    /// 2-way curved connection
    Curve,
    /// Highway merge/onramp
    HighwayOnramp,
}

impl Default for IntersectionType {
    fn default() -> Self {
        Self::Cross
    }
}

impl IntersectionType {
    /// Get the radius for this intersection type
    pub fn radius(&self) -> f32 {
        match self {
            IntersectionType::Cross => 20.0,
            IntersectionType::TJunction => 15.0,
            IntersectionType::Curve => 12.0,
            IntersectionType::HighwayOnramp => 30.0,
        }
    }

    /// Get the maximum number of roads that can connect to this intersection
    pub fn max_connections(&self) -> usize {
        match self {
            IntersectionType::Cross => 4,
            IntersectionType::TJunction => 3,
            IntersectionType::Curve => 2,
            IntersectionType::HighwayOnramp => 2,
        }
    }
}

/// Road surface material type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum RoadSurface {
    /// New asphalt surface
    Asphalt,
    /// Older concrete surface
    Concrete,
    /// Dirt or gravel road
    Dirt,
    /// Cobblestone (historical areas)
    Cobblestone,
}

impl Default for RoadSurface {
    fn default() -> Self {
        Self::Asphalt
    }
}

impl RoadSurface {
    /// Get the friction coefficient for this surface type
    pub fn friction_coefficient(&self) -> f32 {
        match self {
            RoadSurface::Asphalt => 0.9,
            RoadSurface::Concrete => 0.8,
            RoadSurface::Dirt => 0.6,
            RoadSurface::Cobblestone => 0.7,
        }
    }

    /// Get the base color for material generation
    pub fn base_color(&self) -> Color {
        match self {
            RoadSurface::Asphalt => Color::srgb(0.35, 0.35, 0.4),
            RoadSurface::Concrete => Color::srgb(0.5, 0.5, 0.55),
            RoadSurface::Dirt => Color::srgb(0.6, 0.4, 0.3),
            RoadSurface::Cobblestone => Color::srgb(0.4, 0.4, 0.45),
        }
    }

    /// Get the roughness value for PBR materials
    pub fn roughness(&self) -> f32 {
        match self {
            RoadSurface::Asphalt => 0.8,
            RoadSurface::Concrete => 0.7,
            RoadSurface::Dirt => 0.9,
            RoadSurface::Cobblestone => 0.6,
        }
    }
}

/// Road configuration component
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct RoadConfig {
    /// Type of road
    pub road_type: RoadType,
    /// Surface material
    pub surface: RoadSurface,
    /// Connected road IDs
    pub connections: Vec<u32>,
    /// Whether this road is enabled for traffic
    pub enabled: bool,
    /// Construction state (for dynamic road building)
    pub construction_progress: f32,
}

impl Default for RoadConfig {
    fn default() -> Self {
        Self {
            road_type: RoadType::default(),
            surface: RoadSurface::default(),
            connections: Vec::new(),
            enabled: true,
            construction_progress: 1.0,
        }
    }
}

/// Intersection configuration component
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct IntersectionConfig {
    /// Type of intersection
    pub intersection_type: IntersectionType,
    /// Connected road IDs
    pub connected_roads: Vec<u32>,
    /// Traffic control type (future: traffic lights, stop signs)
    pub traffic_control: TrafficControl,
    /// Whether intersection is enabled
    pub enabled: bool,
}

impl Default for IntersectionConfig {
    fn default() -> Self {
        Self {
            intersection_type: IntersectionType::default(),
            connected_roads: Vec::new(),
            traffic_control: TrafficControl::default(),
            enabled: true,
        }
    }
}

/// Traffic control at intersections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum TrafficControl {
    /// No traffic control
    None,
    /// Stop signs on all approaches
    AllWayStop,
    /// Yield signs on minor roads
    Yield,
    /// Traffic lights (future implementation)
    TrafficLight,
    /// Roundabout (future implementation)
    Roundabout,
}

impl Default for TrafficControl {
    fn default() -> Self {
        Self::None
    }
}

/// Road generation timer resource for performance optimization
#[derive(Resource, Default)]
pub struct RoadGenerationTimer {
    /// Timer for reducing road generation frequency
    pub timer: f32,
    /// Last player chunk position to detect movement
    pub last_player_chunk: Option<(i32, i32)>,
    /// Chunk size for road generation
    pub chunk_size: f32,
}

impl RoadGenerationTimer {
    pub fn new(chunk_size: f32) -> Self {
        Self {
            timer: 0.0,
            last_player_chunk: None,
            chunk_size,
        }
    }
}

/// Road mesh cache for performance optimization
#[derive(Resource, Default)]
pub struct RoadMeshCache {
    /// Cached road meshes by configuration hash
    pub road_meshes: std::collections::HashMap<String, Handle<Mesh>>,
    /// Cached marking meshes by configuration hash
    pub marking_meshes: std::collections::HashMap<String, Vec<Handle<Mesh>>>,
    /// Maximum cache size to prevent memory leaks
    pub max_cache_size: usize,
}

impl RoadMeshCache {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            road_meshes: std::collections::HashMap::new(),
            marking_meshes: std::collections::HashMap::new(),
            max_cache_size,
        }
    }

    /// Clear the cache when it gets too large
    pub fn cleanup_if_needed(&mut self) {
        if self.road_meshes.len() > self.max_cache_size {
            self.road_meshes.clear();
            self.marking_meshes.clear();
        }
    }
}

/// Dynamic content marker for cleanup systems
#[derive(Component, Debug, Clone, Copy)]
pub struct DynamicRoadContent {
    /// Type of dynamic content
    pub content_type: RoadContentType,
}

/// Types of dynamic road content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadContentType {
    /// Road surface mesh
    Road,
    /// Lane markings
    Markings,
    /// Intersection surface
    Intersection,
    /// Road barriers or curbs
    Barriers,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_type_properties() {
        assert_eq!(RoadType::Highway.width(), 16.0);
        assert_eq!(RoadType::Highway.priority(), 4);
        assert_eq!(RoadType::Highway.lane_count(), 4);
        assert!(RoadType::Highway.has_lane_markings());
        assert!(RoadType::Highway.has_barriers());

        assert_eq!(RoadType::Alley.width(), 4.0);
        assert_eq!(RoadType::Alley.priority(), 1);
        assert_eq!(RoadType::Alley.lane_count(), 1);
        assert!(!RoadType::Alley.has_lane_markings());
        assert!(!RoadType::Alley.has_barriers());
    }

    #[test]
    fn test_intersection_type_properties() {
        assert_eq!(IntersectionType::Cross.radius(), 20.0);
        assert_eq!(IntersectionType::Cross.max_connections(), 4);

        assert_eq!(IntersectionType::Curve.radius(), 12.0);
        assert_eq!(IntersectionType::Curve.max_connections(), 2);
    }

    #[test]
    fn test_road_surface_properties() {
        assert_eq!(RoadSurface::Asphalt.friction_coefficient(), 0.9);
        assert_eq!(RoadSurface::Dirt.friction_coefficient(), 0.6);

        let asphalt_color = RoadSurface::Asphalt.base_color();
        assert!(!asphalt_color.to_srgba().red.is_nan());
    }

    #[test]
    fn test_road_entity_creation() {
        let road = RoadEntity { road_id: 42 };
        assert_eq!(road.road_id, 42);

        let intersection = IntersectionEntity {
            intersection_id: 24,
        };
        assert_eq!(intersection.intersection_id, 24);
    }

    #[test]
    fn test_road_config_defaults() {
        let config = RoadConfig::default();
        assert_eq!(config.road_type, RoadType::SideStreet);
        assert_eq!(config.surface, RoadSurface::Asphalt);
        assert!(config.enabled);
        assert_eq!(config.construction_progress, 1.0);
    }
}
