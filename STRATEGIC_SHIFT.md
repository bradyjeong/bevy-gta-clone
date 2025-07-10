# STRATEGIC SHIFT: Oracle-Guided Architecture Change

## DECISION
**Moving from bevy_ecs 0.13 + micro-crates to Bevy 0.16.1 + strategic modularity**

## WHY
Current architecture fights Bevy ecosystem, creates unnecessary complexity:
- ❌ Reinventing RON loaders, wgpu wrappers, asset pipelines  
- ❌ Cross-crate compilation overhead dominates CI time (40%+)
- ❌ Test failures from mocked ECS instead of integrated Bevy App
- ❌ Future Bevy upgrades require multi-month re-integration

## CURRENT ARCHITECTURE
```
├─ crates/
│   ├─ amp_core/          # Pure Rust utilities, error handling (no Bevy deps)
│   ├─ amp_math/          # glam re-exports, Morton, AABB (no Bevy deps)  
│   ├─ amp_engine/        # Bevy 0.16.1 dependency, engine plugins
│   ├─ config_core/       # Configuration loading and management
│   ├─ gameplay_factory/  # Entity factory for prefab-based systems
│   └─ tools/xtask/       # Build pipeline helpers
```

## MIGRATION PLAN (10-14 DAYS)
1. **Days 1-2**: Branch & lock Bevy 0.16.1 versions (keep Rust 2021 edition)
2. **Days 3-4**: Consolidate amp_spatial, amp_gpu, amp_world → amp_engine  
3. **Days 5-6**: Replace custom RON loader with Bevy asset pipeline
4. **Days 7-9**: Rewrite tests to use App::new().add_plugins(DefaultPlugins)
5. **Days 10-14**: Documentation, stabilization, playtest

## EXPECTED BENEFITS
- ✅ **Ecosystem Leverage**: Full Bevy plugins, examples, community support
- ✅ **Compile Performance**: 30-40% faster builds  
- ✅ **Test Reliability**: Integrated App-based testing
- ✅ **Future-Proofing**: Bevy 0.17+ upgrades = cargo upgrade + minor fixes
- ✅ **Amp Productivity**: Clear boundaries without micro-crate coordination tax

## STATUS
- ✅ Oracle consultation complete
- ✅ ADR-007 created  
- ✅ Agent.md updated
- ✅ Documentation aligned
- ✅ Phase 0-4 implementation completed
- ✅ New crate structure active: amp_core, amp_math, amp_engine, config_core, gameplay_factory
- ✅ Asset pipeline integrated with hot-reload
- ✅ Phase 5B: Documentation updates completed
- ✅ **STRATEGIC SHIFT MIGRATION COMPLETED**

## 🎯 NEXT PHASE: AAA-RESTORATION

**Oracle's 12-Week Master Plan**: Restore professional game features from commit f430bc6 to current Bevy 0.16.1 architecture.

### Restoration Strategy
- **Migrate behavior, not code**: Re-implement features using Bevy 0.16.1 idioms
- **Preserve architecture**: Maintain 5-crate strategic structure
- **Green bar guarantee**: Keep all 122 existing tests passing
- **Professional focus**: Target AAA-level game development capabilities

### Sprint 0 (Current)
- [ ] Create `restore/f430bc6` branch
- [ ] Set up git worktree for f430bc6 reference
- [ ] Create GAP_REPORT.md mapping features to current architecture
- [ ] Update all documentation for AAA development focus

### Target Features for Restoration
1. **12 RON Configuration System**: Data-driven game settings
2. **Unified Entity Factory**: Single-source prefab system
3. **Advanced Vehicle Physics**: Realistic movement with supercar effects
4. **Professional Audio Graph**: Advanced audio system integration
5. **GPU-Ready Culling**: Compute shader optimization (300%+ performance)
6. **Distance-Based LOD**: Professional quality management
7. **Batch Processing**: Modern parallel job system with Bevy RenderWorld

See [Agent.md](Agent.md) for complete roadmap and Oracle consultation details.
