//! Animation graph setup systems
//!
//! Handles the creation of AnimationGraph assets for character animations in Bevy 0.16.1

use crate::character::components::{
    AnimationPlayback, AnimationSet, CharacterAnimations, Locomotion,
};
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle};
use bevy::prelude::*;

/// Marker so we don't set the same player twice
#[derive(Component)]
pub struct GraphInitialised;

/// System that binds AnimationGraphHandle to AnimationPlayer to fix T-pose issue
/// Oracle's guidance: Animation components are on skeleton entities
pub fn initialise_animation_players_with_graph(
    mut query: Query<
        (
            Entity,
            &AnimationGraphHandle,
            &mut AnimationPlayer,
            &CharacterAnimations,
        ),
        (Added<AnimationGraphHandle>, Without<GraphInitialised>),
    >,
    mut commands: Commands,
    animation_sets: Res<Assets<AnimationSet>>,
) {
    for (skeleton_entity, _graph_handle, mut player, character_animations) in &mut query {
        // Get the animation set and start with idle animation to prevent T-pose
        if let Some(animation_set) = animation_sets.get(&character_animations.animation_set) {
            if let Some(idle_node) = animation_set.get_node_index(Locomotion::Idle) {
                player.play(idle_node).repeat();
                info!(
                    "✅ Started idle animation on skeleton entity {:?} to prevent T-pose",
                    skeleton_entity
                );
            }
        }
        commands.entity(skeleton_entity).insert(GraphInitialised);
    }
}

/// System that sets up animation graphs for loaded character animation sets
pub fn setup_animation_graphs(
    mut animation_sets: ResMut<Assets<AnimationSet>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    debug!(
        "🎯 Animation graph setup running with {} animation sets",
        animation_sets.len()
    );

    for (handle_id, animation_set) in animation_sets.iter_mut() {
        debug!(
            "🔍 Checking animation set: {} (handle: {:?})",
            animation_set.character_type, handle_id
        );

        // Skip if graph is already set up (has node indices)
        if animation_set.get_node_index(Locomotion::Idle).is_some() {
            debug!(
                "⏩ Animation graph already set up for: {}",
                animation_set.character_type
            );
            continue;
        }

        debug!(
            "🏗️ Creating new animation graph for: {}",
            animation_set.character_type
        );

        // Create a new animation graph
        let mut graph = AnimationGraph::new();
        let mut nodes_added = 0;

        // Add each animation clip as a node in the graph
        for locomotion_state in Locomotion::all_variants() {
            debug!("🔄 Processing locomotion state: {:?}", locomotion_state);

            if let Some(clip_handle) = animation_set.get_clip(locomotion_state).cloned() {
                debug!(
                    "✅ Found clip handle for {:?}: {:?}",
                    locomotion_state, clip_handle
                );

                // Add the clip as a node to the graph with zero initial weight
                // Only idle animation gets weight 1.0, all others start at 0.0
                let initial_weight = if locomotion_state == Locomotion::Idle {
                    1.0
                } else {
                    0.0
                };
                let node_index = graph.add_clip(clip_handle, initial_weight, graph.root);

                // Store the node index in the animation set
                animation_set.set_node_index(locomotion_state, node_index);
                nodes_added += 1;

                info!(
                    "🎵 Added animation node: {:?} -> {:?}",
                    locomotion_state, node_index
                );
            } else {
                warn!(
                    "❌ Missing animation clip for locomotion state: {:?}",
                    locomotion_state
                );
            }
        }

        // Add the graph to assets and store the handle
        let graph_handle = animation_graphs.add(graph);
        animation_set.graph = graph_handle.clone();

        info!(
            "🎼 Created AnimationGraph for character: {} with {} nodes (handle: {:?})",
            animation_set.character_type, nodes_added, graph_handle
        );
    }
}

/// System that ensures skeleton entities with CharacterAnimations have the AnimationGraphHandle component
/// Oracle's guidance: Animation components are on skeleton entities
pub fn setup_animation_graph_handles(
    mut commands: Commands,
    query: Query<
        (Entity, &crate::character::components::CharacterAnimations),
        Without<AnimationGraphHandle>,
    >,
    animation_sets: Res<Assets<AnimationSet>>,
) {
    for (skeleton_entity, character_animations) in query.iter() {
        if let Some(animation_set) = animation_sets.get(&character_animations.animation_set) {
            commands
                .entity(skeleton_entity)
                .insert(AnimationGraphHandle(animation_set.graph.clone()));

            // Only insert AnimationPlayback if it doesn't already exist
            commands
                .entity(skeleton_entity)
                .try_insert(AnimationPlayback::default());

            info!(
                "🎯 Added AnimationGraphHandle and AnimationPlayback to skeleton entity {:?} for character type: {}",
                skeleton_entity, animation_set.character_type
            );
        }
    }
}
