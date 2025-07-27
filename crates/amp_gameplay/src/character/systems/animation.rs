//! Animation state machine systems
//!
//! Handles locomotion state transitions and animation blending for humanoid characters.

use crate::character::components::{
    AnimationPlayback, AnimationSet, CharacterAnimations, Locomotion, LocomotionState,
    Velocity as CharacterVelocity,
};
use bevy::prelude::*;

/// Velocity thresholds for locomotion state transitions
#[derive(Resource)]
pub struct LocomotionThresholds {
    /// Idle threshold (below this is idle)
    pub idle: f32,
    /// Walk threshold (above idle, below this is walk)
    pub walk: f32,
    /// Run threshold (above walk, below this is run)
    pub run: f32,
    /// Sprint threshold (above run is sprint)
    pub sprint: f32,
    /// Hysteresis factor to prevent rapid state changes (0.0-1.0)
    pub hysteresis: f32,
}

impl Default for LocomotionThresholds {
    fn default() -> Self {
        Self {
            idle: 0.25,       // Oracle tuning: Relaxed movement threshold
            walk: 0.8,        // Oracle tuning: Normal walking speed threshold
            run: 2.0,         // Oracle tuning: Faster movement (>walk * 2)
            sprint: 3.5,      // Oracle tuning: Max movement (walk * sprint_multiplier)
            hysteresis: 0.85, // Slightly larger buffer
        }
    }
}

/// System that updates locomotion state based on velocity (velocity-based, not input-based)
/// This ensures animations accurately reflect character movement state
/// Characters hitting walls go to Idle when velocity drops to zero, regardless of input
/// Input is only used as a hint for walk vs run when velocity is near zero
pub fn update_locomotion_state(
    mut skeleton_query: Query<(
        &mut LocomotionState,
        &crate::character::components::ControlledBy,
        Entity,
    )>,
    velocity_query: Query<&CharacterVelocity>,
    character_query: Query<&crate::character::components::CharacterInput>,
    npc_query: Query<&crate::npc::components::NpcState>,
    thresholds: Res<LocomotionThresholds>,
    time: Res<Time>,
) {
    for (mut locomotion_state, controlled_by, skeleton_entity) in skeleton_query.iter_mut() {
        // Get velocity from the controlling character entity
        if let Ok(velocity) = velocity_query.get(controlled_by.controller_entity) {
            // Calculate speed from velocity magnitude
            let speed = velocity.linear.length();
            let horiz_speed = velocity.linear.truncate().length(); // XZ only
            info!(
                "Speed check {:?}: v=({:+.3},{:+.3},{:+.3}), 3D_speed={:.3}, horiz_speed={:.3}",
                controlled_by.controller_entity,
                velocity.linear.x,
                velocity.linear.y,
                velocity.linear.z,
                speed,
                horiz_speed
            );

            // Use the existing calculate_locomotion_state function for velocity-based transitions
            let velocity_based_state =
                calculate_locomotion_state(speed, locomotion_state.current, &thresholds);

            // When velocity is near zero, use input as a hint for intended movement state
            let new_state = if speed <= thresholds.idle {
                // At low velocity, check input for walk vs run intention
                if let Ok(character_input) = character_query.get(controlled_by.controller_entity) {
                    debug!(
                        "🔍 Animation: Low velocity ({:.3}), input move_2d={:?}, length={:.3}",
                        speed,
                        character_input.move_2d,
                        character_input.move_2d.length()
                    );
                    if character_input.move_2d.length() > 0.0 && speed > 0.05 {
                        // Input detected but low velocity - use input hint for walk vs run
                        info!(
                            "🎮 Animation: Input detected at low velocity - setting to Walk. Input: {:?}", 
                            character_input.move_2d
                        );
                        if character_input.sprint {
                            Locomotion::Run
                        } else {
                            Locomotion::Walk
                        }
                    } else {
                        // No input and low velocity - idle
                        Locomotion::Idle
                    }
                } else if let Ok(npc_state) = npc_query.get(controlled_by.controller_entity) {
                    // For NPCs at low velocity, use AI state as hint
                    match npc_state.current {
                        crate::npc::components::NpcBehaviorState::Idle => Locomotion::Idle,
                        crate::npc::components::NpcBehaviorState::Wander => Locomotion::Walk,
                        crate::npc::components::NpcBehaviorState::Flee => Locomotion::Run,
                        crate::npc::components::NpcBehaviorState::Follow => {
                            if npc_state.state_data.speed_multiplier > 1.5 {
                                Locomotion::Run
                            } else {
                                Locomotion::Walk
                            }
                        }
                        crate::npc::components::NpcBehaviorState::Interact => Locomotion::Idle,
                    }
                } else {
                    // No input or AI state - default to idle
                    Locomotion::Idle
                }
            } else {
                // At significant velocity, trust the velocity-based calculation
                velocity_based_state
            };

            debug!(
                "🎮 Animation state for skeleton {:?} (controller {:?}): speed={:.2}, state={:?}",
                skeleton_entity, controlled_by.controller_entity, speed, new_state
            );

            // Apply state transition if changed
            if new_state != locomotion_state.current {
                // Use different transition durations based on state change
                let transition_duration =
                    calculate_transition_duration(locomotion_state.current, new_state);

                info!(
                    "🎭 Animation state change for skeleton {:?}: {:?} -> {:?}",
                    skeleton_entity, locomotion_state.current, new_state
                );

                locomotion_state.transition_to(new_state, transition_duration);
            }
        } else {
            // No velocity found for this controller, default to idle
            debug!(
                "❌ No velocity found for controller entity {:?} of skeleton {:?}",
                controlled_by.controller_entity, skeleton_entity
            );

            let new_state = Locomotion::Idle;
            if new_state != locomotion_state.current {
                locomotion_state.transition_to(new_state, 0.2);
                debug!(
                    "🎭 Setting skeleton {:?} to idle (no velocity data)",
                    skeleton_entity
                );
            }
        }

        // Update transition timer
        locomotion_state.update(time.delta_secs());
    }
}

