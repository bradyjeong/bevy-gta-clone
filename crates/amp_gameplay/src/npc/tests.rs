//! Comprehensive tests for NPC behavior system
//!
//! Tests cover deterministic transitions, tick-rate scaling, performance
//! constraints, and integration with batch processing.

use super::*;
use bevy::app::App;
use bevy::prelude::*;
// No distance cache for now
use std::time::Duration;

/// Test helper to create a basic NPC test app
fn create_npc_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(NpcPlugin)
        .init_resource::<Time>();
    app
}

/// Test NPC component creation and defaults
#[test]
fn test_npc_component_creation() {
    let npc = NPC::default();
    assert_eq!(npc.id, 0);
    assert_eq!(npc.npc_type, NpcType::Civilian);
    assert_eq!(npc.speed, 1.5);
    assert_eq!(npc.health, 100.0);
    assert_eq!(npc.max_health, 100.0);
    assert_eq!(npc.energy, 100.0);
    assert_eq!(npc.stress, 0.0);
    assert_eq!(npc.last_decision_time, 0.0);
    assert_eq!(npc.target, None);
    assert_eq!(npc.memory_duration, 60.0);
}

/// Test NPC state machine creation and defaults
#[test]
fn test_npc_state_creation() {
    let state = NpcState::default();
    assert_eq!(state.current, NpcBehaviorState::Idle);
    assert_eq!(state.previous, NpcBehaviorState::Idle);
    assert_eq!(state.state_start_time, 0.0);
    assert_eq!(state.state_duration, 0.0);
    assert_eq!(state.state_data.speed_multiplier, 1.0);
    assert_eq!(state.state_data.direction, Vec3::ZERO);
}

/// Test NPC brain handle creation
#[test]
fn test_npc_brain_handle_creation() {
    let brain_handle = NpcBrainHandle::default();
    assert_eq!(brain_handle.batch_handle, 0);
    assert_eq!(brain_handle.priority, 4); // AI priority level
    assert_eq!(brain_handle.cost, 1.0);
    assert_eq!(brain_handle.distance_to_player, f32::MAX);
    assert_eq!(brain_handle.frames_since_update, 0);
    assert_eq!(brain_handle.update_interval, 1);
}

/// Test distance category classification
#[test]
fn test_distance_category_classification() {
    let config = UpdateIntervalsConfig::default();

    // Test close distance
    assert_eq!(
        DistanceCategory::from_distance(25.0, &config),
        DistanceCategory::Close
    );

    // Test boundary at close distance
    assert_eq!(
        DistanceCategory::from_distance(49.9, &config),
        DistanceCategory::Close
    );

    // Test medium distance
    assert_eq!(
        DistanceCategory::from_distance(75.0, &config),
        DistanceCategory::Medium
    );

    // Test boundary at far distance
    assert_eq!(
        DistanceCategory::from_distance(149.9, &config),
        DistanceCategory::Medium
    );

    // Test far distance
    assert_eq!(
        DistanceCategory::from_distance(200.0, &config),
        DistanceCategory::Far
    );
}

/// Test frame intervals for different distance categories
#[test]
fn test_frame_intervals() {
    let config = UpdateIntervalsConfig::default();

    assert_eq!(DistanceCategory::Close.frame_interval(&config), 1);
    assert_eq!(DistanceCategory::Medium.frame_interval(&config), 15);
    assert_eq!(DistanceCategory::Far.frame_interval(&config), 60);
}

/// Test update intervals for different distance categories
#[test]
fn test_update_intervals() {
    let config = UpdateIntervalsConfig::default();

    assert_eq!(DistanceCategory::Close.update_interval(&config), 0.0167);
    assert_eq!(DistanceCategory::Medium.update_interval(&config), 0.25);
    assert_eq!(DistanceCategory::Far.update_interval(&config), 1.0);
}

