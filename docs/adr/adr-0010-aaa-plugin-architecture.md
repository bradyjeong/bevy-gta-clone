# ADR-0010: AAAPlugin Architecture Design

## Status
Accepted - Sprint 7 Implementation (2025-01-13)

## Context

Following the successful completion of ADR-0008's AAA Feature Restoration Strategy through Sprint 6, the project now requires a professional plugin architecture to integrate the various subsystems (physics, audio, rendering, configuration) into a cohesive, extensible framework.

The current architecture has proven subsystems distributed across multiple crates:
- **amp_physics**: Vehicle physics and collision detection
- **amp_render**: GPU culling and batch processing
- **amp_gameplay**: Audio systems and entity management
- **config_core**: Configuration management and hot-reloading
- **gameplay_factory**: Entity spawning and factory patterns

However, these subsystems lack a unified integration strategy that allows for:
1. **Professional Plugin Architecture**: Clean separation of concerns with extensible plugin system
2. **User Extension**: Ability for users to add custom plugins alongside built-in systems
3. **Bevy Integration**: Seamless integration with Bevy's Plugin system and App builder
4. **Configuration Management**: Centralized configuration of all subsystems through unified interface

Oracle's guidance emphasizes the need for a plugin architecture that enables "professional integration" while maintaining the clean boundaries established in previous ADRs.

## Decision

We will implement the **AAAPlugin Architecture** consisting of:

### 1. AAAPlugin Trait
```rust
pub trait AAAPlugin: Send + Sync + 'static {
    fn build(&self, app: &mut App) -> Result<()>;
    fn stage(&self) -> PluginStage { PluginStage::Default }
}
```

Key characteristics:
- **Lifecycle Management**: `build()` method for App integration
- **Staging Control**: `stage()` method for initialization ordering
- **Error Handling**: Result<()> return type for graceful failure
- **Thread Safety**: Send + Sync bounds for parallel plugin loading

### 2. AAAPlugins PluginGroup
```rust
pub struct AAAPlugins {
    physics: Box<dyn AAAPlugin>,
    audio: Box<dyn AAAPlugin>,
    render: Box<dyn AAAPlugin>,
    config: Box<dyn AAAPlugin>,
    custom: Vec<Box<dyn AAAPlugin>>,
}
```

Features:
- **Built-in Subsystems**: Physics, audio, render, config plugins included by default
- **User Extension**: `add_plugin()` method for custom plugin registration
- **Bevy Integration**: Implements Bevy's `PluginGroup` trait for seamless App integration
- **Configuration**: Unified configuration interface across all subsystems

### 3. Plugin Stage System
```rust
pub enum PluginStage {
    PreStartup,   // Config and core systems
    Startup,      // Physics and audio initialization
    Default,      // Rendering and gameplay systems
    PostStartup,  // User extensions and finalizations
}
```

### 4. Integration with amp_engine
- **Central Location**: All plugin definitions and AAAPlugins in amp_engine crate
- **Dependency Management**: amp_engine depends on all subsystem crates
- **Public API**: Single entry point for users via `amp_engine::AAAPlugins`
- **Example Integration**: Updated city_demo to use AAAPlugins instead of manual plugin registration

## Consequences

### Positive
- **Professional Architecture**: Clean plugin system matching industry standards
- **Extensibility**: Users can add custom plugins alongside built-in systems
- **Maintainability**: Clear separation of concerns with unified initialization
- **Bevy Integration**: Seamless integration with Bevy's Plugin system and patterns
- **Configuration**: Centralized configuration management across all subsystems
- **Testing**: Easier unit testing with isolated plugin components

### Negative
- **Complexity**: Additional abstraction layer over direct Bevy plugin usage
- **Performance**: Minimal overhead from trait object dispatch
- **Migration**: Existing examples need update to use AAAPlugins
- **Dependencies**: amp_engine now depends on all subsystem crates

### Mitigations
- **Documentation**: Comprehensive examples and migration guides
- **Performance**: Trait object overhead is negligible for plugin initialization
- **Migration**: Automated migration for existing examples
- **Testing**: Plugin isolation enables better testing strategies

## Implementation Approach

### Phase 1: Core Plugin Infrastructure (Week 1)
- Implement AAAPlugin trait in amp_engine
- Create PluginStage enum and staging system
- Implement basic AAAPlugins struct with builder pattern

### Phase 2: Subsystem Integration (Week 2)
- Convert amp_physics to PhysicsAAAPlugin
- Convert amp_gameplay audio to AudioAAAPlugin
- Convert amp_render to RenderAAAPlugin
- Convert config_core to ConfigAAAPlugin

### Phase 3: User Extension & Testing (Week 3)
- Implement custom plugin registration system
- Update city_demo to use AAAPlugins
- Comprehensive integration testing
- Performance validation

### Phase 4: Documentation & Examples (Week 4)
- Complete API documentation
- Plugin development guide
- Migration examples from manual plugin registration
- Performance benchmarks

## Performance Targets
- **Plugin Loading**: <5ms for all built-in plugins
- **Runtime Overhead**: <0.1ms per frame for plugin system
- **Memory Usage**: <1MB additional memory for plugin infrastructure
- **Startup Time**: No significant impact on application startup

## Quality Gates
- All existing tests continue to pass
- New plugin system integration tests
- Performance benchmarks meet targets
- Documentation coverage >90%
- Example migration completed successfully

## References
- ADR-0007: Strategic Shift to Bevy 0.16.1 Meta-Crate (architecture foundation)
- ADR-0008: Oracle-Guided AAA Feature Restoration Strategy (subsystem context)
- ADR-0009: GPU Culling Pipeline (rendering integration)
- Oracle consultations: Professional integration guidance
- Bevy Plugin system documentation: https://docs.rs/bevy/0.16.1/bevy/app/trait.Plugin.html
