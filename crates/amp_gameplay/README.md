# amp_gameplay

Core gameplay systems for AAA-level open world game built with Bevy 0.16.1.

## Features

- **Vehicle Physics**: Comprehensive vehicle simulation with realistic suspension, drivetrain, and steering
- **Audio Systems**: Advanced audio integration with bevy_kira_audio for engine sounds, environmental audio, and music
- **Rapier3D Integration**: Physics synchronization with Bevy's official physics backend
- **Plugin Architecture**: Modular design with easy-to-use plugin groups

## Usage

```rust
use amp_gameplay::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayPlugins)
        .run();
}
```

## Vehicle System

The vehicle system provides realistic physics simulation:

```rust
use amp_gameplay::prelude::*;

fn spawn_vehicle(mut commands: Commands, asset_server: Res<AssetServer>) {
    let vehicle = commands
        .spawn(PbrBundle {
            mesh: asset_server.load("models/car.glb#Scene0"),
            ..default()
        })
        .insert(Vehicle::default())
        .insert(VehicleEngine::default())
        .insert(VehicleSuspension::default())
        .insert(VehicleSteering::default())
        .id();
}
```

## Audio System

Advanced audio features with spatial positioning:

```rust
use amp_gameplay::prelude::*;

fn setup_audio(mut commands: Commands) {
    commands.spawn(EngineAudio::default());
    commands.spawn(EnvironmentalAudio::default());
    commands.spawn(MusicSystem::default());
}
```

## Architecture

- `vehicle/`: Vehicle physics and components
- `audio/`: Audio systems and components  
- `physics.rs`: Rapier3D integration wrapper

## Dependencies

- `bevy` (0.16.1): Game engine
- `bevy_rapier3d` (0.30.0): Physics simulation
- `bevy_kira_audio` (0.23.0): Audio engine
- `amp_physics`: Vehicle physics calculations
- `amp_core`: Core utilities and error handling
- `amp_math`: Mathematical utilities

## Development

This crate follows Oracle's Strategic Restoration Plan for AAA-level game development.
