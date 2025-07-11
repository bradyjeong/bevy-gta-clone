//! Vehicle systems and components
//!
//! This module provides comprehensive vehicle physics including:
//! - Suspension systems
//! - Drivetrain and engine simulation
//! - Steering mechanics
//! - Rapier3D integration

pub mod bundles;
pub mod components;
pub mod resources;
pub mod systems;

/// Prelude for vehicle module
pub mod prelude {
    pub use crate::vehicle::VehiclePlugin;
    pub use crate::vehicle::bundles::*;
    pub use crate::vehicle::components::*;
    pub use crate::vehicle::resources::*;
}

use bevy::prelude::*;

/// Schedule set for post-physics systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostPhysics;

/// Plugin for vehicle systems
#[derive(Default)]
pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        // Configure FixedUpdate to run at 60 Hz for physics
        app.insert_resource(Time::<Fixed>::from_hz(60.0));

        app.add_systems(Startup, systems::setup::setup_vehicle_systems)
            .add_systems(
                FixedUpdate,
                (
                    // VehicleControl phase
                    systems::input::handle_vehicle_input,
                    // Physics phase (handled by amp_physics)
                    // Rapier phase (handled by bevy_rapier3d)
                    // PostPhysics phase
                    (
                        systems::suspension::update_suspension,
                        systems::drivetrain::update_drivetrain,
                        systems::steering::update_steering,
                        systems::sync_rapier::sync_vehicle_physics,
                    )
                        .in_set(PostPhysics),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    systems::input::handle_vehicle_input,
                    systems::audio::update_vehicle_audio,
                ),
            )
            .configure_sets(
                FixedUpdate,
                PostPhysics.after(bevy_rapier3d::plugin::PhysicsSet::StepSimulation),
            )
            .register_type::<components::Vehicle>()
            .register_type::<components::VehicleInput>()
            .register_type::<components::VehicleAudio>()
            .register_type::<components::CarConfig>()
            // Physics component types are now owned by amp_gameplay
            .register_type::<components::PhysicsVehicle>()
            .register_type::<components::Engine>()
            .register_type::<components::Transmission>()
            .register_type::<components::Suspension>()
            .register_type::<components::Drivetrain>()
            .register_type::<components::Steering>()
            .register_type::<components::Brakes>()
            .register_type::<components::PhysicsVehicleInput>()
            .register_type::<components::WheelPhysics>();
    }
}
