# Oracle Consultation Log

This document tracks key Oracle consultations and their strategic impact on the project.

## Consultation Format

Each consultation should include:
- **Date**: When the consultation occurred
- **Context**: What problem or decision prompted the consultation
- **Key Insights**: Most important strategic guidance
- **Actions Taken**: How the guidance was implemented
- **ADR Reference**: Link to any resulting Architecture Decision Records

## Consultations

### 2025-07-13: Sprint 8 Completion and Sprint 9 Planning
**Context**: Sprint 8 "Integration Hardening & Performance Baseline" completed with Oracle conditional approval, need Sprint 9 strategy

**Key Insights**:
- Sprint 8 successfully completed all P1 deliverables (AAAPlugins rollout, GPU Culling Phase 2, xtask perf JSON, baseline CI)
- Oracle provided CONDITIONALLY APPROVED status with all requirements addressed
- Performance baseline established with 60 FPS gates, ready for optimization phase
- Sprint 9 should focus on final optimization and polish for AAA-grade release
- Real compute shader implementation needed for GPU Culling Phase 3
- Large-scale performance optimization requires 37× improvement for 100k entities

**Actions Taken**:
- Updated all documentation (AGENT.md, STRATEGIC_RESTORATION_PLAN.md, README.md) for Sprint 9
- Established Sprint 9 priorities: P1 Performance Optimization, P1 GPU Culling Phase 3, P2 Memory optimization
- Set performance targets: city_demo 60+ FPS @1080p, spawn_100k ≤3ms, gpu_culling ≤0.25ms
- Updated quality gates for 370+ tests and comprehensive performance validation

**ADR Reference**: Continuing with ADR-0009 (GPU Culling) and ADR-0010 (AAAPlugin Architecture)

### 2025-01-10: Version Consistency Strategy
**Context**: Need for consistent versioning strategy across all dependencies with Rust 2024 edition and Bevy 0.16.1 migration

**Key Insights**:
- Engine nucleus (Bevy) requires patch-locking (`bevy = "=0.16.1"`)
- Ecosystem sidekicks need patch-locking (`bevy_rapier3d = "=0.30.0"`)
- Rendering dependencies managed via [patch.crates-io] for exact wgpu/winit versions
- Mature crates use caret-semver (`serde = "^1"`, `anyhow = "^1.0"`)
- Single source of truth in [workspace.dependencies] with workspace inheritance
- Zero duplicate major versions in final cargo tree

**Actions Taken**:
- Updated Cargo.toml with Oracle's version-consistency strategy
- Added [patch.crates-io] section for wgpu/winit version control
- Updated Agent.md with version lock-in rules and bump playbook
- Established CI guards for version consistency

**ADR Reference**: Version strategy integrated into ADR-0007

### 2025-01-10: Strategic Shift to Bevy 0.16.1 Meta-Crate
**Context**: Current bevy_ecs 0.13 + micro-crate architecture creating ecosystem misalignment, test failures, development overhead

**Key Insights**:
- Current approach fights Bevy ecosystem, wastes time on solved problems (RON loaders, wgpu wrappers)
- Amp productivity comes from clear boundaries, not excessive crate count
- Cross-crate compilation overhead dominates CI time (40%+)
- Strategic 4-5 crate structure better than 6+ micro-crates
- Full Bevy 0.16.1 provides ecosystem leverage + future-proofing

**Actions Taken**:
- Created ADR-007 documenting strategic shift
- Updated Agent.md with Oracle's recommended structure
- Planned 10-14 day migration roadmap
- Target: amp_core + amp_math + amp_engine + amp_gameplay + amp_tools

**ADR Reference**: [ADR-0007](adr/0007-strategic-shift-bevy-meta-crate.md)

### 2025-01-07: Architecture Strategy Decision
**Context**: Choosing between clean restart, continued refactoring, or hybrid approach

**Key Insights**:
- Current codebase is 40% AAA implementation, 60% good architecture
- "Strangler-fig" hybrid approach optimal: extract proven systems, rebuild cleanly
- Multi-crate structure is correct direction but needs pruning
- Oracle estimates 2 months with disciplined execution

**Actions Taken**:
- Implemented 8-week extraction-based restart plan
- Created multi-crate workspace structure
- Established quality gates (no warnings, <60s compile, CI)

**ADR Reference**: [ADR-0002](adr/0002-oracle-guided-architecture.md)

### 2025-01-07: Week 1 Verification
**Context**: Verifying successful completion of foundation phase

**Key Insights**:
- Foundation is solid for Week 2 progression
- 78 tests passing with comprehensive coverage
- Minor polish items identified (coverage gate, publishing hygiene)
- Technical quality assessment: good algorithms, clean compilation

**Actions Taken**:
- Fixed documentation validation issues
- Implemented comprehensive documentation system
- Added automated validation to CI pipeline

**ADR Reference**: Documentation strategy captured in development workflows

### 2025-01-10: AAA-Restoration Master Plan
**Context**: ADR-0007 migration complete, need strategy for restoring professional game features from commit f430bc6 to current Bevy 0.16.1 architecture

**Key Insights**:
- **Migrate behavior, not code**: Re-implement features using Bevy 0.16.1 idioms, never drag legacy abstractions
- **Green bar guarantee**: All 122 existing tests must stay passing throughout restoration
- **Strategic 12-week roadmap**: Phased approach with clear deliverables and benchmarks
- **Professional focus**: Target AAA-level game development capabilities with proper tooling
- **Performance gates**: Maintain 60 FPS @1080p, <1GB memory, spawn_100k ≤3ms benchmarks

**Target Features for Restoration (f430bc6)**:
1. **12 RON Configuration System**: Data-driven game tuning with hot-reload
2. **Advanced Vehicle Physics**: Realistic drivetrain, suspension, tire friction curves
3. **Professional LOD System**: Distance-based mesh and material swapping
4. **GPU Culling & Batch Processing**: Compute shader optimization with multi-draw-indirect
5. **Modern ECS Patterns**: SystemSets, QueryData, sparse storage optimization
6. **Audio Graph**: Advanced audio system with bevy_kira_audio integration
7. **Performance Claims**: 300%+ FPS improvement, 60% memory reduction

**Actions Taken**:
- Created comprehensive [STRATEGIC_RESTORATION_PLAN.md](STRATEGIC_RESTORATION_PLAN.md)
- Updated Agent.md, README.md, IMPLEMENTATION_SUMMARY.md, CONTRIBUTING.md
- Established Week 0 preparation phase for branch creation and gap analysis
- Documented Oracle's 12-week timeline with weekly milestones and performance gates

**ADR Reference**: Future ADR-0008 to document restoration completion

---

## Usage Guidelines

### When to Consult Oracle
- Major architectural decisions
- Technology choice evaluation
- Performance optimization strategy
- Project milestone verification
- When stuck on complex technical problems

### When NOT to Consult Oracle
- Implementation details
- Minor bug fixes
- Routine development tasks
- Questions answered by existing documentation

### Documentation Process
1. **Consult Oracle** on strategic question
2. **Extract key insights** from response
3. **Document in this log** with context and actions
4. **Create ADR** for major architectural decisions
5. **Update AGENT.md** if workflow changes

## Benefits

- **Historical context** for future architectural decisions
- **Team alignment** on strategic direction
- **Decision rationale** preserved for new team members
- **Pattern recognition** for similar future problems
- **Oracle guidance** doesn't get lost in conversation history