/// Calculate locomotion state based on speed with hysteresis to prevent rapid changes
pub fn calculate_locomotion_state(
    speed: f32,
    current_state: Locomotion,
    thresholds: &LocomotionThresholds,
) -> Locomotion {
    // Apply hysteresis based on current state to prevent oscillation
    let hysteresis_factor = thresholds.hysteresis;

    match current_state {
        Locomotion::Idle => {
            if speed > thresholds.idle {
                if speed <= thresholds.walk {
                    Locomotion::Walk
                } else if speed <= thresholds.run {
                    Locomotion::Run
                } else {
                    Locomotion::Sprint
                }
            } else {
                Locomotion::Idle
            }
        }
        Locomotion::Walk => {
            if speed <= thresholds.idle * hysteresis_factor {
                Locomotion::Idle
            } else if speed > thresholds.walk {
                if speed <= thresholds.run {
                    Locomotion::Run
                } else {
                    Locomotion::Sprint
                }
            } else {
                Locomotion::Walk
            }
        }
        Locomotion::Run => {
            if speed <= thresholds.walk * hysteresis_factor {
                if speed <= thresholds.idle * hysteresis_factor {
                    Locomotion::Idle
                } else {
                    Locomotion::Walk
                }
            } else if speed > thresholds.run {
                Locomotion::Sprint
            } else {
                Locomotion::Run
            }
        }
        Locomotion::Sprint => {
            if speed <= thresholds.run * hysteresis_factor {
                if speed <= thresholds.walk * hysteresis_factor {
                    if speed <= thresholds.idle * hysteresis_factor {
                        Locomotion::Idle
                    } else {
                        Locomotion::Walk
                    }
                } else {
                    Locomotion::Run
                }
            } else {
                Locomotion::Sprint
            }
        }
        // For jump/fall/land states, we'll add logic later
        // For now, transition back to ground-based states
        Locomotion::Jump | Locomotion::Fall | Locomotion::Land => {
            if speed <= thresholds.idle {
                Locomotion::Idle
            } else if speed <= thresholds.walk {
                Locomotion::Walk
            } else if speed <= thresholds.run {
                Locomotion::Run
            } else {
                Locomotion::Sprint
            }
        }
        // Turn state logic can be added later
        Locomotion::Turn => {
            if speed <= thresholds.idle {
                Locomotion::Idle
            } else if speed <= thresholds.walk {
                Locomotion::Walk
            } else if speed <= thresholds.run {
                Locomotion::Run
            } else {
                Locomotion::Sprint
            }
        }
    }
}

