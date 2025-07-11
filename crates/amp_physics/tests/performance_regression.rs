//! Performance regression tests for physics systems.
//!
//! These tests ensure that physics performance remains within acceptable bounds
//! and catches performance regressions before they reach production.

use amp_physics::*;
use bevy::prelude::*;
use std::time::Instant;

/// Test that 10 vehicles can be simulated within 16ms per frame.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn physics_performance_10_vehicles_60fps() {
    let mut app = create_test_app();

    // Spawn 10 test vehicles
    for i in 0..10 {
        spawn_test_vehicle(&mut app, Vec3::new(i as f32 * 5.0, 1.0, 0.0));
    }

    // Configure benchmark
    let benchmark_config = BenchmarkConfig {
        enabled: true,
        vehicle_count: 10,
        frame_count: 60,
        target_cpu_time: 16.0, // 60 FPS target
        name: "performance_regression_test".to_string(),
    };
    app.insert_resource(benchmark_config);

    // Run simulation
    let start_time = Instant::now();
    for _ in 0..60 {
        app.update();
    }
    let total_time = start_time.elapsed();

    // Verify performance
    let average_frame_time = total_time.as_secs_f32() * 1000.0 / 60.0;
    assert!(
        average_frame_time < 16.0,
        "Physics simulation too slow: {average_frame_time:.2}ms per frame (target: 16ms)"
    );

    // Check benchmark results
    let benchmark_results = app.world().resource::<BenchmarkResults>();
    assert!(
        benchmark_results.passed,
        "Benchmark failed: average CPU time {:.2}ms > target {:.2}ms",
        benchmark_results.average_cpu_time, 16.0
    );
}

/// Test that physics time maintains consistent 60Hz timestep.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn physics_time_consistency() {
    let mut app = create_test_app();

    // Spawn a test vehicle
    spawn_test_vehicle(&mut app, Vec3::ZERO);

    let mut physics_time = app.world_mut().resource_mut::<PhysicsTime>();
    physics_time.reset();

    // Simulate variable frame times
    let frame_times = [0.016, 0.020, 0.013, 0.025, 0.015]; // Variable frame times

    for &frame_time in &frame_times {
        // Simulate frame time
        physics_time.update(frame_time);

        // Verify timestep remains fixed
        assert_eq!(physics_time.fixed_timestep, 1.0 / 60.0);

        // Verify accumulator behavior
        if physics_time.should_step() {
            let steps = physics_time.steps_needed();
            assert!(steps <= physics_time.max_steps);

            for _ in 0..steps {
                physics_time.consume_step();
            }
        }
    }
}

/// Test that suspension system performance is within bounds.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn suspension_system_performance() {
    let mut app = create_test_app();

    // Spawn multiple vehicles with suspension
    for i in 0..20 {
        spawn_test_vehicle(&mut app, Vec3::new(i as f32 * 2.0, 1.0, 0.0));
    }

    // Measure suspension system performance
    let start_time = Instant::now();
    for _ in 0..100 {
        app.update();
    }
    let total_time = start_time.elapsed();

    let average_frame_time = total_time.as_secs_f32() * 1000.0 / 100.0;
    assert!(
        average_frame_time < 20.0,
        "Suspension system too slow: {average_frame_time:.2}ms per frame (target: <20ms for 20 vehicles)"
    );
}

/// Test that debug rendering doesn't significantly impact performance.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn debug_rendering_performance_impact() {
    let mut app = create_test_app();

    // Spawn test vehicles
    for i in 0..5 {
        spawn_test_vehicle(&mut app, Vec3::new(i as f32 * 3.0, 1.0, 0.0));
    }

    // Measure performance without debug rendering
    let start_time = Instant::now();
    for _ in 0..50 {
        app.update();
    }
    let time_without_debug = start_time.elapsed();

    // Enable debug rendering
    let mut debug_config = app.world_mut().resource_mut::<DebugConfig>();
    debug_config.enabled = true;
    debug_config.show_suspension_rays = true;
    debug_config.show_force_vectors = true;
    debug_config.show_contact_points = true;

    let mut physics_config = app.world_mut().resource_mut::<PhysicsConfig>();
    physics_config.debug_rendering = true;

    // Measure performance with debug rendering
    let start_time = Instant::now();
    for _ in 0..50 {
        app.update();
    }
    let time_with_debug = start_time.elapsed();

    // Verify debug rendering doesn't add more than 5ms overhead
    let overhead = time_with_debug.as_secs_f32() - time_without_debug.as_secs_f32();
    let overhead_per_frame = overhead * 1000.0 / 50.0;

    assert!(
        overhead_per_frame < 5.0,
        "Debug rendering overhead too high: {overhead_per_frame:.2}ms per frame (target: <5ms)"
    );
}

