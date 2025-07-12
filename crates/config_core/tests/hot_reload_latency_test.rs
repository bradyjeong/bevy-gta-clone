use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[test]
fn test_config_file_write_latency() {
    // Setup temporary directory for test config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.ron");

    // Initial config content
    let initial_config = r#"(
        test_value: 42,
        string_value: "initial",
    )"#;

    fs::write(&config_path, initial_config).expect("Failed to write initial config");

    // Give file system time to settle
    thread::sleep(Duration::from_millis(10));

    // Record start time for latency measurement
    let start_time = Instant::now();

    // Modified config content
    let modified_config = r#"(
        test_value: 84,
        string_value: "modified",
    )"#;

    // Write the modified config to simulate hot-reload trigger
    fs::write(&config_path, modified_config).expect("Failed to write modified config");

    // Measure file system write latency
    let write_latency = start_time.elapsed();

    // Verify the config was actually written
    let content = fs::read_to_string(&config_path).expect("Failed to read modified config");
    assert!(content.contains("test_value: 84"));

    // Performance Gate: File write should be very fast (< 5ms typically)
    assert!(
        write_latency < Duration::from_millis(5),
        "File write latency {} ms is unexpectedly high",
        write_latency.as_millis()
    );

    println!(
        "✅ Config file write latency: {} ms",
        write_latency.as_millis()
    );
}

#[test]
fn test_config_reload_simulation() {
    // This test simulates the hot-reload workflow without async dependencies
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("performance_config.ron");

    // Create initial performance config
    let initial_config = r#"(
        target_fps: 60.0,
        max_entities: (
            buildings: 500,
            vehicles: 50,
            npcs: 30,
            vegetation: 1000,
        ),
        spawn_rates: (
            buildings: 0.08,
            vehicles: 0.04,
            trees: 0.05,
            npcs: 0.01,
        ),
        culling_distances: (
            buildings: 300.0,
            vehicles: 150.0,
            npcs: 100.0,
            vegetation: 200.0,
        ),
    )"#;

    fs::write(&config_path, initial_config).expect("Failed to write initial config");

    // Simulate hot-reload detection and processing
    let reload_start = Instant::now();

    // Step 1: Detect file change (simulated)
    let detection_time = Instant::now();
    thread::sleep(Duration::from_millis(1)); // Simulate detection overhead
    let detection_latency = detection_time.elapsed();

    // Step 2: Read and parse config
    let parse_start = Instant::now();
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    let _parsed: ron::Value = ron::from_str(&content).expect("Failed to parse config");
    let parse_latency = parse_start.elapsed();

    // Step 3: Simulate ECS resource update
    let ecs_start = Instant::now();
    thread::sleep(Duration::from_millis(1)); // Simulate ECS update
    let ecs_latency = ecs_start.elapsed();

    let total_latency = reload_start.elapsed();

    // Performance Gate: Total simulated hot-reload should be <16ms
    assert!(
        total_latency < Duration::from_millis(16),
        "Simulated hot-reload latency {} ms exceeds 16ms requirement. Detection: {} ms, Parse: {} ms, ECS: {} ms",
        total_latency.as_millis(),
        detection_latency.as_millis(),
        parse_latency.as_millis(),
        ecs_latency.as_millis()
    );

    println!(
        "✅ Simulated hot-reload latency: {} ms (requirement: <16ms)",
        total_latency.as_millis()
    );
    println!("   - Detection: {} ms", detection_latency.as_millis());
    println!("   - Parse: {} ms", parse_latency.as_millis());
    println!("   - ECS update: {} ms", ecs_latency.as_millis());
}

#[test]
fn test_config_parsing_performance() {
    // Test parsing performance for various config sizes
    let configs = vec![
        ("small", r#"(test: 42)"#),
        (
            "medium",
            include_str!("../../../assets/config/performance_config.ron"),
        ),
        (
            "large",
            include_str!("../../../assets/config/game_config.ron"),
        ),
    ];

    for (size, config_content) in configs {
        let parse_start = Instant::now();
        let _parsed: ron::Value = ron::from_str(config_content)
            .unwrap_or_else(|_| panic!("Failed to parse {size} config"));
        let parse_latency = parse_start.elapsed();

        // Performance gate: Even large configs should parse quickly
        assert!(
            parse_latency < Duration::from_millis(5),
            "{} config parse time {} ms exceeds 5ms",
            size,
            parse_latency.as_millis()
        );

        println!(
            "✅ {} config parse time: {} ms",
            size,
            parse_latency.as_millis()
        );
    }
}