/// Calculate transition duration based on state change type
pub fn calculate_transition_duration(from: Locomotion, to: Locomotion) -> f32 {
    use Locomotion::*;

    match (from, to) {
        // Quick transitions between adjacent movement speeds
        (Idle, Walk) | (Walk, Idle) => 0.2,
        (Walk, Run) | (Run, Walk) => 0.15,
        (Run, Sprint) | (Sprint, Run) => 0.1,

        // Longer transitions for larger speed changes
        (Idle, Run) | (Run, Idle) => 0.3,
        (Idle, Sprint) | (Sprint, Idle) => 0.4,
        (Walk, Sprint) | (Sprint, Walk) => 0.25,

        // Special state transitions (can be refined later)
        (_, Jump) | (Jump, _) => 0.1,
        (_, Fall) | (Fall, _) => 0.05,
        (_, Land) | (Land, _) => 0.2,
        (_, Turn) | (Turn, _) => 0.15,

        // Default for any other combinations
        _ => 0.2,
    }
}

/// System that applies animation transitions and manages blend factors
/// Oracle's guidance: All animation components are on skeleton entities
pub fn apply_animation_transitions(
    skeleton_query: Query<(&LocomotionState, &CharacterAnimations)>,
    animation_sets: Res<Assets<AnimationSet>>,
    time: Res<Time>,
) {
    for (locomotion_state, character_animations) in skeleton_query.iter() {
        // Get the animation set for this character
        if let Some(animation_set) = animation_sets.get(&character_animations.animation_set) {
            // Note: Timer is updated in update_locomotion_state() to prevent double-ticking

            // For now, we're just managing the state machine
            // Actual animation player integration will be added when we implement
            // the full animation graph system with Bevy's animation player

            // Calculate blend factors for debugging/future use
            let _transition_progress = locomotion_state.transition_progress();
            let _current_weight = if locomotion_state.is_transitioning() {
                _transition_progress
            } else {
                1.0
            };
            let _previous_weight = if locomotion_state.is_transitioning() {
                1.0 - _transition_progress
            } else {
                0.0
            };

            // Log state changes for debugging
            if locomotion_state.is_transitioning() {
                debug!(
                    "Animation transition: {:?} -> {:?} (progress: {:.2})",
                    locomotion_state.previous, locomotion_state.current, _transition_progress
                );
            }

            // Get animation clips for current and previous states
            let _current_clip = animation_set.get_clip(locomotion_state.current);
            let _previous_clip = if locomotion_state.is_transitioning() {
                animation_set.get_clip(locomotion_state.previous)
            } else {
                None
            };

            // TODO: Apply animation weights to actual animation player components
            // This will be implemented when we add full animation player integration
        }
    }
}

/// Debug system to check what animation components entities actually have
/// Oracle's guidance: Animation components should be on skeleton entities
pub fn debug_animation_components(
    players: Query<Entity, With<crate::character::components::Player>>,
    humanoid_rigs: Query<&crate::character::components::HumanoidRig>,
    locomotion_query: Query<Entity, With<LocomotionState>>,
    animations_query: Query<Entity, With<CharacterAnimations>>,
    playback_query: Query<Entity, With<AnimationPlayback>>,
    player_query: Query<Entity, With<AnimationPlayer>>,
    name_query: Query<&Name>,
) {
    debug!("🔍 Animation component debug (Oracle hierarchy):");
    debug!("  - Players: {}", players.iter().count());
    debug!(
        "  - With LocomotionState: {}",
        locomotion_query.iter().count()
    );
    debug!(
        "  - With CharacterAnimations: {}",
        animations_query.iter().count()
    );
    debug!(
        "  - With AnimationPlayback: {}",
        playback_query.iter().count()
    );
    debug!("  - With AnimationPlayer: {}", player_query.iter().count());

    for player_entity in players.iter() {
        debug!("🎭 Player entity {:?} components:", player_entity);
        debug!(
            "   - Has LocomotionState: {}",
            locomotion_query.contains(player_entity)
        );
        debug!(
            "   - Has CharacterAnimations: {}",
            animations_query.contains(player_entity)
        );
        debug!(
            "   - Has AnimationPlayback: {}",
            playback_query.contains(player_entity)
        );
        debug!(
            "   - Has AnimationPlayer: {}",
            player_query.contains(player_entity)
        );

        // Check if the humanoid rig points to a skeleton with animation components
        if let Ok(rig) = humanoid_rigs.get(player_entity) {
            let skeleton_entity = rig.skeleton_entity;
            debug!("   🏗️ Skeleton entity {:?} components:", skeleton_entity);
            debug!(
                "      - Has LocomotionState: {}",
                locomotion_query.contains(skeleton_entity)
            );
            debug!(
                "      - Has CharacterAnimations: {}",
                animations_query.contains(skeleton_entity)
            );
            debug!(
                "      - Has AnimationPlayback: {}",
                playback_query.contains(skeleton_entity)
            );
            debug!(
                "      - Has AnimationPlayer: {}",
                player_query.contains(skeleton_entity)
            );

            // Also show skeleton entity name if available
            if let Ok(name) = name_query.get(skeleton_entity) {
                debug!("      - Skeleton name: {}", name);
            }
        }
    }
}

