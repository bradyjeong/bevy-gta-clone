//! Physics bundles for the Amp game engine.
//!
//! This module provides convenient bundles for spawning complete physics entities
//! with all necessary components pre-configured.

use crate::components::*;
use bevy::prelude::*;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::{Collider, RigidBody};

/// Complete vehicle physics bundle.
///
/// This bundle contains all the necessary components for a fully functional
/// vehicle entity with realistic physics simulation. It includes:
/// - Vehicle marker component
/// - RigidBody for physics simulation
/// - Collider for collision detection
/// - Engine component for engine physics
/// - Transmission component for gear simulation
/// - Transform for positioning
/// - Name for identification
///
/// The bundle is designed to work with 4 wheel child entities that should be
/// spawned separately and attached to this vehicle entity.
#[derive(Bundle)]
pub struct VehicleBundle {
    /// Vehicle marker component
    pub vehicle: Vehicle,

    /// Rigid body for physics simulation
    #[cfg(feature = "rapier3d_030")]
    pub rigid_body: RigidBody,

    /// Collider for collision detection
    #[cfg(feature = "rapier3d_030")]
    pub collider: Collider,

    /// Engine component
    pub engine: Engine,

    /// Transmission component
    pub transmission: Transmission,

    /// Transform for positioning
    pub transform: Transform,

    /// Name for identification
    pub name: Name,
}

impl Default for VehicleBundle {
    fn default() -> Self {
        Self {
            vehicle: Vehicle,
            #[cfg(feature = "rapier3d_030")]
            rigid_body: RigidBody::Dynamic,
            #[cfg(feature = "rapier3d_030")]
            collider: Collider::cuboid(2.0, 0.8, 4.5), // Typical car dimensions
            engine: Engine::default(),
            transmission: Transmission::default(),
            transform: Transform::default(),
            name: Name::new("Vehicle"),
        }
    }
}

impl VehicleBundle {
    /// Create a new VehicleBundle with custom parameters.
    pub fn new(
        engine: Engine,
        transmission: Transmission,
        transform: Transform,
        name: String,
    ) -> Self {
        Self {
            vehicle: Vehicle,
            #[cfg(feature = "rapier3d_030")]
            rigid_body: RigidBody::Dynamic,
            #[cfg(feature = "rapier3d_030")]
            collider: Collider::cuboid(2.0, 0.8, 4.5),
            engine,
            transmission,
            transform,
            name: Name::new(name),
        }
    }

    /// Create a VehicleBundle with custom collider dimensions.
    #[cfg(feature = "rapier3d_030")]
    pub fn with_collider(
        engine: Engine,
        transmission: Transmission,
        transform: Transform,
        collider: Collider,
        name: String,
    ) -> Self {
        Self {
            vehicle: Vehicle,
            rigid_body: RigidBody::Dynamic,
            collider,
            engine,
            transmission,
            transform,
            name: Name::new(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_bundle_creation() {
        let bundle = VehicleBundle::default();

        // Test that all components are properly initialized
        assert_eq!(bundle.vehicle, Vehicle);
        assert_eq!(bundle.engine.rpm, 0.0);
        assert_eq!(bundle.transmission.current_gear, 1);
        assert_eq!(bundle.transform, Transform::default());
        assert_eq!(bundle.name.as_str(), "Vehicle");
    }

    #[test]
    fn vehicle_bundle_custom_creation() {
        let engine = Engine {
            rpm: 2000.0,
            throttle: 0.5,
            torque: 100.0,
            max_rpm: 8000.0,
            max_torque: 400.0,
            idle_rpm: 800.0,
            engine_braking: 0.3,
            fuel_consumption: 15.0,
            torque_curve: vec![
                (0.0, 0.0),
                (800.0, 120.0),
                (1500.0, 200.0),
                (3000.0, 400.0),
                (4500.0, 350.0),
                (6000.0, 250.0),
                (8000.0, 200.0),
            ],
        };
        let transmission = Transmission {
            gear_ratios: vec![3.0, 2.0, 1.5, 1.0],
            current_gear: 2,
            final_drive_ratio: 3.8,
        };
        let transform = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let name = "Test Vehicle";

        let bundle = VehicleBundle::new(
            engine.clone(),
            transmission.clone(),
            transform,
            name.to_string(),
        );

        assert_eq!(bundle.vehicle, Vehicle);
        assert_eq!(bundle.engine, engine);
        assert_eq!(bundle.transmission, transmission);
        assert_eq!(bundle.transform, transform);
        assert_eq!(bundle.name.as_str(), name);
    }

    #[cfg(feature = "rapier3d_030")]
    #[test]
    fn vehicle_bundle_with_custom_collider() {
        let engine = Engine::default();
        let transmission = Transmission::default();
        let transform = Transform::default();
        let collider = Collider::cuboid(1.5, 0.6, 3.0);
        let name = "Custom Collider Vehicle";

        let bundle = VehicleBundle::with_collider(
            engine.clone(),
            transmission.clone(),
            transform,
            collider.clone(),
            name.to_string(),
        );

        assert_eq!(bundle.vehicle, Vehicle);
        assert_eq!(bundle.engine, engine);
        assert_eq!(bundle.transmission, transmission);
        assert_eq!(bundle.transform, transform);
        assert_eq!(bundle.name.as_str(), name);

        // Note: We can't directly compare colliders as they don't implement PartialEq
        // But we can verify the bundle was created without panicking
    }

    #[cfg(feature = "rapier3d_030")]
    #[test]
    fn vehicle_bundle_has_physics_components() {
        let bundle = VehicleBundle::default();

        // Test that physics components are present
        match bundle.rigid_body {
            RigidBody::Dynamic => {} // Expected
            _ => panic!("Expected Dynamic rigid body"),
        }

        // Collider is present (can't test exact values due to no PartialEq)
        // but we can verify it was created
    }

    #[test]
    fn vehicle_bundle_spawn_test() {
        // Test that the bundle can be used in a minimal Bevy app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let bundle = VehicleBundle::default();

        // Spawn the bundle
        app.world_mut().spawn(bundle);

        // Verify the entity was created with the correct components
        let mut query = app
            .world_mut()
            .query::<(Entity, &Vehicle, &Engine, &Transmission, &Name)>();
        let results: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(results.len(), 1);
        let (_, vehicle, engine, transmission, name) = results[0];

        assert_eq!(*vehicle, Vehicle);
        assert_eq!(engine.rpm, 0.0);
        assert_eq!(transmission.current_gear, 1);
        assert_eq!(name.as_str(), "Vehicle");
    }
}
