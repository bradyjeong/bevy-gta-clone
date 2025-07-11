//! Rapier3D physics backend implementation.

use bevy::prelude::*;

/// Re-export commonly used Rapier types for convenience.
pub use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, ActiveEvents, ActiveHooks, AdditionalMassProperties, Ccd, Collider,
    ColliderDisabled, ColliderMassProperties, CollisionEvent, CollisionGroups, ContactForceEvent,
    ContactSkin, Damping, ExternalForce, ExternalImpulse, Friction, GravityScale, LockedAxes,
    NoUserData, PhysicsSet, RapierConfiguration, RapierContext, RapierDebugRenderPlugin,
    RapierPhysicsPlugin, ReadMassProperties, Restitution, RigidBody, RigidBodyDisabled, Sensor,
    Sleeping, SolverGroups, TimestepMode, Velocity,
};

/// Helper function to create a static ground plane.
pub fn create_ground_plane(commands: &mut Commands) {
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(50.0, 0.1, 50.0),
        Transform::from_xyz(0.0, -0.1, 0.0),
        Name::new("Ground"),
    ));
}

/// Helper function to create a dynamic cube.
pub fn create_dynamic_cube(commands: &mut Commands, position: Vec3, size: f32) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(size, size, size),
        Transform::from_translation(position),
        Name::new("Dynamic Cube"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_physics_components() {
        // Test that we can create the physics components
        let rigid_body = RigidBody::Dynamic;
        let _collider = Collider::cuboid(1.0, 1.0, 1.0);
        let velocity = Velocity::default();

        // Basic smoke test to ensure the types are accessible
        assert_eq!(format!("{rigid_body:?}"), "Dynamic");
        assert_eq!(velocity.linvel, Vec3::ZERO);
    }
}