/// System that drives the AnimationPlayer based on locomotion state
/// Oracle's guidance: All animation components are now on skeleton entities
/// Uses weight-based transitions for smooth animation blending in Bevy 0.16.1
pub fn drive_animation_player(
    mut skeleton_query: Query<(
        Entity,
        &LocomotionState,
        &CharacterAnimations,
        &mut AnimationPlayback,
        &mut AnimationPlayer,
    )>,
    animation_sets: Res<Assets<AnimationSet>>,
) {
    info!(
        "🎮 Drive animation player system running with {} skeletons",
        skeleton_query.iter().len()
    );

    for (
        skeleton_entity,
        locomotion_state,
        character_animations,
        mut playback,
        mut animation_player,
    ) in skeleton_query.iter_mut()
    {
        // Timer is updated in update_locomotion_state to prevent double-ticking

        debug!(
            "🔄 Processing skeleton entity {:?} with locomotion state: {:?}",
            skeleton_entity, locomotion_state.current
        );

        // Get the animation set for this character
        if let Some(animation_set) = animation_sets.get(&character_animations.animation_set) {
            debug!(
                "✅ Found animation set for character type: {}",
                animation_set.character_type
            );

            // Calculate weights based on Oracle's formula
            let transition_progress = locomotion_state.transition_progress();
            let current_weight = if locomotion_state.is_transitioning() {
                transition_progress
            } else {
                1.0
            };
            let previous_weight = if locomotion_state.is_transitioning() {
                1.0 - transition_progress
            } else {
                0.0
            };

            debug!(
                "🎛️ Weight calculation: current={:.2}, previous={:.2}, progress={:.2}",
                current_weight, previous_weight, transition_progress
            );

            // Set weights for all locomotion variants
            for variant in Locomotion::all_variants() {
                if let Some(node_index) = animation_set.get_node_index(variant) {
                    let weight = if variant == locomotion_state.current {
                        current_weight
                    } else if locomotion_state.is_transitioning()
                        && variant == locomotion_state.previous
                    {
                        previous_weight
                    } else {
                        0.0
                    };

                    // Oracle's optimization: Only start/play animations when weight > 0
                    if weight == 0.0 {
                        // Stop the animation to avoid evaluating unnecessary clips
                        animation_player.stop(node_index);
                        debug!(
                            "⏹️ Stopped animation for {:?} (node {:?}, weight=0.0)",
                            variant, node_index
                        );
                    } else {
                        // Ensure animation is playing before setting weight
                        if animation_player.animation(node_index).is_none() {
                            // Start the animation if it's not already playing
                            animation_player
                                .play(node_index)
                                .set_weight(weight)
                                .repeat();
                            debug!(
                                "🎬 Started animation for {:?} (node {:?}, weight={:.2})",
                                variant, node_index, weight
                            );
                        } else {
                            // Apply weight to the animation node using Bevy 0.16.1 API
                            if let Some(active_animation) =
                                animation_player.animation_mut(node_index)
                            {
                                active_animation.set_weight(weight);
                            }
                        }

                        info!(
                            "⚖️ Set weight {:.2} for {:?} (node {:?}) - current_state={:?}",
                            weight, variant, node_index, locomotion_state.current
                        );
                    }
                } else {
                    warn!("❌ No node index found for {:?} in animation set", variant);
                }
            }

            // Only play animation when the actual clip changes, not on every state change
            let desired_clip = animation_set.get_clip(locomotion_state.current);
            let animation_changed = playback.current_clip.as_ref() != desired_clip;

            if animation_changed {
                if let Some(node_index) = animation_set.get_node_index(locomotion_state.current) {
                    let target_speed = target_speed(locomotion_state.current);

                    // Play the animation using Bevy 0.16.1's AnimationPlayer API
                    animation_player
                        .play(node_index)
                        .set_speed(target_speed)
                        .repeat();

                    // Update tracking state
                    playback.current_clip = desired_clip.cloned();
                    playback.wants_speed = target_speed;

                    info!(
                        "🎬 Animation clip changed: {:?} -> node={:?}, speed={:.1}",
                        locomotion_state.current, node_index, target_speed
                    );

                    // Log the actual clip path being played
                    if let Some(clip_handle) = desired_clip {
                        info!(
                            "🎭 Playing clip: {:?} for state {:?}",
                            clip_handle.path(),
                            locomotion_state.current
                        );
                    }
                } else {
                    warn!(
                        "❌ No node index found for locomotion state: {:?}",
                        locomotion_state.current
                    );
                }
            } else {
                info!(
                    "⚖️ Weights updated for {:?} - Idle weight should be 1.0, visible animation should match",
                    locomotion_state.current
                );
            }
        } else {
            warn!(
                "❌ No animation set found for handle: {:?}",
                character_animations.animation_set
            );
        }
    }
}

