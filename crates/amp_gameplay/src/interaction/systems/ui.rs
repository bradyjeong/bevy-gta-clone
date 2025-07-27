//! UI system for interaction prompts

use bevy::prelude::*;

use crate::interaction::{components::*, events::*, resources::InteractionState};

/// Update interaction prompts based on events
pub fn update_interaction_prompts(
    mut commands: Commands,
    mut interaction_events: EventReader<InteractionPromptEvent>,
    mut interaction_state: ResMut<InteractionState>,
    mut ui_query: Query<(Entity, &mut Text, &mut Visibility), With<InteractionPrompt>>,
    asset_server: Res<AssetServer>,
) {
    for event in interaction_events.read() {
        if event.visible {
            // Show or update prompt
            if let Ok((ui_entity, mut text, mut visibility)) = ui_query.single_mut() {
                // Update existing prompt
                text.0 = event.prompt_text.clone();
                *visibility = Visibility::Visible;
            } else {
                // Create new prompt
                let font = asset_server.load("fonts/FiraSans-Bold.ttf");

                commands.spawn((
                    Text::new(event.prompt_text.clone()),
                    TextFont {
                        font,
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(100.0),
                        left: Val::Percent(50.0),
                        ..default()
                    },
                    InteractionPrompt {
                        prompt_text: event.prompt_text.clone(),
                        target_entity: event.target_entity,
                        visible: true,
                    },
                ));
            }

            interaction_state.prompt_visible = true;
        } else {
            // Hide prompt
            if let Ok((ui_entity, text, mut visibility)) = ui_query.single_mut() {
                *visibility = Visibility::Hidden;
            }

            interaction_state.prompt_visible = false;
        }
    }
}

/// System to clean up interaction prompts when no longer needed
pub fn cleanup_interaction_prompts(
    mut commands: Commands,
    interaction_state: Res<InteractionState>,
    ui_query: Query<Entity, With<InteractionPrompt>>,
) {
    if !interaction_state.prompt_visible && interaction_state.target_vehicle.is_none() {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
