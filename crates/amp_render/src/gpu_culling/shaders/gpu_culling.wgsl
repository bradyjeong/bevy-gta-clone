//! Optimized GPU culling compute shader - Oracle's scalability solution
//!
//! High-performance frustum + distance culling for 100K+ instances.
//! Features: early-out optimizations, hierarchical culling, memory coalescing.

@group(0) @binding(0) var<uniform> camera_data: CameraData;
@group(0) @binding(1) var<storage, read> instance_data: array<InstanceData>;
@group(0) @binding(2) var<uniform> culling_params: CullingParams;
@group(0) @binding(3) var<storage, read_write> visibility_bitset: array<u32>;
@group(0) @binding(4) var<storage, read_write> visible_count: array<atomic<u32>>;
@group(0) @binding(5) var<storage, read_write> indirect_draw_commands: array<IndirectDrawCommand>;

struct CameraData {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    near_plane: f32,
    frustum_planes: array<vec4<f32>, 6>,
    far_plane: f32,
    fov: f32,
    aspect_ratio: f32,
    _padding: f32,
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
    performance_tier: u32, // 0=low-end, 1=mid-range, 2=high-end
    batch_size: u32,
    hierarchical_culling: u32,
    early_z_enabled: u32,
    _padding: u32,
};

struct IndirectDrawCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
};

// Shared memory for workgroup-level optimizations
var<workgroup> workgroup_visibility: array<u32, 64>; // One per thread in workgroup
var<workgroup> workgroup_visible_count: atomic<u32>;

// Oracle's optimized workgroup size for 100K+ instances
@compute @workgroup_size(64, 1, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let instance_id = global_id.x;
    let local_index = local_id.x;
    
    // Initialize workgroup shared memory
    if (local_index == 0u) {
        atomicStore(&workgroup_visible_count, 0u);
    }
    workgroupBarrier();
    
    var visible = false;
    
    // Bounds check with early return for out-of-range threads
    if (instance_id < culling_params.instance_count) {
        let instance = instance_data[instance_id];
        
        // Hierarchical culling optimization for large datasets
        if (culling_params.hierarchical_culling > 0u) {
            visible = hierarchical_cull(instance);
        } else {
            visible = standard_cull(instance);
        }
    }
    
    // Store local result
    workgroup_visibility[local_index] = select(0u, 1u, visible);
    workgroupBarrier();
    
    // Workgroup-level reduction to minimize atomic contention
    if (local_index == 0u) {
        var workgroup_total = 0u;
        for (var i = 0u; i < 64u; i = i + 1u) {
            workgroup_total += workgroup_visibility[i];
        }
        
        if (workgroup_total > 0u) {
            atomicAdd(&visible_count[0], workgroup_total);
        }
    }
    
    // Global visibility tracking
    if (visible && instance_id < culling_params.instance_count) {
        // Set bit in visibility bitset with reduced atomic contention
        let word_index = instance_id / 32u;
        let bit_index = instance_id % 32u;
        atomicOr(&visibility_bitset[word_index], 1u << bit_index);
    }
}

// Standard culling for moderate instance counts (<100K)
fn standard_cull(instance: InstanceData) -> bool {
    // Bounding sphere calculation with optimized math
    let aabb_center = (instance.aabb_min + instance.aabb_max) * 0.5;
    let aabb_extent = instance.aabb_max - instance.aabb_min;
    let sphere_radius = length(aabb_extent) * 0.5;
    let world_sphere_center = (instance.transform * vec4<f32>(aabb_center, 1.0)).xyz;
    
    // Early distance rejection (cheapest test first)
    let distance_to_camera = distance(world_sphere_center, camera_data.camera_pos);
    let max_distance = culling_params.lod_distances[min(instance.lod_level, 3u)];
    
    if (distance_to_camera > (max_distance + sphere_radius)) {
        return false;
    }
    
    // Frustum culling with early-out optimization
    return sphere_in_frustum(world_sphere_center, sphere_radius);
}

// Hierarchical culling for massive instance counts (100K+)
fn hierarchical_cull(instance: InstanceData) -> bool {
    // Quick AABB bounds check against camera frustum
    let world_aabb_min = (instance.transform * vec4<f32>(instance.aabb_min, 1.0)).xyz;
    let world_aabb_max = (instance.transform * vec4<f32>(instance.aabb_max, 1.0)).xyz;
    
    // Conservative AABB vs frustum test (faster than sphere)
    if (!aabb_in_frustum(world_aabb_min, world_aabb_max)) {
        return false;
    }
    
    // LOD-based distance culling with early rejection
    let center = (world_aabb_min + world_aabb_max) * 0.5;
    let distance = distance(center, camera_data.camera_pos);
    let lod_distance = culling_params.lod_distances[min(instance.lod_level, 3u)];
    
    if (distance > lod_distance) {
        return false;
    }
    
    // Detailed sphere test only for objects that passed AABB test
    let sphere_radius = length(world_aabb_max - world_aabb_min) * 0.5;
    return sphere_in_frustum(center, sphere_radius);
}

// Optimized sphere-frustum intersection test
fn sphere_in_frustum(center: vec3<f32>, radius: f32) -> bool {
    // Test sphere against all 6 frustum planes with early-out
    for (var i = 0u; i < 6u; i = i + 1u) {
        let plane = camera_data.frustum_planes[i];
        let distance_to_plane = dot(plane.xyz, center) + plane.w;
        
        // Early-out: sphere completely outside this plane
        if (distance_to_plane < -radius) {
            return false;
        }
    }
    return true;
}

// Conservative AABB-frustum intersection test
fn aabb_in_frustum(aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    // Test AABB corners against frustum planes
    for (var i = 0u; i < 6u; i = i + 1u) {
        let plane = camera_data.frustum_planes[i];
        let normal = plane.xyz;
        let d = plane.w;
        
        // Find the corner of AABB closest to the plane
        let closest_corner = select(aabb_min, aabb_max, normal > vec3<f32>(0.0));
        
        // If closest corner is outside plane, AABB is completely outside
        if (dot(normal, closest_corner) + d < 0.0) {
            return false;
        }
    }
    return true;
}

// Performance monitoring entry point for benchmarking
@compute @workgroup_size(1, 1, 1)
fn performance_monitor(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Reset counters for new frame
    if (global_id.x == 0u) {
        atomicStore(&visible_count[0], 0u);
        
        // Clear visibility bitset
        let bitset_size = (culling_params.instance_count + 31u) / 32u;
        for (var i = 0u; i < bitset_size; i = i + 1u) {
            visibility_bitset[i] = 0u;
        }
    }
}
