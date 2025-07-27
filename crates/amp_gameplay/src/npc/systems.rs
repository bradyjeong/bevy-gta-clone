//! NPC behavior systems
//!
//! Implements the main NPC brain system with distance-based tick rates,
//! batch processing integration, and finite state machine logic.

use crate::npc::{components::*, config::*};
use bevy::prelude::*;
// Simple distance calculation without cache for now
use std::time::Instant;

/// Main NPC brain system that processes NPC behavior
pub fn npc_brain_system(
    commands: Commands,
    mut npcs: Query<(
        Entity,
        &mut NPC,
        &mut NpcState,
        &mut NpcBrainHandle,
        &mut LastUpdateFrame,
        &Transform,
    )>,

    mut npc_metrics: ResMut<NpcMetrics>,
    npc_config: Res<NpcConfig>,
    time: Res<Time>,
    query_player: Query<&Transform, (With<Camera>, Without<NPC>)>,
) {
    let start_time = Instant::now();
    let current_frame = time.elapsed_secs_f64() as u64;
    let current_time = time.elapsed_secs();
    let delta_time = time.delta_secs();

    // Reset frame metrics
    npc_metrics.npcs_updated_this_frame = 0;
    npc_metrics.total_npcs = npcs.iter().count() as u32;
    npc_metrics.npcs_by_distance.fill(0);

    // Get player position for distance calculations
    let player_pos = if let Ok(transform) = query_player.single() {
        transform.translation
    } else {
        Vec3::ZERO
    };

    // Process NPCs with distance-based tick rates (with performance budget)
    let mut npcs_processed = 0;
    for (entity, mut npc, mut state, mut brain_handle, mut last_update, transform) in
        npcs.iter_mut()
    {
        // Performance budget: limit NPCs processed per frame
        if npcs_processed >= MAX_NPCS_PER_FRAME {
            break;
        }
        // Calculate distance to player
        let npc_pos = transform.translation;
        let distance = (npc_pos - player_pos).length();

        // Update distance tracking
        brain_handle.distance_to_player = distance;

        // Determine distance category and update interval
        let distance_category =
            DistanceCategory::from_distance(distance, &npc_config.npc_behavior.update_intervals);
        let frame_interval =
            distance_category.frame_interval(&npc_config.npc_behavior.update_intervals);

        // Update distance category metrics
        match distance_category {
            DistanceCategory::Close => npc_metrics.npcs_by_distance[0] += 1,
            DistanceCategory::Medium => npc_metrics.npcs_by_distance[1] += 1,
            DistanceCategory::Far => npc_metrics.npcs_by_distance[2] += 1,
        }

        // Update frame tracking
        brain_handle.frames_since_update += 1;
        brain_handle.update_interval = frame_interval;

        // Check if this NPC should be updated this frame
        if brain_handle.frames_since_update >= frame_interval {
            // Process the NPC brain update
            process_npc_brain(
                &mut npc,
                &mut state,
                &mut brain_handle,
                &mut last_update,
                &npc_config.npc_behavior,
                player_pos,
                npc_pos,
                distance,
                current_frame,
                current_time,
                delta_time,
            );

            // Reset frame counter
            brain_handle.frames_since_update = 0;
            npc_metrics.npcs_updated_this_frame += 1;
            npcs_processed += 1;
        }
    }

    // Update processing time metrics
    let processing_time = start_time.elapsed();
    npc_metrics.processing_time_ms = processing_time.as_secs_f32() * 1000.0;
    npc_metrics.avg_processing_time_per_npc = if npc_metrics.npcs_updated_this_frame > 0 {
        npc_metrics.processing_time_ms / npc_metrics.npcs_updated_this_frame as f32
    } else {
        0.0
    };
}

/// Simple performance budget tracking
const MAX_NPCS_PER_FRAME: u32 = 200; // Limit NPCs processed per frame for performance

/// Process individual NPC brain logic
fn process_npc_brain(
    npc: &mut NPC,
    state: &mut NpcState,
    brain_handle: &mut NpcBrainHandle,
    last_update: &mut LastUpdateFrame,
    config: &NpcBehaviorConfig,
    player_pos: Vec3,
    npc_pos: Vec3,
    distance: f32,
    current_frame: u64,
    current_time: f32,
    delta_time: f32,
) {
    // Update timing
    last_update.frame = current_frame;
    last_update.time = current_time;
    state.state_duration += delta_time;

    // Update NPC energy and stress
    update_npc_emotions(npc, config, delta_time);

    // Evaluate state transitions
    let new_state = evaluate_state_transition(
        npc,
        state,
        config,
        player_pos,
        npc_pos,
        distance,
        current_time,
    );

    // Handle state change
    if new_state != state.current {
        transition_to_state(state, new_state, current_time);
    }

    // Execute current state behavior
    execute_state_behavior(npc, state, config, player_pos, npc_pos, delta_time);

    // Update decision timing
    npc.last_decision_time = current_time;
}

