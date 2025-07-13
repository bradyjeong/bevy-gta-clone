//! GPU culling compute shader for frustum and LOD culling
//!
//! This shader performs parallel culling of instances using frustum and distance checks.
//! Workgroup size must match the Rust configuration (default: 64).

@group(0) @binding(0) var<uniform> camera_data: CameraData;
@group(0) @binding(1) var<storage, read> instance_data: array<InstanceData>;
@group(0) @binding(2) var<uniform> culling_params: CullingParams;
@group(0) @binding(3) var<storage, read_write> visibility_bitset: array<u32>;
@group(0) @binding(4) var<storage, read_write> visible_count: array<atomic<u32>>;

struct CameraData {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
    frustum_planes: array<vec4<f32>, 6>,
};

struct InstanceData {
    transform: mat4x4<f32>,
    aabb_min: vec3<f32>,
    lod_level: u32,
    aabb_max: vec3<f32>,
    entity_id: u32,
};

struct CullingParams {
    lod_distances: array<f32, 4>,
    instance_count: u32,
    debug_enabled: u32,
    _padding: array<u32, 2>,
};

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_id = global_id.x;
    
    // Bounds check
    if (instance_id >= culling_params.instance_count) {
        return;
    }
    
    let instance = instance_data[instance_id];
    let world_pos = instance.transform[3].xyz;
    
    // Distance culling
    let distance_to_camera = distance(world_pos, camera_data.camera_pos);
    let max_distance = culling_params.lod_distances[instance.lod_level];
    
    var visible = distance_to_camera <= max_distance;
    
    // Frustum culling (sphere-plane test)
    if (visible) {
        let sphere_radius = length(instance.aabb_max - instance.aabb_min) * 0.5;
        let sphere_center = (instance.aabb_min + instance.aabb_max) * 0.5;
        let world_sphere_center = (instance.transform * vec4<f32>(sphere_center, 1.0)).xyz;
        
        // Test against all 6 frustum planes
        for (var i = 0u; i < 6u; i = i + 1u) {
            let plane = camera_data.frustum_planes[i];
            let distance_to_plane = dot(plane.xyz, world_sphere_center) + plane.w;
            
            // If sphere is completely outside this plane, it's not visible
            if (distance_to_plane < -sphere_radius) {
                visible = false;
                break;
            }
        }
    }
    
    // Update visibility bitset
    let word_index = instance_id / 32u;
    let bit_index = instance_id % 32u;
    
    if (visible) {
        // Set bit atomically
        atomicOr(&visibility_bitset[word_index], 1u << bit_index);
        // Increment visible count
        atomicAdd(&visible_count[0], 1u);
    }
    
    // Debug output (if enabled)
    if (culling_params.debug_enabled != 0u && instance_id < 10u) {
        // In a real implementation, this would write to a debug buffer
        // For now, this is just a placeholder
    }
}
