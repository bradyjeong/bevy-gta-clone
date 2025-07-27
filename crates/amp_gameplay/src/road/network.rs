use amp_math::spline::Spline;
/// Road network management and generation system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::road::components::{IntersectionType, RoadType};

/// A road represented as a spline curve with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadSpline {
    /// Unique road identifier
    pub id: u32,
    /// Spline curve defining the road path
    pub spline: Spline,
    /// Type of road
    pub road_type: RoadType,
    /// Connected road IDs
    pub connections: Vec<u32>,
    /// Cached length for performance
    cached_length: Option<f32>,
}

impl RoadSpline {
    /// Create a new straight road
    pub fn new_straight(id: u32, start: Vec3, end: Vec3, road_type: RoadType) -> Self {
        Self {
            id,
            spline: Spline::linear(start, end),
            road_type,
            connections: Vec::new(),
            cached_length: None,
        }
    }

    /// Create a new curved road
    pub fn new_curved(id: u32, start: Vec3, control: Vec3, end: Vec3, road_type: RoadType) -> Self {
        Self {
            id,
            spline: Spline::curved(start, control, end),
            road_type,
            connections: Vec::new(),
            cached_length: None,
        }
    }

    /// Create a road from an existing spline
    pub fn from_spline(id: u32, spline: Spline, road_type: RoadType) -> Self {
        Self {
            id,
            spline,
            road_type,
            connections: Vec::new(),
            cached_length: None,
        }
    }

    /// Evaluate the road position at parameter t (0.0 to 1.0)
    pub fn evaluate(&self, t: f32) -> Vec3 {
        self.spline.evaluate(t)
    }

    /// Get the tangent vector at parameter t
    pub fn evaluate_tangent(&self, t: f32) -> Vec3 {
        self.spline.evaluate_tangent(t)
    }

    /// Get the road length
    pub fn length(&self) -> f32 {
        if let Some(cached) = self.cached_length {
            cached
        } else {
            let length = self.spline.length();
            // Note: We can't modify self here due to &self, so we don't cache
            length
        }
    }

    /// Update cached length (requires mutable reference)
    pub fn update_cached_length(&mut self) {
        self.cached_length = Some(self.spline.length());
    }

    /// Add a curve control point to the road
    pub fn add_curve_point(&mut self, point: Vec3) {
        self.spline.add_control_point(point);
        self.cached_length = None;
    }

    /// Connect this road to another road
    pub fn connect_to(&mut self, road_id: u32) {
        if !self.connections.contains(&road_id) {
            self.connections.push(road_id);
        }
    }

    /// Check if a point is on this road within tolerance
    pub fn contains_point(&self, position: Vec3, tolerance: f32) -> bool {
        let (_, distance) = self.spline.closest_point(position);
        distance <= tolerance + self.road_type.width() * 0.5
    }

    /// Find the closest point on this road to a given position
    pub fn closest_point(&self, position: Vec3) -> Vec3 {
        let (closest_point, _) = self.spline.closest_point(position);
        closest_point
    }

    /// Sample points along the road at regular intervals
    pub fn sample_points(&self, count: usize) -> Vec<Vec3> {
        self.spline.sample_points(count)
    }

    /// Get road width
    pub fn width(&self) -> f32 {
        self.road_type.width()
    }
}

/// An intersection connecting multiple roads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadIntersection {
    /// Unique intersection identifier
    pub id: u32,
    /// Position of the intersection center
    pub position: Vec3,
    /// Connected road IDs
    pub connected_roads: Vec<u32>,
    /// Type of intersection
    pub intersection_type: IntersectionType,
    /// Intersection radius
    pub radius: f32,
}

impl RoadIntersection {
    /// Create a new intersection
    pub fn new(
        id: u32,
        position: Vec3,
        connected_roads: Vec<u32>,
        intersection_type: IntersectionType,
    ) -> Self {
        Self {
            id,
            position,
            connected_roads,
            intersection_type,
            radius: intersection_type.radius(),
        }
    }

