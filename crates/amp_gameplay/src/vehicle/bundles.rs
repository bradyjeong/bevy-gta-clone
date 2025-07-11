//! Vehicle component bundles for easy entity creation

use crate::vehicle::components::*;
use amp_physics::components::{
    Brakes, Drivetrain, Engine, Steering, Suspension, Transmission, Vehicle as PhysicsVehicle,
    VehicleInput as PhysicsVehicleInput,
};
use bevy::prelude::*;

/// Bundle for creating a complete vehicle entity
#[derive(Bundle, Default)]
pub struct VehicleBundle {
    /// Vehicle identification and metadata (gameplay)
    pub vehicle: crate::vehicle::components::Vehicle,
    /// Vehicle input handling (gameplay)
    pub input: VehicleInput,
    /// Vehicle audio components (gameplay)
    pub audio: VehicleAudio,
    /// Transform component
    pub transform: Transform,
    /// Global transform component
    pub global_transform: GlobalTransform,
    /// Visibility component
    pub visibility: Visibility,
    /// Inherited visibility component
    pub inherited_visibility: InheritedVisibility,
    /// View visibility component
    pub view_visibility: ViewVisibility,
    /// Vehicle marker component (physics)
    pub physics_vehicle: PhysicsVehicle,
    /// Engine component (physics)
    pub engine: Engine,
    /// Transmission component (physics)
    pub transmission: Transmission,
    /// Suspension component (physics)
    pub suspension: Suspension,
    /// Drivetrain component (physics)
    pub drivetrain: Drivetrain,
    /// Steering component (physics)
    pub steering: Steering,
    /// Brakes component (physics)
    pub brakes: Brakes,
    /// Vehicle input (physics)
    pub physics_input: PhysicsVehicleInput,
}

/// Bundle for a basic car
#[derive(Bundle, Default)]
pub struct CarBundle {
    /// Vehicle identification and metadata (gameplay)
    pub vehicle: crate::vehicle::components::Vehicle,
    /// Vehicle input handling (gameplay)
    pub input: VehicleInput,
    /// Vehicle audio components (gameplay)
    pub audio: VehicleAudio,
    /// Transform component
    pub transform: Transform,
    /// Global transform component
    pub global_transform: GlobalTransform,
    /// Visibility component
    pub visibility: Visibility,
    /// Inherited visibility component
    pub inherited_visibility: InheritedVisibility,
    /// View visibility component
    pub view_visibility: ViewVisibility,
    /// Car-specific configuration
    pub car_config: CarConfig,
    /// Vehicle marker component (physics)
    pub physics_vehicle: PhysicsVehicle,
    /// Engine component (physics)
    pub engine: Engine,
    /// Transmission component (physics)
    pub transmission: Transmission,
    /// Suspension component (physics)
    pub suspension: Suspension,
    /// Drivetrain component (physics)
    pub drivetrain: Drivetrain,
    /// Steering component (physics)
    pub steering: Steering,
    /// Brakes component (physics)
    pub brakes: Brakes,
    /// Vehicle input (physics)
    pub physics_input: PhysicsVehicleInput,
}
