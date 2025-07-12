// GPU frustum culling compute shader
// Oracle's specification: 256 workgroup size, <0.2ms for 10K instances
// Implements 6-plane frustum test with distance culling

// Workgroup size optimized for GPU utilization
const WORKGROUP_SIZE: u32 = 256u;

// Instance data (80 bytes aligned)
struct GpuInstance {
    model_matrix: mat4x4<f32>,    // 64 bytes
    center: vec3<f32>,            // 12 bytes  
    radius: f32,                  // 4 bytes
    batch_id: u32,                // 4 bytes
    // Total: 84 bytes, but aligned to 80 with padding in host
}

// Draw indirect parameters
struct DrawIndirect {
    vertex_count: u32,
    instance_count: atomic<u32>,  // Updated by shader
    first_vertex: u32,
    base_instance: u32,
}

// Frustum planes (6 planes x 4 components)
struct GpuFrustum {
    planes: array<vec4<f32>, 6>,
}

// Culling uniforms
struct CullingUniforms {
    frustum: GpuFrustum,
    max_distance: f32,
    instance_count: u32,
    batch_count: u32,
    enable_flags: u32,  // bit 0: frustum, bit 1: distance
}

// Batch offset data
struct GpuBatchOffset {
    instance_offset: u32,
    max_instances: u32,
    vertex_count: u32,
    padding: u32,
}

// Binding groups
@group(0) @binding(0) var<storage, read> instances: array<GpuInstance>;
@group(0) @binding(1) var<storage, read_write> indirect_draws: array<DrawIndirect>;
@group(0) @binding(2) var<storage, read> batch_offsets: array<GpuBatchOffset>;
@group(0) @binding(3) var<uniform> uniforms: CullingUniforms;

// Test sphere against frustum plane
fn sphere_plane_test(center: vec3<f32>, radius: f32, plane: vec4<f32>) -> bool {
    let distance = dot(plane.xyz, center) + plane.w;
    return distance > -radius;
}

// Full 6-plane frustum culling test
fn frustum_cull_sphere(center: vec3<f32>, radius: f32) -> bool {
    // Test against all 6 frustum planes
    for (var i = 0u; i < 6u; i = i + 1u) {
        if (!sphere_plane_test(center, radius, uniforms.frustum.planes[i])) {
            return false;  // Outside frustum
        }
    }
    return true;  // Inside frustum
}

// Distance culling test
fn distance_cull_sphere(center: vec3<f32>) -> bool {
    let distance = length(center);
    return distance <= uniforms.max_distance;
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let instance_index = global_id.x;
    
    // Bounds check
    if (instance_index >= uniforms.instance_count) {
        return;
    }
    
    let instance = instances[instance_index];
    
    // Extract world position from model matrix
    let world_position = instance.center;
    let radius = instance.radius;
    
    // Perform culling tests
    var visible = true;
    
    // Frustum culling (if enabled)
    if ((uniforms.enable_flags & 1u) != 0u) {
        visible = visible && frustum_cull_sphere(world_position, radius);
    }
    
    // Distance culling (if enabled)
    if ((uniforms.enable_flags & 2u) != 0u) {
        visible = visible && distance_cull_sphere(world_position);
    }
    
    // Update batch instance count atomically
    if (visible) {
        let batch_id = instance.batch_id;
        if (batch_id < uniforms.batch_count) {
            atomicAdd(&indirect_draws[batch_id].instance_count, 1u);
        }
    }
}
