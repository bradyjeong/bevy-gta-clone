#![deny(missing_docs, rust_2018_idioms, unsafe_code)]

//! # amp_game
//!
//! Game facade crate providing gameplay systems and entity factories.
//! This crate contains the high-level game logic and content creation systems.
//!
//! ## Re-exported Crates
//! - [`amp_gameplay`] - Core gameplay systems, components, and mechanics
//! - [`gameplay_factory`] - Entity spawning, factory systems, and prefab management
//!
//! ## Usage
//! ```rust
//! use amp_game::prelude::*;
//! use bevy::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(GameplayPlugins)
//!         .add_plugins(PrefabFactoryPlugin)
//!         .run();
//! }
//! ```

// Re-export selected public items from gameplay crates
pub use amp_gameplay::{
    character::components::{Player, Velocity},
    npc::NpcPlugin,
    vehicle::bundles::VehicleBundle,
    GameplayPlugins,
};
pub use gameplay_factory::{
    BasicPrefab, Factory, NpcFactory, PrefabFactoryPlugin, PrefabId, VehicleFactory,
};

/// Prelude module for common gameplay imports
pub mod prelude {
    // Gameplay systems
    pub use amp_gameplay::GameplayPlugins;

    // Factory systems
    pub use gameplay_factory::{
        BasicPrefab, Factory, NpcFactory, PrefabFactoryPlugin, PrefabId, VehicleFactory,
    };

    // Common components from gameplay
    pub use amp_gameplay::character::components::{Player, Velocity};

    // Essential Bevy re-exports for game development
    pub use bevy::prelude::Resource;
    pub use bevy::{
        app::{App, Plugin, Startup, Update},
        ecs::{
            bundle::Bundle,
            component::Component,
            entity::Entity,
            system::{Commands, Query, Res, ResMut},
        },
        math::{Quat, Vec3},
        time::Time,
        transform::components::Transform,
    };
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_facade_imports() {
        // Test that we can create basic game components
        let _player = Player;
        let _velocity = Velocity::default();
    }

    #[test]
    fn test_prefab_id() {
        let prefab_id = PrefabId::new(42);
        assert_eq!(prefab_id.raw(), 42);
    }
}
