use amp_engine::world_streaming::*;
use amp_math::chunk_key::ChunkKey;
use bevy::prelude::*;
use config_core::types::WorldGenerationConfig;
use std::time::{Duration, Instant};

/// Smoke test for world streaming system
/// - 800m radius
/// - 2k buildings
/// - 500 vehicles
/// - Target: ≤0.5ms per streaming pass
#[test]
fn world_streaming_smoke_test() {
    let mut app = App::new();

    // Add minimal plugins for testing
    app.add_plugins((MinimalPlugins, WorldStreamingPlugin));

    // Configure world generation for smoke test
    let config = WorldGenerationConfig {
        chunk_size: 200.0,
        streaming_radius: 800.0,
        active_radius: 400.0,
        entity_limit_per_chunk: 200, // Higher limit for stress test
        ..Default::default()
    };

    // Add world streamer resource
    app.insert_resource(WorldStreamer::new(&config));

    // Add player entity
    let player_entity = app
        .world
        .spawn((
            Player,
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
        ))
        .id();

    // Run initial streaming pass
    let start_time = Instant::now();
    app.update();
    let first_pass_time = start_time.elapsed();

    println!(
        "First streaming pass: {:.3}ms",
        first_pass_time.as_secs_f32() * 1000.0
    );
    assert!(
        first_pass_time.as_secs_f32() * 1000.0 <= 0.5,
        "First pass exceeded 0.5ms"
    );

    // Move player to trigger streaming
    let mut player_transform = app
        .world
        .entity_mut(player_entity)
        .get_mut::<Transform>()
        .unwrap();
    player_transform.translation = Vec3::new(100.0, 0.0, 100.0);

    // Run multiple streaming passes to measure performance
    let mut total_time = Duration::ZERO;
    let mut max_time = Duration::ZERO;
    const NUM_PASSES: u32 = 100;

    for i in 0..NUM_PASSES {
        // Move player gradually
        let mut player_transform = app
            .world
            .entity_mut(player_entity)
            .get_mut::<Transform>()
            .unwrap();
        player_transform.translation =
            Vec3::new((i as f32 * 10.0) % 1000.0, 0.0, (i as f32 * 10.0) % 1000.0);

        let pass_start = Instant::now();
        app.update();
        let pass_time = pass_start.elapsed();

        total_time += pass_time;
        max_time = max_time.max(pass_time);

        // Ensure each pass is under 0.5ms
        let pass_time_ms = pass_time.as_secs_f32() * 1000.0;
        if pass_time_ms > 0.5 {
            println!("Pass {} exceeded 0.5ms: {:.3}ms", i, pass_time_ms);
        }
        assert!(pass_time_ms <= 0.5, "Pass {} exceeded 0.5ms target", i);
    }

    let avg_time_ms = (total_time.as_secs_f32() / NUM_PASSES as f32) * 1000.0;
    let max_time_ms = max_time.as_secs_f32() * 1000.0;

    println!("Average streaming pass: {:.3}ms", avg_time_ms);
    println!("Max streaming pass: {:.3}ms", max_time_ms);

    // Verify performance targets
    assert!(avg_time_ms <= 0.5, "Average pass time exceeded 0.5ms");
    assert!(max_time_ms <= 0.5, "Max pass time exceeded 0.5ms");

    // Check that chunks are being loaded and unloaded
    let streamer = app.world.resource::<WorldStreamer>();
    assert!(streamer.stats.chunks_loaded > 0, "No chunks were loaded");
    assert!(
        streamer.stats.chunks_unloaded > 0,
        "No chunks were unloaded"
    );

    println!("Smoke test passed!");
    println!("- Chunks loaded: {}", streamer.stats.chunks_loaded);
    println!("- Chunks unloaded: {}", streamer.stats.chunks_unloaded);
    println!("- Entities spawned: {}", streamer.stats.entities_spawned);
    println!(
        "- Entities despawned: {}",
        streamer.stats.entities_despawned
    );
}

/// Performance stress test for large-scale streaming
#[test]
fn world_streaming_stress_test() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, WorldStreamingPlugin));

    // Configure for stress test with 2k buildings, 500 vehicles
    let config = WorldGenerationConfig {
        chunk_size: 200.0,
        streaming_radius: 1000.0, // Larger radius for stress test
        active_radius: 500.0,
        entity_limit_per_chunk: 250, // Higher limit for stress test
        ..Default::default()
    };

    app.insert_resource(WorldStreamer::new(&config));

    // Add player entity
    let player_entity = app
        .world
        .spawn((
            Player,
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
        ))
        .id();

    // Simulate rapid player movement to stress test streaming
    for i in 0..50 {
        // Move player in large steps to trigger many chunk loads/unloads
        let mut player_transform = app
            .world
            .entity_mut(player_entity)
            .get_mut::<Transform>()
            .unwrap();
        player_transform.translation = Vec3::new(
            (i as f32 * 200.0) % 2000.0,
            0.0,
            (i as f32 * 200.0) % 2000.0,
        );

        let start_time = Instant::now();
        app.update();
        let update_time = start_time.elapsed();

        let update_time_ms = update_time.as_secs_f32() * 1000.0;

        // Allow slightly higher tolerance for stress test
        if update_time_ms > 1.0 {
            println!("Stress test iteration {} took {:.3}ms", i, update_time_ms);
        }

        // Ensure we don't exceed 1ms even under stress
        assert!(
            update_time_ms <= 1.0,
            "Stress test iteration {} exceeded 1ms",
            i
        );
    }

    let streamer = app.world.resource::<WorldStreamer>();

    println!("Stress test passed!");
    println!("- Chunks loaded: {}", streamer.stats.chunks_loaded);
    println!("- Chunks unloaded: {}", streamer.stats.chunks_unloaded);
    println!(
        "- Peak update time: {:.3}ms",
        streamer.stats.peak_update_time_ms
    );
    println!(
        "- Average update time: {:.3}ms",
        streamer.stats.average_update_time_ms
    );

    // Verify substantial activity occurred
    assert!(
        streamer.stats.chunks_loaded >= 20,
        "Insufficient chunks loaded in stress test"
    );
    assert!(
        streamer.stats.chunks_unloaded >= 10,
        "Insufficient chunks unloaded in stress test"
    );
}

