//! # amp_gameplay
//!
//! Core gameplay systems for AAA-level open world game.
//!
//! This crate provides:
//! - Vehicle physics integration with Rapier3D
//! - Advanced audio systems with bevy_kira_audio
//! - Gameplay components and systems
//! - Plugin architecture for easy integration
//!
//! ## Usage
//!
//! ```rust
//! use amp_gameplay::prelude::*;
//! use bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(GameplayPlugins)
//!     .run();
//! ```

pub mod audio;
pub mod physics;
pub mod vehicle;

// Re-export amp_physics components as single source of truth
pub use amp_physics::components::{
    Brakes, Drivetrain, Engine as VehicleEngine, Steering as VehicleSteering,
    Suspension as VehicleSuspension, VehicleInput as PhysicsVehicleInput,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::GameplayPlugins;
    pub use crate::audio::*;
    pub use crate::physics::*;
    pub use crate::vehicle::prelude::*;
    // Re-export physics components in prelude
    pub use crate::{
        Brakes, Drivetrain, PhysicsVehicleInput, VehicleEngine, VehicleSteering, VehicleSuspension,
    };
}

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

/// Plugin group for all gameplay systems
pub struct GameplayPlugins;

impl PluginGroup for GameplayPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(physics::PhysicsPluginBridge::default())
            .add(vehicle::VehiclePlugin)
            .add(audio::AudioPlugin)
    }
}
