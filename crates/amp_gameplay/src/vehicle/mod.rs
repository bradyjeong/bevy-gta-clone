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

#[cfg(test)]
pub mod tests;

/// Prelude for vehicle module
pub mod prelude {
    pub use crate::vehicle::bundles::*;
    pub use crate::vehicle::components::*;
    pub use crate::vehicle::resources::*;
    pub use crate::vehicle::VehiclePlugin;
}

use bevy::prelude::*;

/// Schedule set for post-physics systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostPhysics;

/// Oracle Sprint 9 D4-7: PhaseSet for gameplay systems to minimize flush barriers
/// Target: -0.5ms from reduced system ordering overhead
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameplayPhaseSet;

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
                    // VehicleControl phase - grouped for minimal flush barriers
                    systems::input::handle_vehicle_input.in_set(GameplayPhaseSet),
                    // Oracle critical fix: Update input state resource from components
                    systems::input::update_input_state_from_components.in_set(GameplayPhaseSet),
                    // Physics phase (handled by amp_physics)
                    // Rapier phase (handled by bevy_rapier3d)
                    // PostPhysics phase - Oracle Sprint 9: Group all systems + SIMD optimization
                    (
                        systems::suspension::update_suspension,
                        systems::drivetrain::update_drivetrain,
                        systems::steering::update_steering,
                        // Oracle Sprint 9 D4-7: Optimized wheel updates (target: -0.2ms)
                        systems::wheel_optimized::update_wheel_physics_optimized,
                        systems::wheel_optimized::apply_steering_optimized,
                        systems::sync_rapier::sync_vehicle_physics,
                        // Sprint 9 optimization: Manage vehicle sleeping
                        systems::sync_rapier::manage_vehicle_sleeping,
                    )
                        .in_set(PostPhysics)
                        .in_set(GameplayPhaseSet),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    // Ensure vehicles have cached physics component
                    systems::audio::ensure_vehicle_cached_physics,
                    // Cache physics data from FixedUpdate for use in Update systems
                    systems::audio::cache_vehicle_physics_for_audio,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    // Audio systems run in Update for real-time responsiveness
                    // but use cached physics data from FixedUpdate
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
            .register_type::<systems::audio::CachedVehiclePhysics>()
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
