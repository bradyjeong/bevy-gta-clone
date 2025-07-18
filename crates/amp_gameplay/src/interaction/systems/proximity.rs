//! Proximity detection system for vehicle interactions

use crate::character::components::Player;
use crate::interaction::{
    components::*, events::InteractionPromptEvent, resources::InteractionState,
};
use crate::vehicle::components::PhysicsVehicle;
use bevy::prelude::*;

/// Detect nearby vehicles that can be interacted with
pub fn detect_vehicle_proximity(
    mut interaction_state: ResMut<InteractionState>,
    mut interaction_events: EventWriter<InteractionPromptEvent>,
    player_query: Query<&Transform, (With<Player>, Without<PhysicsVehicle>)>,
    vehicle_query: Query<(Entity, &Transform, &VehicleInteraction), With<PhysicsVehicle>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let mut closest_vehicle = None;
    let mut closest_distance = f32::MAX;

    // Only check for vehicles if player is walking
    if !matches!(interaction_state.player_state, PlayerState::Walking) {
        return;
    }

    // Find closest interactable vehicle
    for (entity, vehicle_transform, vehicle_interaction) in vehicle_query.iter() {
        // Skip occupied vehicles
        if vehicle_interaction.occupied {
            continue;
        }

        let vehicle_pos = vehicle_transform.translation;
        let distance = (player_pos - vehicle_pos).length();

        if distance <= vehicle_interaction.radius && distance < closest_distance {
            closest_distance = distance;
            closest_vehicle = Some(entity);
        }
    }

    // Update interaction state
    let previous_target = interaction_state.target_vehicle;
    interaction_state.target_vehicle = closest_vehicle;
    interaction_state.target_distance = closest_distance;

    // Send prompt events when target changes
    match (previous_target, closest_vehicle) {
        (None, Some(entity)) => {
            // New target found
            interaction_events.write(InteractionPromptEvent {
                visible: true,
                prompt_text: "F - Enter Vehicle".to_string(),
                target_entity: entity,
            });
            interaction_state.prompt_visible = true;
        }
        (Some(_), None) => {
            // Target lost
            interaction_events.write(InteractionPromptEvent {
                visible: false,
                prompt_text: String::new(),
                target_entity: Entity::PLACEHOLDER,
            });
            interaction_state.prompt_visible = false;
        }
        (Some(old_entity), Some(new_entity)) if old_entity != new_entity => {
            // Target changed
            interaction_events.write(InteractionPromptEvent {
                visible: true,
                prompt_text: "F - Enter Vehicle".to_string(),
                target_entity: new_entity,
            });
        }
        _ => {
            // No change or both None
        }
    }
}