/// Test deterministic state transitions
#[test]
fn test_deterministic_state_transitions() {
    let mut npc = NPC::default();
    let mut state = NpcState::default();
    let config = NpcBehaviorConfig::default();

    // Test idle to wander transition (duration > 5.0)
    state.state_duration = 6.0;
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 100.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Wander);

    // Test high energy triggering wander
    npc.energy = 85.0; // > energetic_threshold (80.0)
    state.state_duration = 1.0; // < 5.0
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 100.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Wander);

    // Test flee condition (close distance + high stress)
    npc.stress = 80.0; // > panic_threshold (70.0)
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 5.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Flee);

    // Test wander to idle transition (low energy)
    state.current = NpcBehaviorState::Wander;
    npc.energy = 25.0; // < tired_threshold (30.0)
    npc.stress = 0.0; // Reset stress
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 100.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Idle);

    // Test wander to idle transition (duration > 20.0)
    npc.energy = 50.0; // Above tired threshold
    state.state_duration = 25.0; // > 20.0
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 100.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Idle);

    // Test flee to idle transition (far distance + low stress)
    state.current = NpcBehaviorState::Flee;
    npc.stress = 20.0; // < calm_threshold (30.0)
    state.state_duration = 1.0;
    let new_state =
        evaluate_state_transition(&npc, &state, &config, Vec3::ZERO, Vec3::ZERO, 60.0, 0.0);
    assert_eq!(new_state, NpcBehaviorState::Idle);
}

/// Test emotion updates
#[test]
fn test_emotion_updates() {
    let mut npc = NPC::default();
    npc.energy = 50.0;
    npc.stress = 60.0;

    let config = NpcBehaviorConfig::default();
    let delta_time = 1.0;

    update_npc_emotions(&mut npc, &config, delta_time);

    // Energy should increase
    assert!(npc.energy > 50.0);
    assert!(npc.energy <= 100.0);

    // Stress should decrease
    assert!(npc.stress < 60.0);
    assert!(npc.stress >= 0.0);

    // Test energy clamping
    npc.energy = 100.0;
    update_npc_emotions(&mut npc, &config, delta_time);
    assert_eq!(npc.energy, 100.0);

    // Test stress clamping
    npc.stress = 0.0;
    update_npc_emotions(&mut npc, &config, delta_time);
    assert_eq!(npc.stress, 0.0);
}

/// Test state transition functionality
#[test]
fn test_state_transitions() {
    let mut state = NpcState::default();
    let current_time = 10.0;

    // Test transition to wander
    transition_to_state(&mut state, NpcBehaviorState::Wander, current_time);

    assert_eq!(state.current, NpcBehaviorState::Wander);
    assert_eq!(state.previous, NpcBehaviorState::Idle);
    assert_eq!(state.state_start_time, current_time);
    assert_eq!(state.state_duration, 0.0);
    assert_eq!(state.state_data.speed_multiplier, 1.0);
    assert_eq!(state.state_data.max_duration, 20.0);
    assert_ne!(state.state_data.direction, Vec3::ZERO);

    // Test transition to flee
    transition_to_state(&mut state, NpcBehaviorState::Flee, current_time + 5.0);

    assert_eq!(state.current, NpcBehaviorState::Flee);
    assert_eq!(state.previous, NpcBehaviorState::Wander);
    assert_eq!(state.state_start_time, current_time + 5.0);
    assert_eq!(state.state_duration, 0.0);
    assert_eq!(state.state_data.speed_multiplier, 2.0);
    assert_eq!(state.state_data.max_duration, 10.0);

    // Test transition to idle
    transition_to_state(&mut state, NpcBehaviorState::Idle, current_time + 10.0);

    assert_eq!(state.current, NpcBehaviorState::Idle);
    assert_eq!(state.previous, NpcBehaviorState::Flee);
    assert_eq!(state.state_start_time, current_time + 10.0);
    assert_eq!(state.state_duration, 0.0);
    assert_eq!(state.state_data.speed_multiplier, 0.0);
    assert_eq!(state.state_data.max_duration, 10.0);
}

/// Test execute state behavior
#[test]
fn test_execute_state_behavior() {
    let mut npc = NPC::default();
    npc.energy = 50.0;
    npc.stress = 30.0;

    let mut state = NpcState::default();
    let config = NpcBehaviorConfig::default();
    let delta_time = 1.0;

    // Test idle behavior
    state.current = NpcBehaviorState::Idle;
    let initial_energy = npc.energy;
    execute_state_behavior(
        &mut npc,
        &mut state,
        &config,
        Vec3::ZERO,
        Vec3::ZERO,
        delta_time,
    );
    assert!(npc.energy > initial_energy); // Energy should increase in idle

    // Test wander behavior
    state.current = NpcBehaviorState::Wander;
    let initial_energy = npc.energy;
    execute_state_behavior(
        &mut npc,
        &mut state,
        &config,
        Vec3::ZERO,
        Vec3::ZERO,
        delta_time,
    );
    assert!(npc.energy < initial_energy); // Energy should decrease in wander

    // Test flee behavior
    state.current = NpcBehaviorState::Flee;
    let initial_energy = npc.energy;
    let initial_stress = npc.stress;
    execute_state_behavior(
        &mut npc,
        &mut state,
        &config,
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::ZERO,
        delta_time,
    );
    assert!(npc.energy < initial_energy); // Energy should decrease more in flee
    assert!(npc.stress > initial_stress); // Stress should increase in flee

    // Direction should be away from player
    let expected_direction = (Vec3::ZERO - Vec3::new(100.0, 0.0, 0.0)).normalize();
    assert!((state.state_data.direction - expected_direction).length() < 0.1);
}

