# Porting Quick Reference Guide

**Oracle-Approved Foundation for Systematic Code Porting**

This guide provides quick access to the porting tools and processes established for the GTA game project.

## 🚀 Quick Start

### 1. Analyze Dependencies
```bash
./scripts/trace_deps.sh <source_directory>
```
**Example**: `./scripts/trace_deps.sh crates/amp_gameplay/src/water`

### 2. Create New Module
```bash
source scripts/porting_utils.sh
create_module_structure <target_crate> <module_name>
```
**Example**: `create_module_structure amp_gameplay lighting`

### 3. Follow Template
Use `docs/PORTING_TEMPLATE.md` as your checklist

### 4. Validate Port
```bash
source scripts/porting_utils.sh
validate_port <target_crate>
```

## 📁 Successful Pattern Reference

Based on **Water** (`amp_gameplay/src/water/`) and **Persistence** (`amp_gameplay/src/persistence/`) systems:

```
module/
├── mod.rs           # Exports: pub use components::*; pub use systems::*; pub use plugin::*;
├── components.rs    # #[derive(Component, Default, Debug, Clone)]
├── systems.rs       # Bevy systems with proper Query patterns
├── plugin.rs        # impl Plugin for ModulePlugin
├── serializable.rs  # Optional: For save/load (see persistence example)
└── tests.rs         # #[cfg(test)] mod tests
```

## 🛠️ Available Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| `trace_deps.sh` | Analyze source dependencies | `./scripts/trace_deps.sh <directory>` |
| `porting_utils.sh` | Collection of helper functions | `source scripts/porting_utils.sh` |
| `PORTING_TEMPLATE.md` | Systematic porting checklist | Fill out for each port |

## 🎯 Target Crate Selection

| Content Type | Suggested Crate | Indicators |
|--------------|----------------|------------|
| **Rendering** | `amp_render` | Transform, Material, Mesh, Camera |
| **Physics** | `amp_physics` | RigidBody, Collider, Velocity, rapier |
| **Gameplay** | `amp_gameplay` | Player, NPC, Vehicle, Character |
| **Configuration** | `config_core` | Config, Settings, .ron, serde |
| **Entity Factory** | `gameplay_factory` | spawn, factory, Entity bundles |
| **Math Utils** | `amp_math` | AABB, Morton, Vec3, Quat |
| **Core Utils** | `amp_core` | No Bevy dependencies, pure Rust |
| **Engine Systems** | `amp_engine` | Bevy plugins, engine abstraction |

## ⚡ Quick Commands

### Dependency Analysis
```bash
# Analyze any directory
./scripts/trace_deps.sh crates/amp_render/src/culling

# Check for porting issues
source scripts/porting_utils.sh
check_porting_issues crates/amp_gameplay/src/vehicle
```

### Module Creation
```bash
source scripts/porting_utils.sh

# Create new module structure
create_module_structure amp_gameplay ai_behavior

# Validate existing structure
validate_module_structure crates/amp_gameplay/src/water

# Generate integration example
generate_integration_template amp_gameplay water
```

### Port Validation
```bash
source scripts/porting_utils.sh

# Comprehensive validation
validate_port amp_gameplay

# Update imports (with backup)
update_imports src/main.rs "use old::path" "use new::path"
```

## 🔄 Standard Workflow

1. **Analysis**: `./scripts/trace_deps.sh <source>`
2. **Planning**: Fill out `PORTING_TEMPLATE.md` Phase 1
3. **Creation**: `create_module_structure <crate> <module>`
4. **Implementation**: Port code following template phases
5. **Validation**: `validate_port <crate>` + comprehensive testing
6. **Integration**: Update main app and examples

## 🎨 Code Patterns

### Component Pattern
```rust
// components.rs
use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct MyComponent {
    pub field: f32,
}
```

### System Pattern
```rust
// systems.rs  
use bevy::prelude::*;
use super::components::*;

pub fn my_system(
    mut query: Query<&mut MyComponent>,
    time: Res<Time>,
) {
    // System logic
}
```

### Plugin Pattern
```rust
// plugin.rs
use bevy::prelude::*;
use super::systems::*;

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system);
        info!("📦 My Plugin initialized");
    }
}
```

### Export Pattern
```rust
// mod.rs
pub mod components;
pub mod systems;
pub mod plugin;

#[cfg(test)]
mod tests;

pub use components::*;
pub use systems::*;  
pub use plugin::*;
```

## ⚠️ Common Gotchas

- **Bevy API**: Use `app.world()` not `app.world`
- **Imports**: Check for circular dependencies between crates
- **Components**: Always include `#[derive(Component)]`
- **Tests**: Don't forget test visibility (`pub fn` or `use super::*`)
- **Documentation**: Add comprehensive doc comments

## 📋 Validation Checklist

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes  
- [ ] `cargo clippy --workspace --all-targets --all-features` clean
- [ ] `cargo fmt --all` applied
- [ ] Module structure follows established pattern
- [ ] All imports updated correctly
- [ ] Integration points tested
- [ ] Documentation complete

## 🔗 Related Files

- **Main Template**: `docs/PORTING_TEMPLATE.md`
- **Architecture Guide**: `AGENT.md`
- **ADR Records**: `docs/adr/`
- **Examples**: `crates/amp_gameplay/src/{water,persistence}/`

---

**📝 Oracle Guidance**: This foundation is based on successful Water and Persistence system ports. Follow these patterns for consistent, maintainable code migration.
