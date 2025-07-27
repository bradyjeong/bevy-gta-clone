//! Vehicle factory for spawning complete vehicle entities with physics.
//!
//! This module provides the VehicleFactory struct that converts VehicleConfig
//! into proper Bevy entities with physics components and hierarchy.

use amp_core::Error;
use amp_gameplay::spawn_budget_integration::{detect_biome_from_position, VehicleTag};
use amp_gameplay::spawn_budget_policy::{
    EntityType, SpawnBudgetPolicy, SpawnData, SpawnPriority, SpawnResult,
};
use amp_physics::{Engine, Suspension, SuspensionRay, Transmission, VehicleBundle, Wheel};
use bevy::prelude::*;
use config_core::VehicleConfig;

#[cfg(feature = "rapier3d_030")]
use amp_physics::rapier::{AdditionalMassProperties, Collider, RigidBody};

/// Factory for creating vehicle entities from VehicleConfig.
///
/// This factory handles the complete spawning of a vehicle entity with:
/// - Parent chassis entity with Dynamic RigidBody using VehicleBundle
/// - 4 wheel entities (Kinematic-PositionBased) each with SuspensionRay marker
/// - Proper parent-child hierarchy
#[derive(Default, Debug, PartialEq)]
pub struct VehicleFactory;

impl VehicleFactory {
    /// Create a new VehicleFactory instance.
    pub fn new() -> Self {
        Self
    }

    /// Spawn a complete vehicle entity from VehicleConfig.
    ///
    /// This method creates:
    /// 1. A parent chassis entity with Vehicle, Engine, Transmission, Suspension, and physics components
    /// 2. Four wheel entities as children, each with Wheel and SuspensionRay markers
    /// 3. Proper parent-child hierarchy
    ///
    /// Returns the Entity ID of the parent chassis entity.
    /// Spawn a vehicle entity with budget enforcement
    pub fn spawn_vehicle_with_budget(
        &self,
        commands: &mut Commands,
        policy: &mut ResMut<SpawnBudgetPolicy>,
        config: &VehicleConfig,
        position: Vec3,
        priority: SpawnPriority,
        time: &Res<Time>,
    ) -> Result<SpawnResult, Error> {
        let biome = detect_biome_from_position(position);
        let game_time = time.elapsed_secs();

        let spawn_data = SpawnData::Vehicle {
            position,
            vehicle_type: "vehicle".to_string(),
        };

        let result = policy.request_spawn(
            EntityType::Vehicle,
            biome,
            priority,
            spawn_data.clone(),
            game_time,
        );

        match result {
            SpawnResult::Approved => {
                // Immediate spawn with position
                let _entity = self.spawn_vehicle_immediate(commands, config, position)?;
                Ok(SpawnResult::Approved)
            }
            SpawnResult::Queued => Ok(SpawnResult::Queued),
            SpawnResult::Rejected(reason) => Ok(SpawnResult::Rejected(reason)),
        }
    }

    /// Spawn a vehicle entity (original method)
    pub fn spawn_vehicle(
        &self,
        commands: &mut Commands,
        config: &VehicleConfig,
    ) -> Result<Entity, Error> {
        self.spawn_vehicle_immediate(commands, config, Vec3::ZERO)
    }

    /// Internal immediate spawn method
    fn spawn_vehicle_immediate(
        &self,
        commands: &mut Commands,
        config: &VehicleConfig,
        position: Vec3,
    ) -> Result<Entity, Error> {
        // Create engine component from config with all fields
        let engine = Engine {
            rpm: 0.0,
            throttle: 0.0,
            torque: 0.0,
            max_rpm: config.engine.max_rpm,
            max_torque: config
                .engine
                .torque_curve_torque
                .iter()
                .copied()
                .fold(0.0, f32::max),
            idle_rpm: config.engine.idle_rpm,
            engine_braking: config.engine.engine_braking,
            fuel_consumption: config.engine.fuel_consumption,
            torque_curve: config
                .engine
                .torque_curve_rpm
                .iter()
                .zip(config.engine.torque_curve_torque.iter())
                .map(|(&rpm, &torque)| (rpm, torque))
                .collect(),
        };

        // Create transmission component from config using exact gear ratios
        let transmission = Transmission {
            gear_ratios: config.transmission.gear_ratios.clone(),
            current_gear: 0, // Start in neutral (0, not 1)
            final_drive_ratio: config.transmission.final_drive_ratio,
        };

        // Create suspension component from config
        let suspension = Suspension {
            spring_stiffness: config.suspension.spring_stiffness,
            damper_damping: config.suspension.damper_damping,
            max_compression: config.suspension.max_compression,
            max_extension: config.suspension.max_extension,
            rest_length: config.suspension.rest_length,
            anti_roll_bar_stiffness: config.suspension.anti_roll_bar_stiffness,
            travel: config.suspension.travel,
        };

        // Create vehicle bundle with position
        let vehicle_bundle = VehicleBundle::new(
            engine,
            transmission,
            Transform::from_translation(position),
            "Vehicle".to_string(),
        );

        // Spawn the parent chassis entity with suspension and mass, plus budget tracking tag
        let mut chassis_entity_commands = commands.spawn((
            vehicle_bundle,
            VehicleTag {
                vehicle_type: "vehicle".to_string(),
            },
        ));
        chassis_entity_commands.insert(suspension);

        // Add custom mass properties if feature is enabled
        #[cfg(feature = "rapier3d_030")]
        {
            chassis_entity_commands.insert(AdditionalMassProperties::Mass(config.mass));
        }

        let chassis_entity = chassis_entity_commands.id();

        // Spawn 4 wheel entities as children
        let wheel_positions = self.calculate_wheel_positions(config);
        let wheel_entities = self.spawn_wheels(commands, config, &wheel_positions)?;

        // Establish parent-child hierarchy
        commands
            .entity(chassis_entity)
            .add_children(&wheel_entities);

        Ok(chassis_entity)
    }

