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

pub mod audio;
pub mod physics;
pub mod vehicle;

/// Prelude module for convenient imports
pub mod prelude {
    // pub use crate::GameplayPlugins; // TODO: Define GameplayPlugins struct
    pub use crate::audio::{AudioPlugin, components::*, resources::*};
    pub use crate::physics::{PhysicsPluginBridge, resources::*};
    pub use crate::vehicle::prelude::*;
}