/// Helper function to calculate target animation speed based on locomotion state
fn target_speed(locomotion: Locomotion) -> f32 {
    match locomotion {
        Locomotion::Idle => 1.0,
        Locomotion::Walk => 1.0,
        Locomotion::Run => 1.5,
        Locomotion::Sprint => 1.8,
        Locomotion::Jump => 1.2,
        Locomotion::Fall => 1.0,
        Locomotion::Land => 1.0,
        Locomotion::Turn => 1.0,
    }
}

/// System to prevent character from shifting due to animation root motion
/// Preserves the character's world position regardless of animation changes
pub fn preserve_character_position(
    mut skeleton_query: Query<&mut Transform, With<crate::character::components::HumanoidRig>>,
) {
    for mut skeleton_transform in skeleton_query.iter_mut() {
        // Reset any unwanted root motion by keeping the skeleton at origin
        if skeleton_transform.translation.length() > 0.01 {
            debug!(
                "🔧 Correcting skeleton root motion from {:?} to origin",
                skeleton_transform.translation
            );
            skeleton_transform.translation = Vec3::ZERO;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locomotion_state_transitions() {
        let thresholds = LocomotionThresholds::default();

        // Test idle to walk transition
        assert_eq!(
            calculate_locomotion_state(1.5, Locomotion::Idle, &thresholds),
            Locomotion::Walk
        );

        // Test walk to run transition
        assert_eq!(
            calculate_locomotion_state(3.0, Locomotion::Walk, &thresholds),
            Locomotion::Run
        );

        // Test run to sprint transition
        assert_eq!(
            calculate_locomotion_state(9.0, Locomotion::Run, &thresholds),
            Locomotion::Sprint
        );

        // Test hysteresis - staying in current state near threshold
        assert_eq!(
            calculate_locomotion_state(1.8, Locomotion::Walk, &thresholds), // 0.9 * walk threshold
            Locomotion::Walk
        );
    }

    #[test]
    fn test_transition_durations() {
        // Test adjacent state transitions are quick
        assert_eq!(
            calculate_transition_duration(Locomotion::Idle, Locomotion::Walk),
            0.2
        );
        assert_eq!(
            calculate_transition_duration(Locomotion::Walk, Locomotion::Run),
            0.15
        );

        // Test larger jumps take longer
        assert!(calculate_transition_duration(Locomotion::Idle, Locomotion::Sprint) > 0.3);
    }

    #[test]
    fn test_locomotion_state_component() {
        let mut state = LocomotionState::default();

        // Test initial state
        assert_eq!(state.current, Locomotion::Idle);
        assert!(!state.is_transitioning());
        assert_eq!(state.transition_progress(), 1.0);

        // Test transition
        state.transition_to(Locomotion::Walk, 0.5);
        assert_eq!(state.current, Locomotion::Walk);
        assert_eq!(state.previous, Locomotion::Idle);
        assert!(state.is_transitioning());
        assert_eq!(state.transition_progress(), 0.0);

        // Test transition progress
        state.update(0.25); // Half way through 0.5s transition
        assert!(state.transition_progress() > 0.4 && state.transition_progress() < 0.6);

        // Test transition completion
        state.update(0.5); // Complete the transition
        assert!(!state.is_transitioning());
        assert_eq!(state.transition_progress(), 1.0);
    }
}
