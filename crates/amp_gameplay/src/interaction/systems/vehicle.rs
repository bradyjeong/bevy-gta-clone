//! Vehicle enter/exit handlers

use crate::interaction::{components::*, events::*, resources::InteractionState};
use crate::prelude::{Player, Vehicle};
use bevy::prelude::*;

/// Handle vehicle enter events
pub fn handle_vehicle_enter(
    mut commands: Commands,
    mut vehicle_enter_events: EventReader<VehicleEnterEvent>,
    mut interaction_state: ResMut<InteractionState>,
    mut player_query: Query<(Entity, &mut Transform, &mut Visibility), With<Player>>,
    vehicle_query: Query<&Transform, (With<Vehicle>, Without<Player>)>,
) {
    for event in vehicle_enter_events.read() {
        if let Ok((player_entity, mut player_transform, mut player_visibility)) =
            player_query.get_mut(event.player_entity)
        {
            if let Ok(vehicle_transform) = vehicle_query.get(event.vehicle_entity) {
                // Add InVehicle component
                commands.entity(player_entity).insert(InVehicle {
                    vehicle_entity: event.vehicle_entity,
                    seat_index: 0,
                });

                // Hide player model
                *player_visibility = Visibility::Hidden;

                // Position player at vehicle position
                player_transform.translation = vehicle_transform.translation;

                // Update interaction state
                interaction_state.player_state = PlayerState::Driving;
                interaction_state.target_vehicle = None;
                interaction_state.prompt_visible = false;

                info!("Player entered vehicle: {:?}", event.vehicle_entity);
            }
        }
    }
}

/// Handle vehicle exit events
pub fn handle_vehicle_exit(
    mut commands: Commands,
    mut vehicle_exit_events: EventReader<VehicleExitEvent>,
    mut interaction_state: ResMut<InteractionState>,
    mut player_query: Query<(Entity, &mut Transform, &mut Visibility), With<Player>>,
    vehicle_query: Query<&Transform, (With<Vehicle>, Without<Player>)>,
) {
    for event in vehicle_exit_events.read() {
        if let Ok((player_entity, mut player_transform, mut player_visibility)) =
            player_query.get_mut(event.player_entity)
        {
            if let Ok(vehicle_transform) = vehicle_query.get(event.vehicle_entity) {
                // Remove InVehicle component
                commands.entity(player_entity).remove::<InVehicle>();

                // Show player model
                *player_visibility = Visibility::Visible;

                // Position player next to vehicle
                let exit_offset = Vec3::new(2.0, 0.0, 0.0); // Exit to the right side
                player_transform.translation = vehicle_transform.translation + exit_offset;

                // Update interaction state
                interaction_state.player_state = PlayerState::Walking;
                interaction_state.target_vehicle = None;
                interaction_state.prompt_visible = false;

                info!("Player exited vehicle: {:?}", event.vehicle_entity);
            }
        }
    }
}
