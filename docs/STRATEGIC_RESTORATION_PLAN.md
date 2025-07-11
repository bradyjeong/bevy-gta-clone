# STRATEGIC RESTORATION PLAN: f430bc6 → Bevy 0.16.1
## Oracle-Guided AAA Feature Restoration Strategy

**Status:** ACTIVE - Post ADR-0007 Implementation  
**Objective:** Restore f430bc6 "REVOLUTIONARY TRANSFORMATION" features to current Bevy 0.16.1 architecture  
**Timeline:** 12-week restoration plan with weekly milestones  
**Oracle Consultation:** Complete - Strategic roadmap approved  

---

## CURRENT STATUS CONFIRMATION

**✅ ADR-0007 COMPLETE** - Oracle-Guided Architecture Migration  
- **Migration Complete**: Legacy micro-crates → Strategic 6-crate Bevy 0.16.1 architecture
- **Current Architecture**: amp_core, amp_math, amp_engine, amp_physics, config_core, gameplay_factory  
- **Test Status**: 180+ tests passing across all crates
- **Version Consistency**: Oracle's definitive strategy implemented with automated guards
- **Foundation**: Clean, professional Bevy 0.16.1 codebase ready for feature restoration

**✅ SPRINT 2 COMPLETE** - Vehicle Physics Foundation  
- **Objective**: Professional-grade vehicle physics system implemented
- **Status**: All Sprint 2 deliverables delivered and Oracle-approved
- **Key Achievements**: amp_physics crate, Rapier3D integration, city_demo_baseline, 60+ FPS stable
- **Quality Gates**: 180+ unit tests + 25+ integration tests passing
- **Foundation**: Vehicle physics infrastructure ready for gameplay integration

**f430bc6 ANALYSIS COMPLETE** - Target Feature Set Identified  
- **Scope**: 76 files changed, 5043 additions, 2212 deletions
- **Core Features**: 14 RON config files, advanced vehicle physics, LOD systems, batch processing, GPU culling
- **Performance Claims**: Significant FPS improvements, memory optimization, code complexity reduction
- **Architecture**: Modern ECS patterns, professional plugin architecture, data-driven configuration

---

## ORACLE'S 12-WEEK RESTORATION ROADMAP

### Phase 0: Foundation & Preparation (Week 0)
**Objective:** Set up restoration infrastructure and baseline metrics

**Tasks:**
- Create feature-restore branch `restore/f430bc6`
- Set up f430bc6 reference workspace for diffing
- Establish baseline performance metrics with simple benchmark scene
- Document current vs. target architecture differences

**Deliverables:**
- Baseline performance numbers
- Feature gap analysis report
- Restoration branch ready for development

### Phase 1: Domain Inventory & Configuration (Weeks 1-2)

#### Week 1: Strategic Analysis
**Objective:** Complete feature inventory and specification freeze

**Tasks:**
- Code archaeology: catalog removed vs. current modules
- Map each of 14 RON files to owning systems
- Write comprehensive functional acceptance criteria
- Define public API contracts per crate

**Deliverables:**
- Feature matrix spreadsheet
- Specification freeze sign-off
- API contract documentation

#### Week 2: Configuration Subsystem
**Objective:** Implement data-driven configuration foundation

**Tasks:**
- Create config_core::assets module with RonAssetPlugin
- Port 14 *.ron files with typed structs using serde + Reflect
- Implement ConfigHandle<T> resource pattern
- Add hot-reload integration tests

**Deliverables:**
- Working RON configuration system
- Hot-reload capability
- Round-trip serialization tests

**Risk Mitigation:** Compile-time reflection tests to prevent serialization drift

### Phase 2: Vehicle Physics Foundation (Weeks 3-4) ✅ COMPLETE

#### Week 3: Core Physics Infrastructure ✅ COMPLETE
**Objective:** Establish vehicle physics foundations

**Tasks:**
- ✅ Create amp_physics crate with comprehensive vehicle physics
- ✅ Port vehicle data structs: Wheel, Engine, Gearbox, Suspension, DamageModel
- ✅ Write Bevy 0.16.1 wrappers for bevy_rapier3d integration
- ✅ Implement adapter layer for legacy API compatibility

