# ADR-0008: Oracle-Guided AAA Feature Restoration Strategy

## Status
Accepted - Sprint 0 Complete (2025-01-07)

## Context

Following the successful completion of ADR-0007's strategic architecture migration from legacy micro-crates to Bevy 0.16.1, the project now requires a comprehensive strategy to restore the advanced AAA game features that were present in the f430bc6 "REVOLUTIONARY TRANSFORMATION" commit.

The f430bc6 commit represented a significant milestone with:
- 76 files changed, 5043 additions, 2212 deletions
- 14 RON configuration files for data-driven gameplay
- Advanced vehicle physics systems
- LOD (Level of Detail) management
- Batch processing with 300%+ FPS performance claims
- GPU culling capabilities
- 60% memory reduction and 70% complexity reduction

However, these features were built on an outdated architecture that has since been superseded by Oracle's strategic shift to Bevy 0.16.1. The challenge is to restore these professional-grade features while maintaining the clean, modern architecture established in ADR-0007.

## Decision

We will implement Oracle's 12-Week AAA Feature Restoration Strategy, structured as a phased approach that migrates behavior (not code) from f430bc6 to the current Bevy 0.16.1 architecture.

### Strategic Approach
1. **Behavior Migration**: Re-implement features using Bevy 0.16.1 idioms rather than direct code porting
2. **Incremental Delivery**: 12-week timeline with weekly milestones and deliverable demos
3. **Performance Parity**: Target matching or exceeding f430bc6 performance claims
4. **Architecture Consistency**: Maintain Oracle's strategic 5-crate structure throughout

### Phase Structure
- **Phase 0**: Foundation & Preparation (Week 0)
- **Phase 1**: Domain Inventory & Configuration (Weeks 1-2)
- **Phase 2**: Core Systems & Physics (Weeks 3-4)
- **Phase 3**: Rendering & Performance (Weeks 5-6)
- **Phase 4**: Professional Integration (Weeks 7-8)
- **Phase 5**: Optimization & Release (Weeks 9-12)

### Key Deliverables
- Data-driven configuration system using Bevy Assets
- Professional vehicle physics with bevy_rapier3d integration
- LOD system with distance-based culling
- Batch processing using Bevy's RenderWorld phases
- GPU culling behind "gpu" feature flag
- Plugin architecture with AAAPlugins group
- Performance gates: 60 FPS @1080p, <1GB memory

## Consequences

### Positive
- **Professional Grade**: Restores AAA-level game development capabilities
- **Performance**: Targets 60+ FPS with advanced optimization techniques
- **Architecture**: Maintains clean Bevy 0.16.1 ecosystem alignment
- **Modularity**: Preserves strategic 5-crate structure for parallel development
- **Future-Proof**: Built on modern Bevy patterns and best practices

### Negative
- **Development Time**: 12-week commitment for full restoration
- **Complexity**: Significant feature set requiring careful coordination
- **Risk**: Performance claims from f430bc6 may not translate directly
- **Dependencies**: Requires deep integration with Bevy's rendering pipeline

### Mitigations
- **Weekly Checkpoints**: Prevent scope creep with regular deliverable demos
- **Performance Gates**: Maintain stable 60+ FPS with optimization improvements
- **Fallback Strategy**: Incremental feature delivery allows for scope adjustment
- **Documentation**: Comprehensive ADR and consultation record for decision tracking

## Sprint 0 Implementation (2025-01-07)

### Completed Tasks
- ✅ **Feature Restoration Branch**: Created `restore/f430bc6` branch
- ✅ **Reference Workspace**: Set up f430bc6 reference workspace using git worktree
- ✅ **Gap Analysis**: Created comprehensive GAP_REPORT.md with 72% architectural readiness
- ✅ **Architecture Analysis**: Documented f430bc6 features and current capabilities
- ✅ **Performance Baseline**: Established baseline metrics (83.53 FPS, 30.55ms 99th percentile)
- ✅ **Oracle Validation**: 90% Sprint 0 completion confirmed by Oracle

### Ready for Sprint 1
- **Target**: Data-Driven Foundation (Weeks 1-2)
- **Focus**: Port 14 RON configuration files to Bevy Assets
- **Performance**: Maintain 60+ FPS baseline with config hot-reloading
- **Next Phase**: Physics & Audio Integration (Weeks 3-4)

## References
- ADR-0007: Strategic Shift to Bevy 0.16.1 Meta-Crate
- STRATEGIC_RESTORATION_PLAN.md: Detailed implementation timeline
- GAP_REPORT.md: Comprehensive feature mapping and restoration strategy
- oracle-consultations.md: Oracle guidance and strategic approval
- f430bc6 commit: Reference implementation for feature restoration
