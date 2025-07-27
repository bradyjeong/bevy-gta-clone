# Amp Game

[![Crates.io](https://img.shields.io/crates/v/amp_game.svg)](https://crates.io/crates/amp_game)
[![Documentation](https://docs.rs/amp_game/badge.svg)](https://docs.rs/amp_game)

Amp Game is a facade crate providing high-level gameplay systems and entity factories for the Amp game engine. It integrates with Bevy to deliver complete game development functionality.

## Purpose

This facade crate serves as the gameplay layer of the Amp ecosystem, providing:

- **Gameplay Systems**: Character systems, NPCs, vehicles, and game mechanics
- **Entity Factories**: Prefab management, spawning systems, and content creation
- **Game Integration**: Complete Bevy plugin integration for gameplay features
- **Unified API**: Single import point for all gameplay functionality

## Key Features

- 🎮 **Complete Gameplay**: Player systems, NPCs, vehicles, and interactive content
- 🏭 **Entity Factories**: Efficient spawning and prefab management systems
- 🔌 **Bevy Integration**: Seamless plugin system integration with minimal setup
- ⚡ **Performance Optimized**: Efficient entity creation and management
- 🧩 **Modular Design**: Mix and match gameplay components as needed

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
amp_game = "0.1.0"
bevy = "0.16.1"
```

Basic usage:

```rust
use amp_game::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayPlugins)
        .add_plugins(PrefabFactoryPlugin)
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(
    mut commands: Commands,
    mut vehicle_factory: VehicleFactory,
    mut npc_factory: NpcFactory,
) {
    // Spawn a player vehicle
    vehicle_factory.spawn_vehicle(&mut commands, "sports_car", Vec3::ZERO);
    
    // Spawn NPCs around the world
    for i in 0..10 {
        let position = Vec3::new(i as f32 * 5.0, 0.0, 0.0);
        npc_factory.spawn_npc(&mut commands, "pedestrian", position);
    }
}
```

## Architecture

Amp Game follows the facade pattern, providing a unified interface to:

- [`amp_gameplay`] - Core gameplay systems, components, and mechanics
- [`gameplay_factory`] - Entity spawning, factory systems, and prefab management

## Feature Flags

- `ron` - Enable RON format support for configuration files
- `tracy` - Enable Tracy profiler integration for performance analysis
- `perf_trace` - Enable performance tracing and diagnostics
- `unstable_hierarchical_world` - Enable experimental world hierarchy features
- `unstable_advanced_input` - Enable experimental input handling features
- `unstable_road_system` - Enable experimental road/path system features
- `rapier3d_030` - Enable Rapier3D physics engine integration

## Systems Overview

### Character Systems
- Player controller with movement and interaction
- NPC behavior and AI systems
- Character physics and animation integration

### Vehicle Systems  
- Realistic vehicle physics and dynamics
- Vehicle spawning and management
- Suspension, engine, and transmission systems

### Factory Systems
- Efficient entity spawning with object pooling
- Prefab-based content creation
- Configurable spawn parameters and templates

## Integration

For foundational utilities, see:

- [`amp_foundation`] - Core utilities and math (Bevy-free)
- [`amp_system`] - Physics and rendering systems (Bevy-based)

## Examples

See the [examples directory](../../examples/) for complete gameplay examples including:

- Character controller setup
- Vehicle spawning and physics
- NPC behavior systems
- Factory-based entity creation

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
