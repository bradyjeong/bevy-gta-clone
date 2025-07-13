# ADR-0009: GPU Culling Pipeline

**Status:** Accepted  
**Date:** 2025-01-07  
**Context:** Sprint 6 implementation completing Oracle's conditional approval requirements

## Context

Sprint 5 implemented CPU-based frustum and LOD culling with acceptable performance (2.5ms prepare+queue vs 4ms target). However, for AAA-scale scenarios with 100k+ entities, GPU compute-based culling offers significant performance advantages by offloading work from the CPU and enabling parallel processing of large instance batches.

Oracle guidance specified implementing GPU culling with proper feature flagging to maintain compatibility while enabling high-performance scenarios.

## Decision

We will implement a **feature-flagged GPU culling pipeline** with the following architecture:

### Feature Flag Strategy
- **Feature name:** `gpu_culling`
- **Default behavior:** OFF on non-desktop targets, ON for `--all-features`
- **Fallback:** CPU culling path remains fully functional when feature disabled
- **Target platforms:** Desktop with modern GPU compute support

### Pipeline Architecture

#### Stages
1. **ExtractStage:** Unchanged - extracts instances from ECS world
2. **CullStage:** New stage in RenderSet::Prepare (before prepare_batches when gpu_culling enabled)
3. **PrepareStage:** Modified to read GPU culling results or use CPU fallback

#### GPU Resources
- **Compute Shader:** Frustum + LOD culling per instance buffer  
- **Buffer Management:** Reuse `TransientBufferPool` for GPU resources allocation
- **Memory Strategy:** Allocate once per batch, double-buffer for frames-in-flight
- **Output Format:** `u32 bitset` SSBO + append buffer for draw-indirect counts

#### Performance Targets
- **Primary:** <0.25ms @ 400k instances (vs ~2.5ms CPU equivalent)
- **Memory:** Reuse existing buffer pool, no additional persistent allocations
- **Stall Mitigation:** Double-buffer + async mapping with frames-in-flight design

### Implementation Phases

#### Phase 1: Foundation (Sprint 6)
- ADR documentation and feature flag setup
- Basic compute shader infrastructure  
- CullStage integration in render pipeline
- Feature flag validation and testing

#### Phase 2: Core Implementation (Sprint 7)
- Complete compute shader implementation
- TransientBufferPool integration
- GPU→CPU result transfer pipeline
- Performance validation @ 400k instances

#### Phase 3: Optimization (Sprint 8)  
- Double-buffering for stall prevention
- Frames-in-flight optimization
- Performance regression testing in CI
- Documentation and examples

## Consequences

### Positive
- **Performance:** 10x+ speedup for large-scale culling scenarios
- **Scalability:** Enables 100k+ entity scenarios for AAA gameplay
- **Future-proof:** Foundation for other GPU compute optimizations
- **Compatibility:** CPU fallback maintains current functionality

### Negative  
- **Complexity:** Additional GPU pipeline management and error handling
- **Platform Support:** Limited to desktop platforms with compute shader support
- **Testing Overhead:** Need to test both CPU and GPU paths in CI
- **Memory Management:** More complex buffer lifecycle management

### Risks and Mitigations
- **Async Mapping Stalls:** Mitigated by double-buffering and frames-in-flight
- **Platform Compatibility:** Feature flag ensures graceful fallback
- **GPU Memory Pressure:** Reuse TransientBufferPool to limit allocations
- **Debugging Complexity:** Maintain CPU path for development and debugging

## Alternatives Considered

1. **CPU-only optimization:** Rejected - limited by single-thread performance ceiling
2. **Always-on GPU culling:** Rejected - compatibility and platform support concerns  
3. **Hybrid CPU+GPU:** Considered for future - current implementation focuses on pure approaches

## Implementation Details

### Feature Flag Configuration
```toml
[features]
default = []
gpu_culling = []
all-features = ["gpu_culling"]
```

### Compute Shader Pipeline
```
Input: Instance buffer (transforms, AABBs, metadata)
Process: Frustum culling + LOD calculation per instance  
Output: Visibility bitset + filtered instance count
Integration: prepare_batches reads GPU results for batch creation
```

### Buffer Management
- Reuse `TransientBufferPool` for GPU-side allocations
- Instance input buffer: Read-only access from GPU
- Culling result buffer: Write from GPU, read from CPU
- Staging buffer: Async mapping for GPU→CPU transfer

This architecture provides a scalable foundation for AAA-level rendering performance while maintaining the robustness and compatibility of the existing CPU-based system.