/// Update NPC emotional state
fn update_npc_emotions(npc: &mut NPC, config: &NpcBehaviorConfig, delta_time: f32) {
    // Update energy
    if npc.energy < config.emotions.energy_levels.max_energy {
        npc.energy += config.emotions.energy_levels.energy_recovery_rate * delta_time;
        npc.energy = npc.energy.min(config.emotions.energy_levels.max_energy);
    }

    // Update stress (gradually decrease over time)
    if npc.stress > 0.0 {
        npc.stress -= config.emotions.stress_levels.stress_recovery_rate * delta_time;
        npc.stress = npc.stress.max(0.0);
    }
}

/// Evaluate state transitions based on current conditions
fn evaluate_state_transition(
    npc: &NPC,
    state: &NpcState,
    config: &NpcBehaviorConfig,
    player_pos: Vec3,
    npc_pos: Vec3,
    distance: f32,
    current_time: f32,
) -> NpcBehaviorState {
    // Check for flee condition (player too close and high stress)
    if distance < 10.0 && npc.stress > config.emotions.stress_levels.panic_threshold {
        return NpcBehaviorState::Flee;
    }

    // Check current state duration and conditions
    match state.current {
        NpcBehaviorState::Idle => {
            // Transition to wander after some time or if energy is high
            if state.state_duration > 5.0
                || npc.energy > config.emotions.energy_levels.energetic_threshold
            {
                NpcBehaviorState::Wander
            } else {
                NpcBehaviorState::Idle
            }
        }
        NpcBehaviorState::Wander => {
            // Transition to idle if tired or after wandering for a while
            if npc.energy < config.emotions.energy_levels.tired_threshold
                || state.state_duration > 20.0
            {
                NpcBehaviorState::Idle
            } else {
                NpcBehaviorState::Wander
            }
        }
        NpcBehaviorState::Flee => {
            // Transition back to idle if far enough away and stress is low
            if distance > 50.0 && npc.stress < config.emotions.stress_levels.calm_threshold {
                NpcBehaviorState::Idle
            } else {
                NpcBehaviorState::Flee
            }
        }
        _ => state.current, // Future states
    }
}

/// Transition to a new state
fn transition_to_state(state: &mut NpcState, new_state: NpcBehaviorState, current_time: f32) {
    state.previous = state.current;
    state.current = new_state;
    state.state_start_time = current_time;
    state.state_duration = 0.0;

    // Reset state data
    state.state_data = StateData::default();

    // Set state-specific data
    match new_state {
        NpcBehaviorState::Idle => {
            state.state_data.speed_multiplier = 0.0;
            state.state_data.max_duration = 10.0;
        }
        NpcBehaviorState::Wander => {
            state.state_data.speed_multiplier = 1.0;
            state.state_data.max_duration = 20.0;
            // Set random wander direction
            state.state_data.direction = Vec3::new(
                (rand::random::<f32>() - 0.5) * 2.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 2.0,
            )
            .normalize_or_zero();
        }
        NpcBehaviorState::Flee => {
            state.state_data.speed_multiplier = 2.0;
            state.state_data.max_duration = 10.0;
        }
        _ => {}
    }
}