    /// Add a road connection to this intersection
    pub fn connect_road(&mut self, road_id: u32) {
        if !self.connected_roads.contains(&road_id)
            && self.connected_roads.len() < self.intersection_type.max_connections()
        {
            self.connected_roads.push(road_id);
        }
    }

    /// Check if this intersection can accept more connections
    pub fn can_connect_more(&self) -> bool {
        self.connected_roads.len() < self.intersection_type.max_connections()
    }
}

/// The main road network resource
#[derive(Resource, Debug)]
pub struct RoadNetwork {
    /// All roads in the network
    pub roads: HashMap<u32, RoadSpline>,
    /// All intersections in the network
    pub intersections: HashMap<u32, RoadIntersection>,
    /// Next available road ID
    pub next_road_id: u32,
    /// Next available intersection ID
    pub next_intersection_id: u32,
    /// Generated chunks to avoid regeneration
    pub generated_chunks: HashSet<(i32, i32)>,
    /// Road generation parameters
    pub generation_params: RoadGenerationParams,
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self {
            roads: HashMap::new(),
            intersections: HashMap::new(),
            next_road_id: 0,
            next_intersection_id: 0,
            generated_chunks: HashSet::new(),
            generation_params: RoadGenerationParams::default(),
        }
    }
}

/// Parameters for road generation
#[derive(Debug, Clone)]
pub struct RoadGenerationParams {
    /// Size of each generation chunk
    pub chunk_size: f32,
    /// Generation radius around player
    pub generation_radius: f32,
    /// Cleanup radius for distant roads
    pub cleanup_radius: f32,
    /// Density of road generation
    pub road_density: f32,
    /// Probability of curved roads
    pub curve_probability: f32,
}

impl Default for RoadGenerationParams {
    fn default() -> Self {
        Self {
            chunk_size: 400.0,
            generation_radius: 800.0,
            cleanup_radius: 2000.0,
            road_density: 0.8,
            curve_probability: 0.3,
        }
    }
}

impl RoadNetwork {
    /// Create a new road network with custom parameters
    pub fn new(params: RoadGenerationParams) -> Self {
        Self {
            roads: HashMap::new(),
            intersections: HashMap::new(),
            next_road_id: 0,
            next_intersection_id: 0,
            generated_chunks: HashSet::new(),
            generation_params: params,
        }
    }

    /// Add a straight road to the network
    pub fn add_road(&mut self, start: Vec3, end: Vec3, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;

        let mut road = RoadSpline::new_straight(id, start, end, road_type);
        road.update_cached_length();
        self.roads.insert(id, road);
        id
    }

    /// Add a curved road to the network
    pub fn add_curved_road(
        &mut self,
        start: Vec3,
        control: Vec3,
        end: Vec3,
        road_type: RoadType,
    ) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;