    /// Calculate wheel positions relative to the chassis.
    ///
    /// Returns an array of 4 Vec3 positions for the wheels in the order:
    /// [front_left, front_right, rear_left, rear_right]
    fn calculate_wheel_positions(&self, config: &VehicleConfig) -> [Vec3; 4] {
        // Use config-driven positioning with suspension travel and wheel radius
        let front_track = 1.6; // Typical front track width
        let rear_track = 1.6; // Typical rear track width
        let wheelbase = 2.7; // Typical wheelbase

        // Calculate Y offset using suspension travel and wheel radius
        let y_offset = -(config.suspension.travel + config.wheels[0].radius);

        // Calculate positions relative to chassis center
        let front_left = Vec3::new(-front_track / 2.0, y_offset, wheelbase / 2.0);
        let front_right = Vec3::new(front_track / 2.0, y_offset, wheelbase / 2.0);
        let rear_left = Vec3::new(-rear_track / 2.0, y_offset, -wheelbase / 2.0);
        let rear_right = Vec3::new(rear_track / 2.0, y_offset, -wheelbase / 2.0);

        [front_left, front_right, rear_left, rear_right]
    }

    /// Spawn wheel entities with proper components.
    ///
    /// Creates 4 wheel entities with:
    /// - Wheel marker component
    /// - SuspensionRay marker component
    /// - Kinematic-PositionBased RigidBody
    /// - Small collider for wheel collision
    /// - Transform at the specified position
    fn spawn_wheels(
        &self,
        commands: &mut Commands,
        config: &VehicleConfig,
        positions: &[Vec3; 4],
    ) -> Result<Vec<Entity>, Error> {
        let mut wheel_entities = Vec::with_capacity(4);

        for (i, position) in positions.iter().enumerate() {
            let wheel_config = &config.wheels[i];

            // Create wheel entity with proper rotation for cylinder collider
            let wheel_transform = Transform::from_translation(*position)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)); // Rotate cylinder to align with wheel axis

            let mut wheel_entity_commands = commands.spawn((
                Wheel,
                SuspensionRay::default(),
                wheel_transform,
                Name::new(format!("Wheel_{i}")),
            ));

            // Add physics components if feature is enabled
            #[cfg(feature = "rapier3d_030")]
            {
                wheel_entity_commands.insert((
                    RigidBody::KinematicPositionBased,
                    Collider::cylinder(wheel_config.width / 2.0, wheel_config.radius), // Cylinder collider
                    AdditionalMassProperties::Mass(wheel_config.mass), // Apply wheel mass
                ));
            }

            wheel_entities.push(wheel_entity_commands.id());
        }

        Ok(wheel_entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_factory_creation() {
        let factory = VehicleFactory::new();
        // Test that factory can be created
        assert_eq!(factory, VehicleFactory);
    }

    #[test]
    fn wheel_position_calculation() {
        let factory = VehicleFactory::new();
        let config = VehicleConfig::default();

        let positions = factory.calculate_wheel_positions(&config);

        // Verify we have 4 positions
        assert_eq!(positions.len(), 4);

        // Verify positions are reasonable (not all zero)
        for pos in positions.iter() {
            assert!(pos.x.abs() > 0.0 || pos.z.abs() > 0.0);
        }

        // Verify symmetry (left and right wheels should have opposite x values)
        assert_eq!(positions[0].x, -positions[1].x); // Front wheels
        assert_eq!(positions[2].x, -positions[3].x); // Rear wheels
    }

    #[test]
    fn spawn_vehicle_basic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let factory = VehicleFactory::new();
        let config = VehicleConfig::default();

        // Spawn vehicle
        let result = factory.spawn_vehicle(&mut app.world_mut().commands(), &config);

        assert!(result.is_ok());
        let spawned_entity = result.unwrap();

        // Apply commands to flush the spawned entities
        app.world_mut().flush();

        // Verify entity exists and has correct components
        let entity_exists = app.world().get_entity(spawned_entity).is_ok();
        assert!(entity_exists);
    }
}
