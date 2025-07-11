//! Physics simulation and collision detection for the Amp game engine.
//!
//! This crate provides physics simulation capabilities using Rapier3D,
//! wrapped in a clean Bevy plugin interface.

use bevy::prelude::*;

pub mod benchmarks;
pub mod bundles;
pub mod components;
pub mod debug;
pub mod drivetrain;
pub mod suspension;
pub mod systems;
pub mod time;

#[cfg(feature = "rapier3d_030")]
pub mod rapier;

pub use benchmarks::{
    BenchmarkConfig, BenchmarkResults, PhysicsBenchmarkPlugin, create_standard_benchmark,
    create_stress_test_benchmark,
};
pub use bundles::*;
pub use components::*;
pub use debug::{DebugConfig, PhysicsDebugPlugin, PhysicsPerformanceMetrics};
pub use drivetrain::DrivetrainPlugin;
pub use suspension::{
    PhysicsUpdate, SuspensionPlugin, SuspensionRay, WheelState, vehicle_suspension_system,
};
pub use systems::*;
pub use time::{PhysicsConfig, PhysicsTime, apply_physics_config, update_physics_time};

#[cfg(feature = "rapier3d_030")]
pub use rapier::*;

/// Main physics plugin for the Amp game engine.
///
/// This plugin wraps the underlying physics engine (Rapier3D) and provides
/// a unified interface for physics simulation in the game.
#[derive(Default)]
pub struct PhysicsPlugin {
    /// Optional configuration for the physics plugin
    pub config: Option<PhysicsConfig>,
}

impl PhysicsPlugin {
    /// Create a new PhysicsPlugin with the given configuration
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Add core physics resources and systems
        app.init_resource::<PhysicsTime>()
            .add_systems(Update, (update_physics_time, apply_physics_config));

        // Insert provided config or default
        if let Some(config) = &self.config {
            app.insert_resource(config.clone());
        } else {
            app.init_resource::<PhysicsConfig>();
        }

        #[cfg(feature = "rapier3d_030")]
        {
            use bevy_rapier3d::prelude::*;

            // Configure Rapier with CCD disabled for performance
            app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
                .add_plugins(SuspensionPlugin)
                .add_plugins(DrivetrainPlugin);
        }

        #[cfg(not(feature = "rapier3d_030"))]
        {
            warn!(
                "No physics backend enabled. Enable the 'rapier3d_030' feature to use physics simulation."
            );
            app.add_plugins(SuspensionPlugin)
                .add_plugins(DrivetrainPlugin);
        }

        // Add debug and benchmarking plugins
        app.add_plugins(PhysicsDebugPlugin)
            .add_plugins(PhysicsBenchmarkPlugin);

        #[cfg(feature = "inspector")]
        {
            app.register_type::<PhysicsTime>()
                .register_type::<PhysicsConfig>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn physics_plugin_can_be_added() {
        let mut app = App::new();
        app.add_plugins(PhysicsPlugin::default());

        // Test that the plugin can be added without panicking
        // Note: We don't call update() because Rapier requires AssetPlugin
        // This test verifies the plugin builds correctly with the feature flag
    }
}