/// Execute behavior for current state
fn execute_state_behavior(
    npc: &mut NPC,
    state: &mut NpcState,
    config: &NpcBehaviorConfig,
    player_pos: Vec3,
    npc_pos: Vec3,
    delta_time: f32,
) {
    match state.current {
        NpcBehaviorState::Idle => {
            // Idle behavior - just recover energy
            npc.energy += config.emotions.energy_levels.energy_recovery_rate * delta_time * 0.5;
        }
        NpcBehaviorState::Wander => {
            // Wander behavior - move in random direction
            npc.energy -= config.emotions.energy_levels.energy_drain_rate * delta_time;

            // Update timer and potentially change direction
            state.state_data.timer += delta_time;
            if state.state_data.timer > 3.0 {
                state.state_data.timer = 0.0;
                state.state_data.direction = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 2.0,
                    0.0,
                    (rand::random::<f32>() - 0.5) * 2.0,
                )
                .normalize_or_zero();
            }
        }
        NpcBehaviorState::Flee => {
            // Flee behavior - move away from player
            npc.energy -= config.emotions.energy_levels.energy_drain_rate * delta_time * 2.0;
            npc.stress += config.emotions.stress_levels.stress_buildup_rate * delta_time;

            // Set direction away from player
            let flee_direction = (npc_pos - player_pos).normalize_or_zero();
            state.state_data.direction = flee_direction;
        }
        _ => {}
    }

    // Clamp values
    npc.energy = npc
        .energy
        .clamp(0.0, config.emotions.energy_levels.max_energy);
    npc.stress = npc
        .stress
        .clamp(0.0, config.emotions.stress_levels.max_stress);
}

/// System to collect and update NPC metrics
pub fn npc_metrics_system(mut npc_metrics: ResMut<NpcMetrics>, time: Res<Time>) {
    // Update frame counter
    npc_metrics.frame_counter += 1;

    // Update accumulated updates
    npc_metrics.accumulated_updates += npc_metrics.npcs_updated_this_frame;

    // Calculate updates per second (rolling average over 60 frames)
    if npc_metrics.frame_counter % 60 == 0 {
        let frames_per_second = 60.0;
        let updates_in_timeframe = npc_metrics.accumulated_updates;
        npc_metrics.updates_per_second = updates_in_timeframe as f32 / frames_per_second;
        npc_metrics.accumulated_updates = 0;
    }

    // Performance warning if processing time exceeds target
    if npc_metrics.processing_time_ms > 0.3 && npc_metrics.total_npcs > 0 {
        if npc_metrics.frame_counter % 300 == 0 {
            // Log every 5 seconds
            warn!(
                "NPC processing time {:.3}ms exceeds target 0.3ms with {} NPCs ({} updated)",
                npc_metrics.processing_time_ms,
                npc_metrics.total_npcs,
                npc_metrics.npcs_updated_this_frame
            );
        }
    }
}

/// Helper function to check if an NPC should be processed this frame
pub fn should_process_npc(frames_since_update: u32, frame_interval: u32) -> bool {
    frames_since_update >= frame_interval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::config::*;

    #[test]
    fn test_distance_category_classification() {
        let config = UpdateIntervalsConfig::default();

        assert_eq!(
            DistanceCategory::from_distance(25.0, &config),
            DistanceCategory::Close
        );
        assert_eq!(
            DistanceCategory::from_distance(75.0, &config),
            DistanceCategory::Medium
        );
        assert_eq!(
            DistanceCategory::from_distance(200.0, &config),
            DistanceCategory::Far
        );
    }

    #[test]
    fn test_frame_intervals() {
        let config = UpdateIntervalsConfig::default();

        assert_eq!(DistanceCategory::Close.frame_interval(&config), 1);
        assert_eq!(DistanceCategory::Medium.frame_interval(&config), 15);
        assert_eq!(DistanceCategory::Far.frame_interval(&config), 60);
    }

    #[test]
    fn test_state_transition_logic() {
        let mut npc = NPC::default();
        let mut state = NpcState::default();
        let config = NpcBehaviorConfig::default();

        // Test idle to wander transition
        state.state_duration = 6.0; // > 5.0
        let new_state =
            evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 100.0, 0.0);
        assert_eq!(new_state, NpcBehaviorState::Wander);

        // Test flee condition
        npc.stress = 80.0; // > panic_threshold
        let new_state =
            evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 5.0, 0.0);
        assert_eq!(new_state, NpcBehaviorState::Flee);
    }

    #[test]
    fn test_emotion_updates() {
        let mut npc = NPC::default();
        npc.energy = 50.0;
        npc.stress = 60.0;

        let config = NpcBehaviorConfig::default();
        update_npc_emotions(&mut npc, &config, 1.0);

        // Energy should increase
        assert!(npc.energy > 50.0);
        // Stress should decrease
        assert!(npc.stress < 60.0);
    }

    #[test]
    fn test_npc_metrics_default() {
        let metrics = NpcMetrics::default();
        assert_eq!(metrics.npcs_updated_this_frame, 0);
        assert_eq!(metrics.total_npcs, 0);
        assert_eq!(metrics.updates_per_second, 0.0);
    }
}
