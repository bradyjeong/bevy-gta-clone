// GPU Culling Compute Shader - Phase 2 Implementation
// Implementation: Sprint 8 - Full frustum culling with distance LOD
//
// This shader performs GPU-based frustum culling of instances with proper
// 6-plane frustum intersection testing and distance-based LOD selection.

// Bind Group 0: Instance Data (read-only storage buffer)
@group(0) @binding(0)
var<storage, read> instance_data: array<InstanceData>;

// Bind Group 1: Culling Parameters (uniform buffer)
@group(1) @binding(0)
var<uniform> culling_params: CullingParams;

// Bind Group 2: Visibility Output (read-write storage buffer)
@group(2) @binding(0)
var<storage, read_write> visibility_output: array<u32>;

// Instance data structure matching Rust InstanceMeta
struct InstanceData {
    transform_matrix: mat4x4<f32>,
    bounds_center: vec3<f32>,
    bounds_radius: f32,
    lod_level: u32,
    _padding: vec3<u32>,
}

// Culling parameters matching CameraProjectionConfig
struct CullingParams {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_projection_matrix: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    camera_position: vec3<f32>,
    near_plane: f32,
    far_plane: f32,
    _padding: vec3<f32>,
}

// Test sphere against frustum plane
fn test_sphere_plane(center: vec3<f32>, radius: f32, plane: vec4<f32>) -> bool {
    let distance = dot(center, plane.xyz) + plane.w;
    return distance >= -radius;
}

// Test sphere against all 6 frustum planes
fn test_sphere_frustum(center: vec3<f32>, radius: f32, frustum: array<vec4<f32>, 6>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        if (!test_sphere_plane(center, radius, frustum[i])) {
            return false;
        }
    }
    return true;
}

// Calculate LOD level based on distance from camera
fn calculate_lod_level(distance: f32) -> u32 {
    // LOD thresholds: 0-50m = LOD 0, 50-150m = LOD 1, 150-400m = LOD 2, 400m+ = LOD 3
    if (distance < 50.0) {
        return 0u;
    } else if (distance < 150.0) {
        return 1u;
    } else if (distance < 400.0) {
        return 2u;
    } else {
        return 3u;
    }
}

// Compute shader entry point
// Workgroup size: 64 threads per group (optimal for most GPUs)
@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_index = global_id.x;
    
    // Bounds check - ensure we don't exceed array size
    if (instance_index >= arrayLength(&instance_data)) {
        return;
    }
    
    let instance = instance_data[instance_index];
    
    // Transform instance bounds center to world space
    let world_center = (instance.transform_matrix * vec4<f32>(instance.bounds_center, 1.0)).xyz;
    let world_radius = instance.bounds_radius; // Assume uniform scaling for simplicity
    
    // Calculate distance from camera
    let distance = length(world_center - culling_params.camera_position);
    
    // Distance culling - reject instances beyond far plane
    if (distance > culling_params.far_plane) {
        visibility_output[instance_index] = 0u;
        return;
    }
    
    // Frustum culling - test world-space bounding sphere against frustum planes
    let visible = test_sphere_frustum(world_center, world_radius, culling_params.frustum_planes);
    
    if (visible) {
        // Calculate LOD level and pack with visibility
        let lod_level = calculate_lod_level(distance);
        // Pack visibility (bit 0) and LOD level (bits 1-2) into u32
        visibility_output[instance_index] = 1u | (lod_level << 1u);
    } else {
        visibility_output[instance_index] = 0u;
    }
}