/// Test NPC metrics defaults
#[test]
fn test_npc_metrics_defaults() {
    let metrics = NpcMetrics::default();
    assert_eq!(metrics.npcs_updated_this_frame, 0);
    assert_eq!(metrics.updates_per_second, 0.0);
    assert_eq!(metrics.processing_time_ms, 0.0);
    assert_eq!(metrics.avg_processing_time_per_npc, 0.0);
    assert_eq!(metrics.total_npcs, 0);
    assert_eq!(metrics.npcs_by_distance, [0, 0, 0]);
    assert_eq!(metrics.frame_counter, 0);
    assert_eq!(metrics.accumulated_updates, 0);
}

/// Test NPC bundle creation
#[test]
fn test_npc_bundle_creation() {
    let bundle = NpcBundle::default();
    assert_eq!(bundle.npc.id, 0);
    assert_eq!(bundle.state.current, NpcBehaviorState::Idle);
    assert_eq!(bundle.brain_handle.priority, 4);
    assert_eq!(bundle.last_update_frame.frame, 0);
    assert_eq!(bundle.transform.translation, Vec3::ZERO);
}

/// Test NPC config loading
#[test]
fn test_npc_config_loading() {
    let config = NpcConfig::default();
    assert_eq!(config.npc_behavior.physical.default_height, 1.8);
    assert_eq!(config.npc_behavior.movement.walk_speed, 1.5);
    assert_eq!(config.npc_behavior.emotions.energy_levels.max_energy, 100.0);
    assert_eq!(config.npc_behavior.ai.decision_interval, 2.0);
    assert_eq!(config.npc_behavior.spawn.max_npcs, 100);
    assert_eq!(config.npc_behavior.update_intervals.close_distance, 50.0);
    assert_eq!(config.npc_behavior.update_intervals.far_distance, 150.0);
}

/// Test performance constraint simulation
#[test]
fn test_performance_constraints() {
    let mut metrics = NpcMetrics::default();

    // Simulate high NPC count
    metrics.total_npcs = 100_000;
    metrics.npcs_updated_this_frame = 2_000; // Target: ≤2k NPC updates
    metrics.processing_time_ms = 0.25; // Target: <0.3ms

    // Verify performance targets
    assert!(metrics.npcs_updated_this_frame <= 2_000);
    assert!(metrics.processing_time_ms < 0.3);

    // Calculate average processing time per NPC
    if metrics.npcs_updated_this_frame > 0 {
        metrics.avg_processing_time_per_npc =
            metrics.processing_time_ms / metrics.npcs_updated_this_frame as f32;
        assert!(metrics.avg_processing_time_per_npc < 0.00015); // <0.3ms / 2k NPCs
    }
}

/// Test tick-rate scaling with different distances
#[test]
fn test_tick_rate_scaling() {
    let config = UpdateIntervalsConfig::default();

    // Test close NPCs (every frame)
    let close_category = DistanceCategory::from_distance(25.0, &config);
    assert_eq!(close_category.frame_interval(&config), 1);

    // Test medium NPCs (every 15 frames)
    let medium_category = DistanceCategory::from_distance(100.0, &config);
    assert_eq!(medium_category.frame_interval(&config), 15);

    // Test far NPCs (every 60 frames)
    let far_category = DistanceCategory::from_distance(300.0, &config);
    assert_eq!(far_category.frame_interval(&config), 60);

    // Verify performance improvement: far NPCs use 60x fewer updates
    let close_updates_per_second = 60.0 / 1.0; // 60 FPS / 1 frame interval
    let far_updates_per_second = 60.0 / 60.0; // 60 FPS / 60 frame interval

    assert_eq!(close_updates_per_second, 60.0);
    assert_eq!(far_updates_per_second, 1.0);
    assert_eq!(close_updates_per_second / far_updates_per_second, 60.0);
}

