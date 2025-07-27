# ADR-0010: Facade Crate Strategy

**Status:** Accepted  
**Date:** 2025-01-24  
**Authors:** Brady Jeong, Amp AI Agent  
**Reviewers:** Oracle  

## Context

The current 8-crate architecture (amp_core, amp_math, amp_engine, amp_physics, amp_render, amp_gameplay, config_core, gameplay_factory) provides strong modularity but creates complexity for developers who need simpler entry points. Two main use cases emerge:

1. **Foundation Layer**: Developers building new game engines who need core utilities and math libraries without Bevy dependencies
2. **Complete Game System**: Developers building games who want a single comprehensive interface to all systems

The proliferation of granular crates, while architecturally sound, creates ergonomic friction for common development scenarios.

## Decision

We will implement a **Facade Crate Strategy** with two new crates that provide simplified interfaces to our underlying modular architecture:

### amp_foundation
- **Purpose**: Foundation layer for engine builders
- **Dependencies**: amp_core + amp_math only (no Bevy dependencies)
- **Target Users**: Engine developers, library authors, non-Bevy projects
- **Re-exports**: Core utilities, error handling, math primitives, spatial indexing

### amp_game
- **Purpose**: Complete game development facade
- **Dependencies**: All core crates (amp_engine, amp_physics, amp_render, amp_gameplay, config_core, gameplay_factory)
- **Target Users**: Game developers wanting full functionality
- **Re-exports**: Complete game development API with sensible defaults

## Implementation Strategy

### Facade Structure
```rust
// amp_foundation/src/lib.rs
pub use amp_core::*;
pub use amp_math::*;

// Common prelude for engine builders
pub mod prelude {
    pub use amp_core::prelude::*;
    pub use amp_math::prelude::*;
}

// amp_game/src/lib.rs  
pub use amp_engine::*;
pub use amp_physics::*;
pub use amp_render::*;
pub use amp_gameplay::*;
pub use config_core::*;
pub use gameplay_factory::*;

// Game development prelude
pub mod prelude {
    pub use amp_engine::prelude::*;
    pub use amp_gameplay::prelude::*;
    // ... other preludes
}
```

### Documentation Strategy
- Facade crates get comprehensive documentation with examples
- Individual crates maintain technical documentation
- README.md highlights facade crates as primary entry points
- Examples demonstrate both facade and granular approaches

### Testing Strategy
- Facade crates test re-export correctness
- Integration tests validate facade interfaces
- Individual crate tests remain unchanged
- CI validates both facade and granular builds

## Rationale

### Benefits
1. **Simplified Entry Point**: New developers get clear starting points (foundation vs. game)
2. **Maintained Modularity**: Granular crates remain for advanced use cases
3. **Better Documentation**: Facade crates provide focused documentation
4. **Ecosystem Compatibility**: Foundation crate works in non-Bevy projects
5. **Gradual Migration**: Developers can start with facades, move to granular as needed

### Trade-offs
1. **Additional Maintenance**: Two more crates to maintain and document
2. **Version Coordination**: Facade versions must stay in sync with underlying crates
3. **Import Clarity**: Re-exports can obscure original module locations
4. **Build Time**: Additional compilation steps for facade crates

## Implementation Plan

### Phase 1: Infrastructure (Current)
- [x] Create amp_foundation and amp_game crate structures
- [x] Update Cargo.toml workspace members
- [ ] Implement basic re-exports and prelude modules
- [ ] Update xtask commands to work with facades

### Phase 2: Documentation Update
- [ ] Update AGENT.md to reflect facade strategy
- [ ] Update README.md to highlight facade crates
- [ ] Create comprehensive facade crate documentation
- [ ] Update examples to demonstrate facade usage

### Phase 3: CI Integration
- [ ] Update GitHub Actions to test facade crates
- [ ] Add facade-specific test jobs
- [ ] Validate documentation generation for facades
- [ ] Ensure performance benchmarks work through facades

### Phase 4: Validation
- [ ] Test all development commands through facades
- [ ] Validate cargo doc generation for both facade and granular approaches
- [ ] Ensure IDE support works correctly with re-exports
- [ ] Performance validation of facade approach vs. direct imports

## Migration Strategy

### For New Projects
- **Engine Builders**: Start with `amp_foundation`
- **Game Developers**: Start with `amp_game`
- **Advanced Users**: Use granular crates directly

### For Existing Code
- Current granular imports remain fully supported
- Optional migration to facade imports
- No breaking changes to existing APIs

## Documentation Impact

### AGENT.md Updates
- Add facade strategy explanation
- Update build commands to include facade testing
- Update architecture diagram to show facade layers
- Add guidance on when to use facades vs. granular crates

### README.md Updates
- Highlight facade crates as primary entry points
- Show both facade and granular usage examples
- Update architecture diagram
- Simplify quick start guide using facades

### CI Updates
- Add facade-specific build and test jobs
- Validate documentation builds for facades
- Test feature flag combinations through facades
- Ensure benchmarks work through facade interfaces

## Success Criteria

1. **Developer Experience**: New developers can start with single facade import
2. **Maintained Performance**: No performance regression from facade layer
3. **Complete Coverage**: All functionality accessible through appropriate facade
4. **Documentation Quality**: Clear examples and guidance for facade usage
5. **CI Validation**: All development workflows work with facade crates

## Related ADRs

- **ADR-0002**: Oracle-Guided Architecture (establishes modular foundation)
- **ADR-0007**: Strategic Shift Bevy Meta Crate (Bevy 0.16.1 alignment)
- **ADR-0008**: Oracle-Guided AAA Feature Restoration (current architecture)

This facade strategy builds upon our established modular architecture while providing ergonomic entry points for different developer personas.