/// Test memory usage remains reasonable with many vehicles.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn memory_usage_stress_test() {
    let mut app = create_test_app();

    // Spawn many vehicles
    for i in 0..50 {
        spawn_test_vehicle(&mut app, Vec3::new(i as f32 * 2.0, 1.0, 0.0));
    }

    // Run simulation to populate systems
    for _ in 0..10 {
        app.update();
    }

    // Check entity counts
    let suspension_count = {
        let world = app.world_mut();
        let mut query = world.query::<&SuspensionRay>();
        query.iter(world).count()
    };

    let wheel_count = {
        let world = app.world_mut();
        let mut query = world.query::<&WheelState>();
        query.iter(world).count()
    };

    assert_eq!(suspension_count, 50 * 4); // 4 wheels per vehicle
    assert_eq!(wheel_count, 50 * 4);

    // Verify systems can handle the load
    let start_time = Instant::now();
    for _ in 0..30 {
        app.update();
    }
    let total_time = start_time.elapsed();

    let average_frame_time = total_time.as_secs_f32() * 1000.0 / 30.0;
    assert!(
        average_frame_time < 30.0,
        "Memory stress test failed: {average_frame_time:.2}ms per frame with 50 vehicles (target: <30ms)"
    );
}

/// Test that physics configuration changes are applied correctly.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn physics_configuration_changes() {
    let mut app = create_test_app();

    // Test initial configuration
    let physics_config = app.world().resource::<PhysicsConfig>();
    assert!(physics_config.enabled);
    assert_eq!(physics_config.timestep_hz, 60.0);

    let physics_time = app.world().resource::<PhysicsTime>();
    assert_eq!(physics_time.fixed_timestep, 1.0 / 60.0);

    // Change configuration
    {
        let mut physics_config = app.world_mut().resource_mut::<PhysicsConfig>();
        physics_config.timestep_hz = 30.0;
        physics_config.max_steps_per_frame = 2;
    }

    // Run a frame to apply changes
    app.update();

    // Verify changes are applied
    let physics_time = app.world().resource::<PhysicsTime>();
    assert_eq!(physics_time.fixed_timestep, 1.0 / 30.0);
    assert_eq!(physics_time.max_steps, 2);
}

/// Helper function to create a test app with physics systems.
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::window::WindowPlugin>()
            .disable::<bevy::render::RenderPlugin>()
            .disable::<bevy::winit::WinitPlugin>(),
    )
    .add_plugins(PhysicsPlugin::default())
    .init_resource::<BenchmarkResults>();

    // Disable debug rendering for performance tests
    let mut debug_config = app.world_mut().resource_mut::<DebugConfig>();
    debug_config.enabled = false;

    app
}

/// Helper function to spawn a test vehicle with full physics setup.
fn spawn_test_vehicle(app: &mut App, position: Vec3) {
    let vehicle_transform = Transform::from_translation(position);

    // Spawn vehicle entity
    let _vehicle_entity = app
        .world_mut()
        .spawn((
            vehicle_transform,
            GlobalTransform::from(vehicle_transform),
            Engine::default(),
            Transmission::default(),
            Steering::default(),
            Brakes::default(),
            VehicleInput::default(),
            Suspension::default(),
        ))
        .id();

    // Spawn wheel entities with suspension
    let wheel_positions = [
        Vec3::new(-1.0, 0.0, 1.5),  // Front left
        Vec3::new(1.0, 0.0, 1.5),   // Front right
        Vec3::new(-1.0, 0.0, -1.5), // Rear left
        Vec3::new(1.0, 0.0, -1.5),  // Rear right
    ];

    for wheel_position in wheel_positions {
        app.world_mut().spawn((
            SuspensionRay {
                cast_distance: 1.0,
                ray_origin: wheel_position,
                ray_direction: Vec3::NEG_Y,
                ..default()
            },
            WheelState::default(),
            Transform::from_translation(position + wheel_position),
            GlobalTransform::from(Transform::from_translation(position + wheel_position)),
        ));
    }
}

/// Benchmark test for CI integration.
#[test]
#[ignore = "Performance test requiring full Bevy setup"]
fn benchmark_ci_gate() {
    let mut app = create_test_app();

    // Use standard benchmark configuration
    let benchmark_config = create_standard_benchmark();
    app.insert_resource(benchmark_config);

    // Spawn benchmark vehicles
    for i in 0..10 {
        spawn_test_vehicle(&mut app, Vec3::new(i as f32 * 4.0, 1.0, 0.0));
    }

    // Run benchmark simulation
    for _ in 0..60 {
        app.update();
    }

    // Verify benchmark results
    let benchmark_results = app.world().resource::<BenchmarkResults>();

    // CI gate: must pass performance target
    assert!(
        benchmark_results.passed,
        "CI benchmark gate failed: {:.2}ms > 16ms target",
        benchmark_results.average_cpu_time
    );

    // Additional checks
    assert_eq!(benchmark_results.frames_processed, 60);
    assert_eq!(benchmark_results.total_entities, 40); // 10 vehicles * 4 wheels
    assert!(benchmark_results.average_cpu_time > 0.0);
    assert!(benchmark_results.min_cpu_time > 0.0);
    assert!(benchmark_results.max_cpu_time > 0.0);
}
