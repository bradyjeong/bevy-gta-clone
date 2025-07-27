use amp_math::spline::Spline;
/// High-performance road mesh generation with proper UV mapping and optimization
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::HashMap;

/// Road mesh generation parameters
#[derive(Debug, Clone)]
pub struct RoadMeshParams {
    /// Road width in meters
    pub width: f32,
    /// Number of segments along the road length
    pub segments: usize,
    /// UV scale factor for texture tiling
    pub uv_scale: f32,
    /// Whether to generate smooth normals
    pub smooth_normals: bool,
}

impl Default for RoadMeshParams {
    fn default() -> Self {
        Self {
            width: 8.0,
            segments: 20,
            uv_scale: 1.0,
            smooth_normals: true,
        }
    }
}

/// Lane marking generation parameters
#[derive(Debug, Clone)]
pub struct MarkingParams {
    /// Width of lane markings in meters
    pub line_width: f32,
    /// Dash length for dashed lines
    pub dash_length: f32,
    /// Gap length for dashed lines
    pub gap_length: f32,
    /// UV scale for marking textures
    pub uv_scale: f32,
}

impl Default for MarkingParams {
    fn default() -> Self {
        Self {
            line_width: 0.2,
            dash_length: 3.0,
            gap_length: 2.0,
            uv_scale: 1.0,
        }
    }
}