**Deliverables:**
- ✅ Vehicle physics data structures
- ✅ Bevy-rapier integration layer
- ✅ Unit tests with kinematic rigs
- ✅ Vehicle "rocker-bogie" demo (city_demo_baseline)

**Feature Flag:** `rapier3d_030` (implemented and working)

#### Week 4: Vehicle Control & Input ✅ COMPLETE
**Objective:** Complete vehicle control systems

**Tasks:**
- ✅ Rebuild TorqueCurve, PID controllers, anti-roll bars
- ✅ Implement Input<VehicleAction> mapping
- ✅ Split ECS commands into pre-physics & post-physics sets
- ✅ Add golden-master performance tests

**Deliverables:**
- ✅ Full vehicle control system
- ✅ Input handling integration
- ✅ Performance validation (≤1ms/vehicle)
- ✅ 0-100 km/h acceleration within legacy ±5%

### Phase 3: Core Gameplay & Physics Integration (Weeks 5-6) 🎯 SPRINT 3 ACTIVE

#### Week 5: Physics Integration & Gameplay Systems
**Objective:** Port vehicle physics to gameplay crate and integrate with core systems

**Tasks:**
- 🔄 Port vehicle physics from amp_physics to amp_gameplay crate
- 🔄 Integrate bevy_rapier3d 0.30 with gameplay systems
- 🔄 Create unified physics/gameplay component architecture
- 🔄 Implement gameplay-specific vehicle behaviors

**Deliverables:**
- Unified physics/gameplay architecture
- Vehicle physics in amp_gameplay crate
- Seamless vehicle/world physics interaction
- Enhanced gameplay systems

**Target Performance:** Maintain 60+ FPS with integrated physics

#### Week 6: Advanced Audio System
**Objective:** Implement advanced audio system with bevy_kira_audio

**Tasks:**
- 🔄 Create amp_gameplay::audio module with bevy_kira_audio integration
- 🔄 Implement spatial audio for vehicles and world events
- 🔄 Create audio configuration system with RON configs
- 🔄 Add audio debugging and visualization tools

**Deliverables:**
- Advanced audio system
- Spatial audio implementation
- Audio configuration management
- Enhanced city_demo with audio integration

**Target Performance:** <1.5ms combined physics/audio update time

### Phase 4: Parallel Processing & ECS Modernization (Weeks 7-8)

#### Week 7: CPU Batch Processing
**Objective:** Implement parallel job graph system

**Tasks:**
- Create amp_core::job_graph with cross-beam scoped threads
- Convert AI, nav, FX systems to ScheduleLabel::ParallelJob
- Configure ExecutorKind::MultiThreaded with task_pool settings
- Add determinism toggle for testing

**Deliverables:**
- Parallel job graph system
- Multi-threaded system execution
- Deterministic testing capability
- Data race prevention (Miri + ThreadSan validation)

#### Week 8: Modern ECS Refactoring
**Objective:** Complete ECS modernization sweep

**Tasks:**
- Replace exclusive-world access with SystemParams
- Optimize component storage (sparse sets → Table where appropriate)
- Implement QueryData for filter reuse
- Group systems into SystemSet::on_update(GameState::InGame)

**Deliverables:**
- Fully modernized ECS patterns
- Zero warnings on RUSTFLAGS="-D rust_2018_idioms"
- Performance-optimized component storage
- Clean system organization

### Phase 5: Optimization & Quality Assurance (Weeks 9-10)

#### Week 9: Performance & Memory Optimization
**Objective:** Achieve target performance metrics

**Tasks:**
- Profiling pass with tracy + bevy_tracy_extras
- GPU optimization: bind group churn, instance buffer resizing
- Physics optimization: broadphase layer masks, sleeping body cleanup
- Memory budget validation

**Deliverables:**
- Performance hotspot elimination (>5% frame time)
- Memory budget: ≤800 MB peak on benchmark map
- GPU rendering optimization
- Physics performance tuning

#### Week 10: QA & Regression Testing
**Objective:** Ensure stability and regression prevention

**Tasks:**
- Enable feature flags by default in nightly CI
- Extend test suites: physics golden-master, render frame hashes
- Implement 8-hour soak testing with 20 vehicles
- Daily defect triage and resolution

