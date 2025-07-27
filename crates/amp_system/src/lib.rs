#![deny(missing_docs, rust_2018_idioms, unsafe_code)]

//! # amp_system
//!
//! System facade crate providing physics and rendering systems.
//! This crate integrates with Bevy and provides the core systems for the game engine.
//!
//! ## Re-exported Crates
//! - [`amp_physics`] - Physics simulation, vehicle dynamics, and collision detection (optional)
//! - [`amp_render`] - Rendering pipeline, GPU culling, and LOD systems (optional)
//!
//! ## Usage
//! ```rust
//! use amp_system::prelude::*;
//! use bevy::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(PhysicsPlugin::default())  // requires "physics" feature
//!         .add_plugins(BatchingPlugin)            // requires "render" feature
//!         .run();
//! }
//! ```

// Re-export selected public items from system crates
#[cfg(feature = "physics")]
pub use amp_physics::{
    InterpolatedTransform, PhysicsConfig, PhysicsDebugPlugin, PhysicsPlugin, PhysicsSets,
    PhysicsTime, SuspensionPlugin,
};

#[cfg(feature = "render")]
pub use amp_render::BatchingPlugin;

#[cfg(feature = "render")]
pub use amp_render::prelude::{
    CullingSystemPlugin, LodSystemPlugin, OptimizedCullingPlugin, PerformanceDiagnosticsPlugin,
};

#[cfg(feature = "gpu_culling")]
pub use amp_render::gpu_culling::prelude::GpuCullingPlugin;

/// Prelude module for common system imports
pub mod prelude {
    // Physics systems
    #[cfg(feature = "physics")]
    pub use amp_physics::{
        PhysicsConfig, PhysicsPlugin, PhysicsSets, PhysicsTime, SuspensionPlugin,
    };

    // Rendering systems
    #[cfg(feature = "render")]
    pub use amp_render::BatchingPlugin;

    #[cfg(feature = "render")]
    pub use amp_render::prelude::{CullingSystemPlugin, LodSystemPlugin, OptimizedCullingPlugin};

    // Essential Bevy re-exports for system facades
    pub use bevy::prelude::{
        App, Commands, Component, Entity, Plugin, Query, Res, ResMut, Resource, Transform, Update,
        Vec3,
    };
}