/// Generate a road surface mesh from a spline
pub fn generate_road_mesh(spline: &Spline, params: &RoadMeshParams) -> Mesh {
    let segments = calculate_optimal_segments(spline, params.segments);

    // Pre-allocate with exact capacity for performance
    let vertex_count = (segments + 1) * 2;
    let index_count = segments * 6;

    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);

    let half_width = params.width * 0.5;

    // Generate vertices along the spline
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);

        // Calculate right vector (perpendicular to tangent)
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        // Left and right edge vertices
        let left_pos = position + right * half_width;
        let right_pos = position - right * half_width;

        // Add vertices
        vertices.push([left_pos.x, left_pos.y, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y, right_pos.z]);

        // Calculate normals
        let normal = if params.smooth_normals {
            calculate_smooth_normal(spline, t, &right)
        } else {
            [0.0, 1.0, 0.0]
        };

        normals.push(normal);
        normals.push(normal);

        // UV coordinates with proper scaling
        let v = t * params.uv_scale;
        uvs.push([0.0, v]);
        uvs.push([1.0, v]);

        // Generate triangles (except for the last segment)
        if i < segments {
            let base = (i * 2) as u32;

            // First triangle (counter-clockwise winding)
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            // Second triangle
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate lane marking meshes
pub fn generate_lane_markings(
    spline: &Spline,
    road_width: f32,
    lane_count: u32,
    params: &MarkingParams,
) -> Vec<Mesh> {
    let mut markings = Vec::new();

    // Generate center line
    if lane_count > 1 {
        let center_line = generate_center_line_mesh(spline, params, true);
        markings.push(center_line);
    }

    // Generate lane divider lines for multi-lane roads
    if lane_count > 2 {
        let lane_width = road_width / lane_count as f32;

        for lane in 1..lane_count {
            let offset = (lane as f32 - lane_count as f32 * 0.5) * lane_width;
            let divider_line = generate_lane_divider_mesh(spline, offset, params);
            markings.push(divider_line);
        }
    }

    // Generate edge lines
    let left_edge = generate_edge_line_mesh(spline, road_width * 0.5, params);
    let right_edge = generate_edge_line_mesh(spline, -road_width * 0.5, params);

    markings.push(left_edge);
    markings.push(right_edge);

    markings
}

/// Generate a center line mesh (can be solid or dashed)
fn generate_center_line_mesh(spline: &Spline, params: &MarkingParams, dashed: bool) -> Mesh {
    let segments = calculate_optimal_segments(spline, 40);
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let dash_cycle = params.dash_length + params.gap_length;
    let total_length = spline.length();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;

        // Skip segments for dashed line
        if dashed {
            let distance = t * total_length;
            let cycle_position = distance % dash_cycle;
            if cycle_position > params.dash_length {
                continue;
            }
        }

        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        // Line vertices
        let left_pos = position + right * half_width;
        let right_pos = position - right * half_width;

        let base_idx = vertices.len() as u32;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]); // Slightly above road surface
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        // Generate triangles for line segments
        if vertices.len() >= 4 && base_idx >= 2 {
            indices.push(base_idx - 2);
            indices.push(base_idx);
            indices.push(base_idx - 1);

            indices.push(base_idx - 1);
            indices.push(base_idx);
            indices.push(base_idx + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate lane divider mesh at specified offset
fn generate_lane_divider_mesh(spline: &Spline, offset: f32, params: &MarkingParams) -> Mesh {
    let segments = calculate_optimal_segments(spline, 40);
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Dashed line for lane dividers
    let dash_cycle = params.dash_length + params.gap_length;
    let total_length = spline.length();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;

        // Dashed pattern
        let distance = t * total_length;
        let cycle_position = distance % dash_cycle;
        if cycle_position > params.dash_length {
            continue;
        }

        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        let line_center = position + right * offset;
        let left_pos = line_center + right * half_width;
        let right_pos = line_center - right * half_width;

        let base_idx = vertices.len() as u32;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        if vertices.len() >= 4 && base_idx >= 2 {
            indices.push(base_idx - 2);
            indices.push(base_idx);
            indices.push(base_idx - 1);

            indices.push(base_idx - 1);
            indices.push(base_idx);
            indices.push(base_idx + 1);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate edge line mesh (solid white lines at road edges)
fn generate_edge_line_mesh(spline: &Spline, offset: f32, params: &MarkingParams) -> Mesh {
    let segments = calculate_optimal_segments(spline, 30);
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        let line_center = position + right * offset;
        let left_pos = line_center + right * half_width;
        let right_pos = line_center - right * half_width;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        if i > 0 {
            let base = ((i - 1) * 2) as u32;

            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 1);

            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 3);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate intersection mesh for different intersection types
pub fn generate_intersection_mesh(
    center: Vec3,
    radius: f32,
    intersection_type: IntersectionType,
) -> Mesh {
    match intersection_type {
        IntersectionType::Cross => generate_cross_intersection_mesh(center, radius),
        IntersectionType::TJunction => generate_t_junction_mesh(center, radius),
        IntersectionType::Curve => generate_curved_intersection_mesh(center, radius),
        IntersectionType::HighwayOnramp => generate_onramp_mesh(center, radius),
    }
}

/// Intersection types (matching the one from gameplay)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionType {
    Cross,
    TJunction,
    Curve,
    HighwayOnramp,
}

/// Generate a cross intersection mesh (4-way)
fn generate_cross_intersection_mesh(center: Vec3, radius: f32) -> Mesh {
    let segments = 16;

    let mut vertices = Vec::with_capacity(segments + 1);
    let mut normals = Vec::with_capacity(segments + 1);
    let mut uvs = Vec::with_capacity(segments + 1);
    let mut indices = Vec::with_capacity(segments * 3);

    // Center vertex
    vertices.push([center.x, center.y, center.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    // Circle vertices
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = center.x + radius * angle.cos();
        let z = center.z + radius * angle.sin();

        vertices.push([x, center.y, z]);
        normals.push([0.0, 1.0, 0.0]);

        let u = 0.5 + 0.5 * angle.cos();
        let v = 0.5 + 0.5 * angle.sin();
        uvs.push([u, v]);

        // Triangle from center to edge
        let next_i = (i + 1) % segments;
        indices.push(0); // Center
        indices.push((i + 1) as u32);
        indices.push((next_i + 1) as u32);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate T-junction mesh (simplified for now)
fn generate_t_junction_mesh(center: Vec3, radius: f32) -> Mesh {
    generate_cross_intersection_mesh(center, radius * 0.8)
}

/// Generate curved intersection mesh
fn generate_curved_intersection_mesh(center: Vec3, radius: f32) -> Mesh {
    generate_cross_intersection_mesh(center, radius * 0.6)
}

/// Generate highway onramp mesh
fn generate_onramp_mesh(center: Vec3, radius: f32) -> Mesh {
    generate_cross_intersection_mesh(center, radius * 1.2)
}

/// Calculate optimal number of segments based on spline curvature and length
fn calculate_optimal_segments(spline: &Spline, base_segments: usize) -> usize {
    let length = spline.length();
    let segments_by_length = (length / 10.0) as usize; // Segment every 10 units

    // Adjust based on curvature (simplified heuristic)
    let control_points = spline.control_points.len();
    let curvature_multiplier = if control_points > 2 { 1.5 } else { 1.0 };

    let optimal = ((segments_by_length as f32 * curvature_multiplier) as usize)
        .max(base_segments.min(4))
        .min(100); // Cap at 100 segments for performance

    optimal
}

/// Calculate smooth normal vector for a point on the road
fn calculate_smooth_normal(_spline: &Spline, _t: f32, _right: &Vec3) -> [f32; 3] {
    // For now, return upward normal
    // Future enhancement: consider road banking based on curvature
    [0.0, 1.0, 0.0]
}

/// Mesh cache for road rendering performance
#[derive(Default)]
pub struct RoadMeshCache {
    /// Cached road surface meshes
    road_meshes: HashMap<String, Handle<Mesh>>,
    /// Cached marking meshes
    marking_meshes: HashMap<String, Vec<Handle<Mesh>>>,
    /// Maximum cache size
    max_size: usize,
}

impl RoadMeshCache {
    /// Create a new mesh cache with specified size limit
    pub fn new(max_size: usize) -> Self {
        Self {
            road_meshes: HashMap::new(),
            marking_meshes: HashMap::new(),
            max_size,
        }
    }

    /// Get or generate a road mesh
    pub fn get_or_generate_road_mesh<F>(&mut self, key: String, generator: F) -> Handle<Mesh>
    where
        F: FnOnce() -> Handle<Mesh>,
    {
        if let Some(handle) = self.road_meshes.get(&key) {
            handle.clone()
        } else {
            let handle = generator();
            if self.road_meshes.len() < self.max_size {
                self.road_meshes.insert(key, handle.clone());
            }
            handle
        }
    }

    /// Clear cache when it gets too large
    pub fn cleanup_if_needed(&mut self) {
        if self.road_meshes.len() > self.max_size {
            self.road_meshes.clear();
            self.marking_meshes.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amp_math::spline::Spline;

    #[test]
    fn test_road_mesh_generation() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0));
        let params = RoadMeshParams::default();

        let mesh = generate_road_mesh(&spline, &params);

        // Verify mesh has the expected attributes
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }

    #[test]
    fn test_lane_marking_generation() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0));
        let params = MarkingParams::default();

        let markings = generate_lane_markings(&spline, 8.0, 2, &params);

        // Should generate center line and edge lines for 2-lane road
        assert!(!markings.is_empty());
        assert!(markings.len() >= 2); // At least center line and edge lines
    }

    #[test]
    fn test_intersection_mesh_generation() {
        let center = Vec3::new(50.0, 0.0, 50.0);
        let radius = 20.0;

        let mesh = generate_intersection_mesh(center, radius, IntersectionType::Cross);

        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }

    #[test]
    fn test_optimal_segments_calculation() {
        let short_spline = Spline::linear(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0));
        let long_spline = Spline::linear(Vec3::ZERO, Vec3::new(200.0, 0.0, 0.0));

        let short_segments = calculate_optimal_segments(&short_spline, 10);
        let long_segments = calculate_optimal_segments(&long_spline, 10);

        assert!(long_segments > short_segments);
        assert!(short_segments >= 4); // Minimum segments
        assert!(long_segments <= 100); // Maximum segments
    }

    #[test]
    fn test_mesh_cache() {
        let mut cache = RoadMeshCache::new(5);

        // Mock handle for testing
        let handle = Handle::default();

        // Test cache hit/miss
        let key = "test_road".to_string();
        let result = cache.get_or_generate_road_mesh(key.clone(), || handle.clone());

        assert_eq!(result.id(), handle.id());
        assert!(cache.road_meshes.contains_key(&key));
    }
}
