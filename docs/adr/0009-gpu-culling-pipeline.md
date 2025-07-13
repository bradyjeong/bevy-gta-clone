# ADR-0009: GPU Culling Pipeline

## Status
Accepted

## Context

The current CPU-based culling system (`amp_render::culling`) provides basic frustum and distance culling but becomes a bottleneck when handling large numbers of instances (>100k entities). Oracle's P3a guidance requires implementing GPU-based culling to achieve AAA performance targets of <0.25ms @ 400k instances.

The existing implementation uses CPU frustum calculations in `PostUpdate` phase, limiting scalability and preventing true parallel processing of large batches. With the recent `TransientBufferPool` infrastructure and established render pipeline phases, we can now implement GPU compute shader culling while maintaining feature compatibility.

Key requirements from Oracle guidance:
- Feature flag `gpu_culling` - default off on non-desktop targets, on for `--all-features`
- Compute shader for frustum + LOD per instance buffer with fallback to CPU path
- Use `TransientBufferPool` for GPU resources allocated once per batch
- Pipeline stages: ExtractStage unchanged, new CullStage (RenderSet::Prepare), prepare_batches reads GPU output
- Mitigate async mapping stalls with double-buffer & frames-in-flight
- Target performance: <0.25ms @ 400k instances

## Decision

We will implement a feature-flagged GPU culling pipeline that:

1. **Feature Flag Architecture**: `gpu_culling` feature flag controls GPU vs CPU culling path
   - Default off on non-desktop targets (mobile, WebGL) due to compute shader limitations
   - Default on for `--all-features` builds (development, benchmarks)
   - Runtime detection for platforms without compute shader support

2. **Compute Shader Pipeline**: GPU culling via compute shaders with structured approach
   - Input: Instance buffer with transform + bounding info
   - Shader: Frustum + distance + LOD culling in parallel
   - Output: Visibility bitset + instance indices for visible objects
   - Integration: `TransientBufferPool` for efficient GPU resource management

3. **Pipeline Integration**: New CullStage in render pipeline without disrupting existing systems
   - ExtractStage: Unchanged - existing instance extraction logic
   - CullStage: New stage in RenderSet::Prepare for GPU/CPU culling dispatch
   - PrepareBatches: Reads culling results to build final render batches
   - Fallback: Graceful degradation to CPU culling when GPU unavailable

4. **Performance Optimization**: Address async mapping stalls with frame pipelining
   - Double-buffered GPU resources to avoid stalls
   - Frames-in-flight tracking for overlap of compute/render phases
   - Early termination for empty batches to minimize GPU overhead

5. **Compatibility**: Maintain existing CPU culling API while adding GPU acceleration
   - Existing `CullingConfig`, `Cullable` components remain unchanged
   - Same frustum extraction and distance culling logic
   - Feature flag transparent to game code

## Consequences

### Positive
- **Performance**: >10x culling performance improvement for large scenes (400k+ instances)
- **Scalability**: GPU parallel processing enables larger world sizes
- **Future-Proof**: Foundation for advanced GPU culling (occlusion, cluster culling)
- **Compatibility**: Fallback ensures support across all target platforms
- **Infrastructure**: Leverages existing `TransientBufferPool` and render phases

### Negative
- **Complexity**: Additional GPU resource management and shader compilation
- **Platform Dependency**: Compute shader support varies across targets
- **Memory**: GPU buffer allocation overhead for small scenes
- **Debugging**: GPU culling harder to debug than CPU path
- **Feature Flag**: Additional testing matrix for GPU vs CPU code paths

### Migration Path
- Phase 1: Implement feature flag and basic compute shader infrastructure
- Phase 2: Add GPU culling logic with `TransientBufferPool` integration  
- Phase 3: Optimize with double-buffering and frame pipelining
- Phase 4: Performance validation and benchmark integration

### Testing Strategy
- Unit tests for both GPU and CPU culling paths
- Feature flag tests ensuring proper fallback behavior
- Performance benchmarks comparing GPU vs CPU at various scales
- Platform compatibility tests across desktop/mobile/WebGL targets
