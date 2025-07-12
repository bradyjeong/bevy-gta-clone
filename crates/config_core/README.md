# Config Core

A robust configuration system for the AMP Game Engine with hot-reload support and full Bevy integration.

## Features

- **RonAssetPlugin**: Stable RON asset loading with typed handles
- **ConfigHandle<T>**: Resource pattern with deref access and change detection
- **Hot-reload**: Live config updates during runtime via Bevy's asset system
- **Hierarchical Merge**: Load configs from multiple sources with proper precedence
- **Type Safety**: Fully typed configuration with compile-time validation
- **Bevy Integration**: First-class Bevy asset system integration

## Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
config_core = { path = "../config_core", features = ["bevy-assets"] }
```

### 2. Define Your Config Type

```rust
use bevy::prelude::*;
use config_core::assets::RonAssetPlugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct GameSettings {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            volume: 1.0,
        }
    }
}
```

### 3. Add the Plugin

```rust
use config_core::assets::{ConfigHandle, RonAssetPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RonAssetPlugin::<GameSettings>::new("configs/game.ron"))
        .add_systems(Update, handle_config_changes)
        .run();
}
```

### 4. Use the Config

```rust
fn handle_config_changes(
    config: Res<ConfigHandle<GameSettings>>,
) {
    if config.is_loaded() {
        if let Some(settings) = config.get() {
            println!("Window size: {}x{}", settings.window_width, settings.window_height);
            println!("Config version: {}", config.version());
        }
    }
}
```

### 5. Create Your Config File

Create `assets/configs/game.ron`:

```ron
(
    window_width: 1920,
    window_height: 1080,
    fullscreen: false,
    volume: 0.8,
)
```

## Advanced Usage

### Change Detection

The `ConfigHandle<T>` provides version tracking for efficient change detection:

```rust
fn detect_config_changes(
    config: Res<ConfigHandle<GameSettings>>,
    mut last_version: Local<u64>,
) {
    if config.has_changed(*last_version) {
        *last_version = config.version();
        println!("Config changed to version {}", *last_version);
        
        // React to config changes
        if let Some(settings) = config.get() {
            // Update systems based on new config
        }
    }
}
```

### Deref Access

ConfigHandle implements `Deref` for convenient access:

```rust
fn use_config_deref(config: Res<ConfigHandle<GameSettings>>) {
    // Direct deref access to Option<Arc<T>>
    if let Some(settings) = &*config {
        println!("Volume: {}", settings.volume);
    }
}
```

### Multiple Config Types

You can register multiple config types:

```rust
use config_core::types::{GameConfigAsset, VehicleStatsConfig, PerformanceSettingsConfig};

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins((
        RonAssetPlugin::<GameConfigAsset>::new("configs/game.ron"),
        RonAssetPlugin::<VehicleStatsConfig>::new("configs/vehicles.ron"),
        RonAssetPlugin::<PerformanceSettingsConfig>::new("configs/performance.ron"),
    ))
    .add_systems(Update, (
        handle_game_config,
        handle_vehicle_config,
        handle_performance_config,
    ))
    .run();
```

### Hot-Reload in Action

The system automatically detects file changes and updates the config:

1. Modify your `.ron` file
2. The asset system detects the change
3. ConfigHandle version increments
4. Your systems can react to `has_changed()` calls

## Configuration Types

The crate provides several pre-defined configuration types:

- `GameConfigAsset`: Main game settings
- `VehicleStatsConfig`: Vehicle parameters
- `PerformanceSettingsConfig`: Performance tuning
- `AudioSettingsConfig`: Audio configuration
- `PhysicsConstantsConfig`: Physics parameters
- And more...

## Legacy Support

The crate also provides the traditional `ConfigLoader` for non-Bevy applications:

```rust
use config_core::{ConfigLoader, GameConfig};

let loader = ConfigLoader::new();
let config: GameConfig = loader.load_with_merge()?;
```

## Examples

Run the provided examples to see the system in action:

```bash
# Basic Bevy integration demo
cargo run --example bevy_config_demo --features="bevy-assets"

# Test hot-reload by modifying configs while running
```

## Testing

Run the comprehensive test suite:

```bash
# All tests with Bevy integration
cargo test --features="bevy-assets"

# Core config system tests only
cargo test --lib

# Integration tests only
cargo test --test bevy_integration_tests --features="bevy-assets"
```

## Architecture

The system is built around three core components:

1. **RonAssetLoader<T>**: Handles loading RON files as Bevy assets
2. **ConfigHandle<T>**: Resource wrapper providing typed access and change detection
3. **RonAssetPlugin<T>**: Bevy plugin that ties everything together

The hot-reload functionality is built on top of Bevy's asset system, which uses file watchers to detect changes and automatically reload modified assets.

## Performance

- **Zero-copy access**: Configs are stored in `Arc<T>` for efficient sharing
- **Change detection**: Version-based change tracking avoids unnecessary processing
- **Async loading**: Non-blocking asset loading via Bevy's asset system
- **Hot-reload**: File watching with minimal overhead

## License

MIT OR Apache-2.0