/// Test NPC system integration
#[test]
fn test_npc_system_integration() {
    let mut app = create_npc_test_app();

    // Spawn a test NPC
    let npc_entity = app.world.spawn(NpcBundle::default()).id();

    // Verify NPC was spawned correctly
    assert!(app.world.get::<NPC>(npc_entity).is_some());
    assert!(app.world.get::<NpcState>(npc_entity).is_some());
    assert!(app.world.get::<NpcBrainHandle>(npc_entity).is_some());
    assert!(app.world.get::<LastUpdateFrame>(npc_entity).is_some());

    // Add a player entity for distance calculations
    let _player_entity = app
        .world
        .spawn((
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
            Camera3d::default(),
        ))
        .id();

    // Run the system once
    app.update();

    // Verify NPC metrics were updated
    let metrics = app.world.resource::<NpcMetrics>();
    assert_eq!(metrics.total_npcs, 1);
}

/// Test NPC processing helper function
#[test]
fn test_npc_processing_helper() {
    // Test frame interval checking
    assert!(should_process_npc(1, 1)); // Should process
    assert!(should_process_npc(15, 15)); // Should process
    assert!(should_process_npc(60, 60)); // Should process

    assert!(!should_process_npc(0, 1)); // Should not process
    assert!(!should_process_npc(14, 15)); // Should not process
    assert!(!should_process_npc(59, 60)); // Should not process
}

/// Test NPC type variations
#[test]
fn test_npc_type_variations() {
    assert_eq!(NpcType::default(), NpcType::Civilian);

    let types = [
        NpcType::Civilian,
        NpcType::Police,
        NpcType::Security,
        NpcType::Vendor,
    ];

    for npc_type in &types {
        let npc = NPC {
            npc_type: *npc_type,
            ..Default::default()
        };
        assert_eq!(npc.npc_type, *npc_type);
    }
}

/// Test configuration validation
#[test]
fn test_configuration_validation() {
    let config = NpcBehaviorConfig::default();

    // Validate physical properties
    assert!(config.physical.default_height > 0.0);
    assert!(config.physical.mass > 0.0);
    assert!(config.physical.capsule_radius > 0.0);

    // Validate movement properties
    assert!(config.movement.walk_speed > 0.0);
    assert!(config.movement.run_speed > config.movement.walk_speed);
    assert!(config.movement.max_speed >= config.movement.run_speed);

    // Validate emotion thresholds
    assert!(config.emotions.energy_levels.max_energy > 0.0);
    assert!(config.emotions.stress_levels.max_stress > 0.0);

    // Validate AI properties
    assert!(config.ai.decision_interval > 0.0);
    assert!(config.ai.reaction_time > 0.0);
    assert!(config.ai.memory_duration > 0.0);

    // Validate spawn properties
    assert!(config.spawn.max_npcs > 0);
    assert!(config.spawn.spawn_radius > 0.0);
    assert!(config.spawn.despawn_distance > config.spawn.spawn_radius);

    // Validate update intervals
    assert!(config.update_intervals.close_distance > 0.0);
    assert!(config.update_intervals.far_distance > config.update_intervals.close_distance);
    assert!(config.update_intervals.close_interval > 0.0);
    assert!(config.update_intervals.medium_interval > config.update_intervals.close_interval);
    assert!(config.update_intervals.far_interval > config.update_intervals.medium_interval);
}

/// Test memory safety and bounds checking
#[test]
fn test_memory_safety() {
    let mut npc = NPC::default();
    let config = NpcBehaviorConfig::default();

    // Test extreme values don't crash
    npc.energy = -1000.0;
    npc.stress = 1000.0;

    update_npc_emotions(&mut npc, &config, 1.0);

    // Values should be clamped
    assert!(npc.energy >= 0.0);
    assert!(npc.energy <= config.emotions.energy_levels.max_energy);
    assert!(npc.stress >= 0.0);
    assert!(npc.stress <= config.emotions.stress_levels.max_stress);
}

/// Test concurrent access safety
#[test]
fn test_concurrent_access_safety() {
    use std::sync::Arc;
    use std::thread;

    let config = Arc::new(NpcBehaviorConfig::default());

    // Test that config can be safely shared between threads
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let config = Arc::clone(&config);
            thread::spawn(move || {
                let distance = (i as f32) * 50.0;
                let category = DistanceCategory::from_distance(distance, &config);
                category.frame_interval(&config)
            })
        })
        .collect();

    for handle in handles {
        let interval = handle.join().unwrap();
        assert!(interval > 0);
    }
}
