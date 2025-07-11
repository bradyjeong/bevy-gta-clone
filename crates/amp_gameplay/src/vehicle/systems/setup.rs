//! Vehicle system setup and initialization
//!
//! Systems for initializing vehicle components and resources.

use crate::vehicle::resources::*;
use crate::{VehicleEngine, VehicleSteering, VehicleSuspension};
use bevy::prelude::*;

/// Initialize vehicle systems and resources
pub fn setup_vehicle_systems(mut commands: Commands) {
    // Insert global vehicle resources
    commands.insert_resource(VehiclePhysicsConfig::default());
    commands.insert_resource(VehicleDebugSettings::default());
    commands.insert_resource(VehicleInputState::default());
    commands.insert_resource(VehicleMetrics::default());

    info!("Vehicle systems initialized");
}

/// Setup a basic vehicle entity with all required components
pub fn spawn_basic_vehicle(
    commands: &mut Commands,
    transform: Transform,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) -> Entity {
    use crate::vehicle::components::*;

    commands
        .spawn((Mesh3d(mesh), MeshMaterial3d(material), transform))
        .insert(Vehicle::default())
        .insert(VehicleEngine::default())
        .insert(VehicleSuspension::default())
        .insert(VehicleSteering::default())
        .insert(Name::new("Vehicle"))
        .id()
}
