//! Input handling system for vehicle interaction

use crate::interaction::{components::*, events::*, resources::InteractionState};
use crate::prelude::Player;
use bevy::prelude::*;

/// Handle F key input for vehicle interaction
pub fn handle_interaction_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    interaction_state: Res<InteractionState>,
    mut vehicle_enter_events: EventWriter<VehicleEnterEvent>,
    mut vehicle_exit_events: EventWriter<VehicleExitEvent>,
    player_query: Query<Entity, (With<Player>, Without<InVehicle>)>,
    vehicle_player_query: Query<(Entity, &InVehicle), With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        match interaction_state.player_state {
            PlayerState::Walking => {
                // Try to enter vehicle
                if let Some(target_vehicle) = interaction_state.target_vehicle {
                    if let Ok(player_entity) = player_query.single() {
                        vehicle_enter_events.write(VehicleEnterEvent {
                            player_entity,
                            vehicle_entity: target_vehicle,
                        });
                    }
                }
            }
            PlayerState::Driving => {
                // Try to exit vehicle
                if let Ok((player_entity, in_vehicle)) = vehicle_player_query.single() {
                    vehicle_exit_events.write(VehicleExitEvent {
                        player_entity,
                        vehicle_entity: in_vehicle.vehicle_entity,
                        exit_position: Vec3::new(0.0, 0.0, 0.0), // Will be calculated by the handler
                    });
                }
            }
        }
    }
}
