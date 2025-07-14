//! GPU culling compute shader - Oracle's Phase 3 WGSL specification
//!
//! Real compute shader implementation with frustum + distance LOD culling.
//! Oracle's specifications: workgroup size 64, bounding sphere early-out, atomic visibility tracking.

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

// Oracle's Phase 3 specification: workgroup size 64, one instance per thread
@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_id = global_id.x;
    
    // Bounds check - early return for out-of-range threads
    if (instance_id >= culling_params.instance_count) {
        return;
    }
    
    let instance = instance_data[instance_id];
    
    // Bounding sphere early-out optimization - Oracle's specification
    let aabb_center = (instance.aabb_min + instance.aabb_max) * 0.5;
    let aabb_extent = instance.aabb_max - instance.aabb_min;
    let sphere_radius = length(aabb_extent) * 0.5;
    let world_sphere_center = (instance.transform * vec4<f32>(aabb_center, 1.0)).xyz;
    
    // Distance LOD culling - Oracle's specification
    let distance_to_camera = distance(world_sphere_center, camera_data.camera_pos);
    let max_distance = culling_params.lod_distances[min(instance.lod_level, 3u)];
    
    var visible = distance_to_camera <= (max_distance + sphere_radius);
    
    // Frustum culling with bounding sphere test - Oracle's specification
    if (visible) {
        // Test sphere against all 6 frustum planes
        for (var i = 0u; i < 6u; i = i + 1u) {
            let plane = camera_data.frustum_planes[i];
            let distance_to_plane = dot(plane.xyz, world_sphere_center) + plane.w;
            
            // Oracle's early-out: if sphere is completely outside this plane, cull it
            if (distance_to_plane < -sphere_radius) {
                visible = false;
                break;
            }
        }
    }
    
    // Atomic visibility tracking - Oracle's specification
    if (visible) {
        // Set bit in visibility bitset atomically
        let word_index = instance_id / 32u;
        let bit_index = instance_id % 32u;
        atomicOr(&visibility_bitset[word_index], 1u << bit_index);
        
        // Increment visible count atomically
        atomicAdd(&visible_count[0], 1u);
    }
}
