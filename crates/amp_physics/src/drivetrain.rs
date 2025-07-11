//! Drivetrain and control systems plugin.
//!
//! This module provides a plugin that adds all the drivetrain and control
//! systems to the Bevy app.

use crate::systems::*;
use bevy::prelude::*;

/// Plugin that adds drivetrain and control systems to the app.
///
/// This plugin registers all the systems needed for realistic vehicle
/// drivetrain, steering, braking, and control input processing.
#[derive(Default)]
pub struct DrivetrainPlugin;

impl Plugin for DrivetrainPlugin {
    fn build(&self, app: &mut App) {
        // Only add control_input_system if not in test mode
        #[cfg(not(test))]
        {
            app.add_systems(
                Update,
                (
                    control_input_system,
                    engine_system,
                    drivetrain_system,
                    steering_system,
                    braking_system,
                )
                    .chain(),
            );
        }

        #[cfg(test)]
        {
            app.add_systems(
                Update,
                (
                    engine_system,
                    drivetrain_system,
                    steering_system,
                    braking_system,
                )
                    .chain(),
            );
        }

        #[cfg(feature = "rapier3d_030")]
        {
            app.add_systems(Update, wheel_physics_system);
        }
    }
}