**Deliverables:**
- Comprehensive test coverage
- Automated regression prevention
- Stability validation
- Performance regression guards

### Phase 6: Polish & Release (Weeks 11-12)

#### Week 11: Playtest & Polish
**Objective:** Prepare for alpha release

**Tasks:**
- Internal alpha build with telemetry collection
- Polish tasks: sound hooks, particle spawns, damage VFX
- Documentation: mdBook sections per restored feature
- Public API examples and tutorials

**Deliverables:**
- Alpha build ready
- Comprehensive documentation
- Polished user experience
- Performance telemetry system

#### Week 12: Release & Retrospective
**Objective:** Complete restoration and capture lessons

**Tasks:**
- Version crates: 0.16.1-aa1.0.0
- Tag mainline and cut changelog
- Retrospective meeting and documentation
- Create ADR-0015: "f430bc6 Restore"

**Deliverables:**
- Released version with full f430bc6 features
- Comprehensive changelog
- Lessons learned documentation
- Strategic retrospective (ADR-0015)

---

## STRATEGIC PRINCIPLES

### A. Vertical Slices Approach
Re-enable one gameplay slice at a time, porting only the code each slice touches. Avoid monolithic rewrites.

### B. Δ-Coverage Testing
Every week add regression/unit/performance tests that lock in restored behavior. No feature without tests.

### C. Crate Boundary Stability
Maintain current 5-crate architecture. New code must live inside feature-specific sub-modules (e.g., amp_gameplay::vehicle).

### D. Two-Phase Migration
For systems with heavy API churn: 1) Adapter shims, 2) Full refactor. Minimize disruption.

### E. Performance Gates
Maintain fps ≥60 on reference scene with optimization improvements where possible. No test >200ms. Continuous performance monitoring.

### F. Integration Windows
Maintain compile green every Friday. Break-glass changes only behind feature flags.

---

## RESOURCE ALLOCATION & RISK MANAGEMENT

### Team Structure
- **4 Engineers**: Physics, Rendering, ECS/Infrastructure, Tooling/Configs
- **1 QA**: Automated merge rules and testing
- **0.3 DevOps**: CI/CD and deployment automation

### Risk Mitigation
- **15% Contingency Buffer**: Built into weeks 8-11
- **Red Flag Metrics**: CI >10% fail rate, frame time regression >20%, memory leak >50MB/min
- **Weekly Checkpoints**: Oracle consultation for strategic decisions
- **Feature Flags**: All new features behind flags until stable

### Success Metrics
- **Functional**: All f430bc6 features restored and working
- **Performance**: Maintain stable 60+ FPS with optimization improvements
- **Quality**: 122+ tests passing, zero regressions
- **Architecture**: Maintain Oracle-guided 5-crate structure

---

## DELIVERY CHECKLIST

### Phase Completions
- [ ] **W-1**: Spec matrix and feature inventory complete
- [ ] **W-2**: Config subsystem with hot-reload working
- [ ] **W-4**: Vehicles driving with full physics
- [ ] **W-6**: LOD + GPU culling active and performant
- [ ] **W-7**: Job graph parallelism implemented
- [ ] **W-10**: All legacy tests ported + new ones passing
- [ ] **W-12**: Alpha polished and released

### Quality Gates
- [ ] **Performance**: Stable 60+ FPS with optimization improvements
- [ ] **Memory**: Optimized memory usage and allocation patterns
- [ ] **Complexity**: Reduced code complexity through clean architecture
- [ ] **Tests**: 122+ tests passing with full coverage
- [ ] **Documentation**: Complete API documentation and examples

---

## NEXT STEPS

1. **Immediate**: Create `restore/f430bc6` branch and begin Week 0 preparation
2. **Week 1**: Begin strategic analysis and feature inventory
3. **Weekly**: Oracle consultation for strategic guidance and course correction
4. **Continuous**: Monitor performance gates and quality metrics

This roadmap provides a clear path to restore all f430bc6 AAA game features while maintaining the clean, Oracle-guided Bevy 0.16.1 architecture achieved through ADR-0007.
