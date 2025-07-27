# Amp Foundation

[![Crates.io](https://img.shields.io/crates/v/amp_foundation.svg)](https://crates.io/crates/amp_foundation)
[![Documentation](https://docs.rs/amp_foundation/badge.svg)](https://docs.rs/amp_foundation)

Amp Foundation is a unified facade crate providing core functionality for the Amp game engine. It offers a clean, Bevy-free API for essential game development utilities.

## Purpose

This facade crate serves as the foundation layer of the Amp ecosystem, providing:

- **Core Utilities**: Error handling, result types, and fundamental abstractions
- **Mathematical Operations**: Vector math, spatial indexing, bounds checking, and transforms
- **Configuration Management**: Asset loading, game configuration, and settings management
- **Clean API**: Single import point for commonly used types and functions

## Key Features

- 🚫 **Bevy-Free**: No Bevy dependencies - pure Rust utilities suitable for any engine
- 🎯 **Focused Interface**: Carefully curated re-exports for essential functionality
- 🧮 **Advanced Math**: Morton 3D encoding, AABB/Sphere bounds, transform utilities
- ⚙️ **Configuration**: Type-safe config loading with hot-reload support
- 🛡️ **Error Handling**: Comprehensive error types with thiserror integration

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
amp_foundation = "0.1.0"
```

Basic usage:

```rust
use amp_foundation::prelude::*;

fn main() -> Result<()> {
    // Load game configuration
    let config = GameConfig::load("game.ron")?;
    
    // Create spatial bounds
    let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::new(100.0, 50.0, 100.0));
    
    // Use Morton encoding for spatial indexing
    let morton = Morton3D::new(Vec3::new(10.0, 20.0, 30.0));
    let spatial_hash = morton.encode();
    
    println!("Spatial hash: {}", spatial_hash);
    Ok(())
}
```

## Architecture

Amp Foundation follows the facade pattern, providing a unified interface to:

- [`amp_core`] - Core error handling and utilities
- [`amp_math`] - Mathematical operations and spatial algorithms  
- [`config_core`] - Configuration management and asset loading

## Feature Flags

- `unstable_advanced_input` - Enable experimental input handling features
- `unstable_road_system` - Enable experimental road/path system features

## Integration

For engine integration, see:

- [`amp_system`] - Physics and rendering systems (Bevy-based)
- [`amp_game`] - Gameplay systems and entity factories (Bevy-based)

## Examples

See the [examples directory](../../examples/) for complete usage examples.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
