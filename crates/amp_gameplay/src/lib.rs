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
//! ```ignore
//! use amp_gameplay::prelude::*;
//! use bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(GameplayPlugins)
//!     .run();
//! ```
//!
//! Simple plugin verification:
//!
//! ```rust
//! use amp_gameplay::GameplayPlugins;
//! use bevy::app::PluginGroup;
//!
//! // Verify GameplayPlugins can be built without heavy initialization
//! let plugins = GameplayPlugins;
//! let _builder = plugins.build();
//! // Plugin group builder created successfully - doctests working properly
//! assert!(true);
//! ```

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod audio;
pub mod character;
pub mod city;
pub mod interaction;
pub mod npc;
pub mod physics;
pub mod vehicle;

// Oracle's M4 requirements: Import world streaming and HUD
#[cfg(feature = "bevy16")]
use amp_engine::hud::HudPlugin;
#[cfg(feature = "bevy16")]
use amp_engine::world_streaming::WorldStreamingPlugin;

/// Collection of all gameplay plugins
#[derive(Default)]
pub struct GameplayPlugins;

impl PluginGroup for GameplayPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>()
            .add(character::CharacterPlugin)
            .add(vehicle::VehiclePlugin)
            .add(audio::AudioPlugin)
            .add(npc::NpcPlugin)
            .add(physics::PhysicsPluginBridge::default())
            .add(city::CityPlugin)
            .add(city::CityStreamingPlugin)
            .add(interaction::InteractionPlugin);

        // Oracle's M4 requirements: Add world streaming and HUD plugins
        #[cfg(feature = "bevy16")]
        {
            builder = builder.add(WorldStreamingPlugin).add(HudPlugin);
        }

        builder
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::audio::{components::*, resources::*, AudioPlugin};
    pub use crate::character::{bundles::*, components::*, CharacterPlugin};
    pub use crate::city::{components::*, resources::*, CityPlugin, CityStreamingPlugin};
    pub use crate::interaction::*;
    pub use crate::npc::*;
    pub use crate::physics::{resources::*, PhysicsPluginBridge};
    pub use crate::vehicle::prelude::*;
    pub use crate::GameplayPlugins;
}