        let mut road = RoadSpline::new_curved(id, start, control, end, road_type);
        road.update_cached_length();
        self.roads.insert(id, road);
        id
    }

    /// Add a road from an existing spline
    pub fn add_spline_road(&mut self, spline: Spline, road_type: RoadType) -> u32 {
        let id = self.next_road_id;
        self.next_road_id += 1;

        let mut road = RoadSpline::from_spline(id, spline, road_type);
        road.update_cached_length();
        self.roads.insert(id, road);
        id
    }

    /// Connect two roads in the network
    pub fn connect_roads(&mut self, road1_id: u32, road2_id: u32) {
        if let Some(road1) = self.roads.get_mut(&road1_id) {
            road1.connect_to(road2_id);
        }
        if let Some(road2) = self.roads.get_mut(&road2_id) {
            road2.connect_to(road1_id);
        }
    }

    /// Add an intersection to the network
    pub fn add_intersection(
        &mut self,
        position: Vec3,
        connected_roads: Vec<u32>,
        intersection_type: IntersectionType,
    ) -> u32 {
        let id = self.next_intersection_id;
        self.next_intersection_id += 1;

        let intersection = RoadIntersection::new(id, position, connected_roads, intersection_type);
        self.intersections.insert(id, intersection);
        id
    }

    /// Generate roads for a specific chunk
    pub fn generate_chunk_roads(&mut self, chunk_x: i32, chunk_z: i32) -> Vec<u32> {
        if self.generated_chunks.contains(&(chunk_x, chunk_z)) {
            return Vec::new();
        }

        self.generated_chunks.insert((chunk_x, chunk_z));

        let base_x = chunk_x as f32 * self.generation_params.chunk_size;
        let base_z = chunk_z as f32 * self.generation_params.chunk_size;

        if chunk_x == 0 && chunk_z == 0 {
            self.generate_spawn_chunk_roads(base_x, base_z)
        } else {
            self.generate_regular_chunk_roads(base_x, base_z, chunk_x, chunk_z)
        }
    }

    /// Generate premium roads for the spawn chunk (0,0)
    fn generate_spawn_chunk_roads(&mut self, base_x: f32, base_z: f32) -> Vec<u32> {
        let mut new_roads = Vec::new();
        let chunk_size = self.generation_params.chunk_size;

        // Main highway (horizontal through center)
        let highway_start = Vec3::new(base_x, 0.0, base_z + chunk_size * 0.5);
        let highway_control = Vec3::new(base_x + chunk_size * 0.3, 0.0, base_z + chunk_size * 0.6);
        let highway_end = Vec3::new(base_x + chunk_size, 0.0, base_z + chunk_size * 0.5);
        let highway_id = self.add_curved_road(
            highway_start,
            highway_control,
            highway_end,
            RoadType::Highway,
        );
        new_roads.push(highway_id);

        // Cross highway (vertical through center)
        let cross_start = Vec3::new(base_x + chunk_size * 0.5, 0.0, base_z);
        let cross_control = Vec3::new(base_x + chunk_size * 0.6, 0.0, base_z + chunk_size * 0.3);
        let cross_end = Vec3::new(base_x + chunk_size * 0.5, 0.0, base_z + chunk_size);
        let cross_id =
            self.add_curved_road(cross_start, cross_control, cross_end, RoadType::Highway);
        new_roads.push(cross_id);

        // Main streets parallel to highways
        let main_street_start = Vec3::new(base_x, 0.0, base_z + chunk_size * 0.25);
        let main_street_end = Vec3::new(base_x + chunk_size, 0.0, base_z + chunk_size * 0.25);
        let main_street_id =
            self.add_road(main_street_start, main_street_end, RoadType::MainStreet);
        new_roads.push(main_street_id);

        // Add side streets for density
        for i in 0..3 {
            for j in 0..3 {
                if i == 1 && j == 1 {
                    continue;
                } // Skip center where highways cross

                let sub_x = base_x + (i as f32 + 0.5) * chunk_size / 4.0;
                let sub_z = base_z + (j as f32 + 0.5) * chunk_size / 4.0;

                // Horizontal side street
                let h_start = Vec3::new(sub_x - 30.0, 0.0, sub_z);
                let h_end = Vec3::new(sub_x + 30.0, 0.0, sub_z);
                let h_id = self.add_road(h_start, h_end, RoadType::SideStreet);
                new_roads.push(h_id);

                // Vertical side street
                let v_start = Vec3::new(sub_x, 0.0, sub_z - 30.0);
                let v_end = Vec3::new(sub_x, 0.0, sub_z + 30.0);
                let v_id = self.add_road(v_start, v_end, RoadType::SideStreet);
                new_roads.push(v_id);
            }
        }

        info!(
            "Generated {} premium roads for spawn chunk (0,0)",
            new_roads.len()
        );
        new_roads
    }

    /// Generate regular roads for non-spawn chunks
    fn generate_regular_chunk_roads(
        &mut self,
        base_x: f32,
        base_z: f32,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Vec<u32> {
        let mut new_roads = Vec::new();
        let chunk_size = self.generation_params.chunk_size;

        // Use alternating pattern to prevent overlapping roads
        if chunk_x % 2 == 0 && chunk_z % 2 != 0 {
            // Vertical main street
            let start = Vec3::new(base_x, 0.0, base_z - chunk_size * 0.5);
            let control = Vec3::new(
                base_x + (fastrand::f32() - 0.5) * 20.0,
                0.0,
                base_z + chunk_size * 0.2,
            );
            let end = Vec3::new(base_x, 0.0, base_z + chunk_size * 1.5);
            let road_id = self.add_curved_road(start, control, end, RoadType::MainStreet);
            new_roads.push(road_id);
        }

        if chunk_z % 2 == 0 && chunk_x % 2 != 0 {
            // Horizontal main street
            let start = Vec3::new(base_x - chunk_size * 0.5, 0.0, base_z);
            let control = Vec3::new(
                base_x + chunk_size * 0.2,
                0.0,
                base_z + (fastrand::f32() - 0.5) * 20.0,
            );
            let end = Vec3::new(base_x + chunk_size * 1.5, 0.0, base_z);
            let road_id = self.add_curved_road(start, control, end, RoadType::MainStreet);
            new_roads.push(road_id);
        }

        // Add side streets with density control
        if new_roads.is_empty() && fastrand::f32() < self.generation_params.road_density {
            for i in 0..2 {
                for j in 0..2 {
                    let sub_x = base_x + (i as f32 + 0.5) * chunk_size / 3.0;
                    let sub_z = base_z + (j as f32 + 0.5) * chunk_size / 3.0;

                    let offset_x = (fastrand::f32() - 0.5) * 30.0;
                    let offset_z = (fastrand::f32() - 0.5) * 30.0;

                    if fastrand::f32() < self.generation_params.road_density {
                        let start = Vec3::new(sub_x + offset_x, 0.0, sub_z - 40.0);
                        let end = Vec3::new(sub_x + offset_x, 0.0, sub_z + 40.0);

                        let road_id = if fastrand::f32() < self.generation_params.curve_probability
                        {
                            let control = Vec3::new(
                                sub_x + offset_x + (fastrand::f32() - 0.5) * 20.0,
                                0.0,
                                sub_z,
                            );
                            self.add_curved_road(start, control, end, RoadType::SideStreet)
                        } else {
                            self.add_road(start, end, RoadType::SideStreet)
                        };
                        new_roads.push(road_id);
                    }
                }
            }
        }

        new_roads
    }

    /// Find the nearest road to a given position
    pub fn find_nearest_road(&self, position: Vec3) -> Option<(u32, Vec3, f32)> {
        let mut nearest_road_id = None;
        let mut nearest_point = Vec3::ZERO;
        let mut nearest_distance = f32::INFINITY;

        for (road_id, road) in &self.roads {
            let closest_point = road.closest_point(position);
            let distance = position.distance(closest_point);

            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_point = closest_point;
                nearest_road_id = Some(*road_id);
            }
        }

        nearest_road_id.map(|id| (id, nearest_point, nearest_distance))
    }

    /// Check if a position is on any road within tolerance
    pub fn is_on_road(&self, position: Vec3, tolerance: f32) -> bool {
        self.roads
            .values()
            .any(|road| road.contains_point(position, tolerance))
    }

    /// Get all roads within a certain distance of a position
    pub fn roads_near_position(&self, position: Vec3, radius: f32) -> Vec<u32> {
        self.roads
            .iter()
            .filter_map(|(id, road)| {
                let closest_point = road.closest_point(position);
                if position.distance(closest_point) <= radius {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Clear the generation cache
    pub fn clear_cache(&mut self) {
        self.generated_chunks.clear();
        info!("Road network cache cleared");
    }

    /// Reset the entire road network
    pub fn reset(&mut self) {
        self.roads.clear();
        self.intersections.clear();
        self.generated_chunks.clear();
        self.next_road_id = 0;
        self.next_intersection_id = 0;
        info!("Road network completely reset");
    }

    /// Get network statistics
    pub fn stats(&self) -> RoadNetworkStats {
        let total_length = self.roads.values().map(|road| road.length()).sum();

        let mut road_counts = HashMap::new();
        for road in self.roads.values() {
            *road_counts.entry(road.road_type).or_insert(0) += 1;
        }

        RoadNetworkStats {
            total_roads: self.roads.len(),
            total_intersections: self.intersections.len(),
            total_length,
            generated_chunks: self.generated_chunks.len(),
            road_type_counts: road_counts,
        }
    }
}

/// Statistics about the road network
#[derive(Debug, Clone)]
pub struct RoadNetworkStats {
    pub total_roads: usize,
    pub total_intersections: usize,
    pub total_length: f32,
    pub generated_chunks: usize,
    pub road_type_counts: HashMap<RoadType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_spline_creation() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(100.0, 0.0, 0.0);
        let road = RoadSpline::new_straight(1, start, end, RoadType::MainStreet);

        assert_eq!(road.id, 1);
        assert_eq!(road.road_type, RoadType::MainStreet);
        assert_eq!(road.evaluate(0.0), start);
        assert_eq!(road.evaluate(1.0), end);
        assert!((road.length() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_road_network_operations() {
        let mut network = RoadNetwork::default();

        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(100.0, 0.0, 0.0);
        let road_id = network.add_road(start, end, RoadType::MainStreet);

        assert_eq!(road_id, 0);
        assert_eq!(network.roads.len(), 1);
        assert!(network.roads.contains_key(&road_id));

        let road = &network.roads[&road_id];
        assert_eq!(road.road_type, RoadType::MainStreet);
    }

    #[test]
    fn test_intersection_creation() {
        let intersection = RoadIntersection::new(
            1,
            Vec3::new(50.0, 0.0, 50.0),
            vec![1, 2, 3, 4],
            IntersectionType::Cross,
        );

        assert_eq!(intersection.id, 1);
        assert_eq!(intersection.connected_roads.len(), 4);
        assert_eq!(intersection.intersection_type, IntersectionType::Cross);
        assert_eq!(intersection.radius, 20.0);
    }

    #[test]
    fn test_road_connections() {
        let mut network = RoadNetwork::default();

        let road1_id =
            network.add_road(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), RoadType::MainStreet);
        let road2_id = network.add_road(
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            RoadType::SideStreet,
        );

        network.connect_roads(road1_id, road2_id);

        let road1 = &network.roads[&road1_id];
        let road2 = &network.roads[&road2_id];

        assert!(road1.connections.contains(&road2_id));
        assert!(road2.connections.contains(&road1_id));
    }

    #[test]
    fn test_nearest_road_finding() {
        let mut network = RoadNetwork::default();

        let _road_id = network.add_road(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 0.0),
            RoadType::MainStreet,
        );

        let test_position = Vec3::new(50.0, 0.0, 5.0);
        if let Some((_, nearest_point, distance)) = network.find_nearest_road(test_position) {
            assert!((nearest_point - Vec3::new(50.0, 0.0, 0.0)).length() < 0.1);
            assert!((distance - 5.0).abs() < 0.1);
        } else {
            panic!("Should have found a nearest road");
        }
    }

    #[test]
    fn test_road_network_stats() {
        let mut network = RoadNetwork::default();

        network.add_road(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), RoadType::Highway);
        network.add_road(Vec3::ZERO, Vec3::new(0.0, 0.0, 100.0), RoadType::MainStreet);
        network.add_road(Vec3::ZERO, Vec3::new(50.0, 0.0, 50.0), RoadType::SideStreet);

        let stats = network.stats();
        assert_eq!(stats.total_roads, 3);
        assert_eq!(stats.road_type_counts[&RoadType::Highway], 1);
        assert_eq!(stats.road_type_counts[&RoadType::MainStreet], 1);
        assert_eq!(stats.road_type_counts[&RoadType::SideStreet], 1);
        assert!(stats.total_length > 200.0);
    }
}