/// Test chunk key generation and Morton encoding
#[test]
fn test_chunk_key_performance() {
    let config = WorldGenerationConfig::default();
    let streamer = WorldStreamer::new(&config);

    let start_time = Instant::now();

    // Generate chunk keys for large area
    let mut chunk_keys = Vec::new();
    for x in -50..50 {
        for z in -50..50 {
            let world_pos = Vec3::new(x as f32 * 200.0, 0.0, z as f32 * 200.0);
            let chunk_key = streamer.get_chunk_key(world_pos);
            chunk_keys.push(chunk_key);
        }
    }

    let generation_time = start_time.elapsed();
    println!(
        "Generated {} chunk keys in {:.3}ms",
        chunk_keys.len(),
        generation_time.as_secs_f32() * 1000.0
    );

    // Test Morton encoding performance
    let morton_start = Instant::now();
    let mut morton_codes = Vec::new();
    for chunk_key in &chunk_keys {
        morton_codes.push(chunk_key.morton_code());
    }
    let morton_time = morton_start.elapsed();

    println!(
        "Generated {} Morton codes in {:.3}ms",
        morton_codes.len(),
        morton_time.as_secs_f32() * 1000.0
    );

    // Verify performance targets
    assert!(
        generation_time.as_secs_f32() * 1000.0 < 1.0,
        "Chunk key generation too slow"
    );
    assert!(
        morton_time.as_secs_f32() * 1000.0 < 1.0,
        "Morton encoding too slow"
    );
}

/// Test streaming radius calculations
#[test]
fn test_streaming_radius_calculations() {
    let config = WorldGenerationConfig {
        chunk_size: 200.0,
        streaming_radius: 800.0,
        active_radius: 400.0,
        ..Default::default()
    };

    let mut streamer = WorldStreamer::new(&config);
    streamer.player_position = Vec3::new(0.0, 0.0, 0.0);

    // Test streaming radius
    let origin_chunk = ChunkKey::new(0, 0);
    assert!(streamer.is_in_streaming_radius(&origin_chunk));

    let far_chunk = ChunkKey::new(10, 10); // ~2800 units away
    assert!(!streamer.is_in_streaming_radius(&far_chunk));

    // Test active radius
    let nearby_chunk = ChunkKey::new(1, 1); // ~283 units away
    assert!(streamer.is_in_active_radius(&nearby_chunk));

    let medium_chunk = ChunkKey::new(3, 3); // ~849 units away
    assert!(!streamer.is_in_active_radius(&medium_chunk));

    // Test streaming chunks generation
    let start_time = Instant::now();
    let streaming_chunks = streamer.get_streaming_chunks();
    let generation_time = start_time.elapsed();

    println!(
        "Generated {} streaming chunks in {:.3}ms",
        streaming_chunks.len(),
        generation_time.as_secs_f32() * 1000.0
    );

    assert!(
        !streaming_chunks.is_empty(),
        "No streaming chunks generated"
    );
    assert!(
        streaming_chunks.contains(&origin_chunk),
        "Origin chunk not in streaming set"
    );
    assert!(
        generation_time.as_secs_f32() * 1000.0 < 0.1,
        "Streaming chunks generation too slow"
    );
}

/// Test entity limit enforcement
#[test]
fn test_entity_limit_enforcement() {
    let config = WorldGenerationConfig {
        entity_limit_per_chunk: 10,
        ..Default::default()
    };

    let mut streamer = WorldStreamer::new(&config);
    let chunk_key = ChunkKey::new(0, 0);

    // Add entities up to limit
    let mut entities = Vec::new();
    for i in 0..10 {
        entities.push(Entity::from_raw(i));
    }

    streamer.mark_chunk_loaded(chunk_key, entities);

    // Check that chunk is at limit
    let chunk_data = streamer.loaded_chunks.get(&chunk_key).unwrap();
    assert!(!chunk_data.can_add_entity(streamer.entity_limit_per_chunk));

    // Add one more entity should still work but update the count
    let mut chunk_data = streamer.loaded_chunks.get_mut(&chunk_key).unwrap();
    chunk_data.add_entity(Entity::from_raw(10));

    // Now it should be over limit
    assert!(!chunk_data.can_add_entity(streamer.entity_limit_per_chunk));
    assert_eq!(chunk_data.entity_count, 11);
}

/// Test batch processing integration
#[test]
fn test_batch_processing_integration() {
    let chunk_key = ChunkKey::new(0, 0);
    let generator = ChunkContentGenerator::new(chunk_key);

    assert_eq!(generator.generation_phase, GenerationPhase::NotStarted);
    assert_eq!(generator.total_entities(), 0);

    // Test phase progression
    let mut test_generator = generator;
    test_generator.buildings_spawned = 5;
    test_generator.vehicles_spawned = 3;
    test_generator.npcs_spawned = 2;
    test_generator.trees_spawned = 8;

    assert_eq!(test_generator.total_entities(), 18);

    // Test entity types
    let chunk_entity = ChunkEntity {
        chunk_key,
        entity_type: EntityType::Building,
    };

    assert_eq!(chunk_entity.entity_type, EntityType::Building);
    assert_eq!(chunk_entity.chunk_key, chunk_key);
}
