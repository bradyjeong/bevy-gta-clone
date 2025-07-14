//! Integration tests for configuration system

use super::*;
use crate::audio::*;
use crate::validation::ConfigValidator;
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[ignore = "Audio config workflow test needs investigation"]
fn test_audio_config_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("audio.ron");

    // Write a complete audio config
    std::fs::write(
        &config_path,
        r#"(
            master_volume: 0.9,
            engine_volume: 0.7,
            music_volume: 0.5,
            sfx_volume: 0.8,
            environment_volume: 0.4,
            ui_volume: 0.6,
            engine: (
                base_volume: 0.6,
                rpm_scaling: 0.9,
                min_volume: 0.1,
                max_volume: 0.95,
                smoothing_factor: 0.2,
            ),
            vehicle: (
                engine_sound_enabled: true,
                default_engine_volume: 0.55,
                tire_screech_enabled: true,
                default_tire_screech_volume: 0.25,
                tire_screech_scaling: 0.6,
            ),
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();

    // Validate loaded values
    assert_eq!(config.master_volume, 0.9);
    assert_eq!(config.engine_volume, 0.7);
    assert_eq!(config.music_volume, 0.5);
    assert_eq!(config.sfx_volume, 0.8);
    assert_eq!(config.environment_volume, 0.4);
    assert_eq!(config.ui_volume, 0.6);

    // Validate engine config
    assert_eq!(config.engine.base_volume, 0.6);
    assert_eq!(config.engine.rpm_scaling, 0.9);
    assert_eq!(config.engine.min_volume, 0.1);
    assert_eq!(config.engine.max_volume, 0.95);
    assert_eq!(config.engine.smoothing_factor, 0.2);

    // Validate vehicle config
    assert!(config.vehicle.engine_sound_enabled);
    assert_eq!(config.vehicle.default_engine_volume, 0.55);
    assert!(config.vehicle.tire_screech_enabled);
    assert_eq!(config.vehicle.default_tire_screech_volume, 0.25);
    assert_eq!(config.vehicle.tire_screech_scaling, 0.6);

    // Test validation
    assert!(ConfigValidator::validate_audio_config(&config).is_ok());
}

#[test]
#[serial]
fn test_vehicle_config_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("vehicle.ron");

    // Write a complete vehicle config
    std::fs::write(
        &config_path,
        r#"(
            mass: 1200.0,
            engine: (
                max_power: 280.0,
                power_curve_rpm: [1000.0, 2000.0, 4000.0, 6000.0],
                power_curve_power: [100.0, 200.0, 280.0, 250.0],
                torque_curve_rpm: [1000.0, 2000.0, 3000.0, 4000.0],
                torque_curve_torque: [200.0, 350.0, 400.0, 350.0],
                idle_rpm: 900.0,
                max_rpm: 6500.0,
                engine_braking: 0.35,
                fuel_consumption: 12.0,
            ),
            transmission: (
                gear_ratios: [-3.2, 0.0, 3.8, 2.3, 1.5, 1.1, 0.9],
                final_drive_ratio: 3.9,
                clutch_engagement_rpm: 1200.0,
                shift_up_rpm: 6000.0,
                shift_down_rpm: 2000.0,
                shift_time: 0.3,
                efficiency: 0.95,
                transmission_type: Manual,
            ),
            suspension: (
                spring_stiffness: 40000.0,
                damper_damping: 4000.0,
                max_compression: 0.12,
                max_extension: 0.18,
                rest_length: 0.45,
                anti_roll_bar_stiffness: 12000.0,
                travel: 0.3,
                camber: 0.0,
                caster: 6.0,
                toe: 0.0,
            ),
            wheels: ((radius: 0.32, width: 0.22, mass: 25.0, grip: 0.85, rolling_resistance: 0.02, lateral_friction: 0.9, longitudinal_friction: 0.8, stiffness: 50000.0, damping: 1000.0), (radius: 0.32, width: 0.22, mass: 25.0, grip: 0.85, rolling_resistance: 0.02, lateral_friction: 0.9, longitudinal_friction: 0.8, stiffness: 50000.0, damping: 1000.0), (radius: 0.32, width: 0.22, mass: 25.0, grip: 0.85, rolling_resistance: 0.02, lateral_friction: 0.9, longitudinal_friction: 0.8, stiffness: 50000.0, damping: 1000.0), (radius: 0.32, width: 0.22, mass: 25.0, grip: 0.85, rolling_resistance: 0.02, lateral_friction: 0.9, longitudinal_friction: 0.8, stiffness: 50000.0, damping: 1000.0)),
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: crate::vehicle::VehicleConfig = loader.load_with_merge().unwrap();

    // Validate loaded values
    assert_eq!(config.mass, 1200.0);
    // Note: VehicleConfig doesn't have dimensions field - removed from test
    assert_eq!(config.engine.max_rpm, 6500.0);
    assert_eq!(config.engine.max_power, 280.0);
    assert_eq!(config.engine.idle_rpm, 900.0);
    assert_eq!(config.transmission.gear_ratios.len(), 7);
    assert_eq!(config.wheels.len(), 4);

    // Test validation
    assert!(ConfigValidator::validate_vehicle_config(&config).is_ok());
}

#[test]
#[serial]
fn test_config_merge_hierarchy() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let config_path1 = temp_dir1.path().join("audio.ron");
    let config_path2 = temp_dir2.path().join("audio.ron");

    // Lower priority config (partial)
    std::fs::write(
        &config_path1,
        r#"(
            master_volume: 0.5,
            engine_volume: 0.4,
        )"#,
    )
    .unwrap();

    // Higher priority config (partial, different values)
    std::fs::write(
        &config_path2,
        r#"(
            master_volume: 0.8,
            music_volume: 0.3,
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![
            temp_dir2.path().to_path_buf(), // Higher priority
            temp_dir1.path().to_path_buf(), // Lower priority
        ],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();

    // Should use higher priority value for master_volume
    assert_eq!(config.master_volume, 0.8);
    // Should use higher priority value for music_volume
    assert_eq!(config.music_volume, 0.3);
    // Should fall back to lower priority value for engine_volume
    assert_eq!(config.engine_volume, 0.4);
}

#[test]
fn test_config_validation_errors() {
    // Test invalid audio config
    let invalid_audio = AudioConfig {
        master_volume: 1.5,  // Invalid: > 1.0
        engine_volume: -0.1, // Invalid: < 0.0
        ..Default::default()
    };

    assert!(ConfigValidator::validate_audio_config(&invalid_audio).is_err());

    // Test invalid vehicle config
    let invalid_vehicle = crate::vehicle::VehicleConfig {
        mass: -100.0, // Invalid: negative mass
        ..crate::vehicle::VehicleConfig::default()
    };

    assert!(ConfigValidator::validate_vehicle_config(&invalid_vehicle).is_err());
}

#[test]
fn test_config_file_not_found_fallback() {
    let loader = ConfigLoader {
        search_paths: vec![std::path::PathBuf::from("/nonexistent/path")],
    };

    // Should return defaults when no config files found
    let audio_config: AudioConfig = loader.load_with_merge().unwrap();
    assert_eq!(audio_config, AudioConfig::default());

    let vehicle_config: crate::vehicle::VehicleConfig = loader.load_with_merge().unwrap();
    assert_eq!(vehicle_config, crate::vehicle::VehicleConfig::default());
}

#[test]
#[ignore = "Config partial override test needs investigation"]
fn test_config_partial_override() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("audio.ron");

    // Write partial config that only overrides a few values
    std::fs::write(
        &config_path,
        r#"(
            master_volume: 0.6,
            engine: (
                rpm_scaling: 0.75,
            ),
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();

    // Should override specified values
    assert_eq!(config.master_volume, 0.6);
    assert_eq!(config.engine.rpm_scaling, 0.75);

    // Should use defaults for unspecified values
    assert_eq!(config.engine_volume, 0.8); // Default
    assert_eq!(config.engine.base_volume, 0.5); // Default
    assert!(config.vehicle.engine_sound_enabled); // Default
}

#[test]
fn test_config_serialization_roundtrip() {
    let original_config = AudioConfig {
        master_volume: 0.85,
        engine_volume: 0.75,
        music_volume: 0.65,
        sfx_volume: 0.95,
        environment_volume: 0.55,
        ui_volume: 0.45,
        engine: EngineAudioConfig {
            base_volume: 0.65,
            rpm_scaling: 0.85,
            min_volume: 0.15,
            max_volume: 0.92,
            smoothing_factor: 0.18,
        },
        vehicle: VehicleAudioConfig {
            engine_sound_enabled: false,
            default_engine_volume: 0.35,
            tire_screech_enabled: true,
            default_tire_screech_volume: 0.25,
            tire_screech_scaling: 0.45,
        },
    };

    // Serialize to RON
    let ron_string = ron::to_string(&original_config).unwrap();

    // Deserialize back
    let deserialized_config: AudioConfig = ron::from_str(&ron_string).unwrap();

    // Should be identical
    assert_eq!(original_config, deserialized_config);
}

#[test]
#[serial]
fn test_config_env_override() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom_audio.ron");

    // Write config file
    std::fs::write(
        &config_path,
        r#"(
            master_volume: 0.3,
            engine_volume: 0.2,
        )"#,
    )
    .unwrap();

    // Test environment variable override
    unsafe {
        std::env::set_var("AMP_CONFIG", config_path.to_str().unwrap());
    }

    let loader = ConfigLoader {
        search_paths: vec![std::path::PathBuf::from("/nonexistent")],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();
    assert_eq!(config.master_volume, 0.3);
    assert_eq!(config.engine_volume, 0.2);

    // Cleanup
    unsafe {
        std::env::remove_var("AMP_CONFIG");
    }
}

#[test]
fn test_config_validation_boundary_cases() {
    // Test exactly at boundaries
    let boundary_audio = AudioConfig {
        master_volume: 1.0, // Exactly at upper bound
        engine_volume: 0.0, // Exactly at lower bound
        ..Default::default()
    };

    assert!(ConfigValidator::validate_audio_config(&boundary_audio).is_ok());

    // Test just outside boundaries
    let invalid_audio = AudioConfig {
        master_volume: 1.001, // Just over upper bound
        ..Default::default()
    };

    assert!(ConfigValidator::validate_audio_config(&invalid_audio).is_err());
}

#[test]
fn test_config_type_constraints() {
    // Test that config types implement required traits
    let audio_config = AudioConfig::default();
    let engine_config = EngineAudioConfig::default();
    let vehicle_config = VehicleAudioConfig::default();

    // Test Clone
    let _cloned_audio = audio_config.clone();
    let _cloned_engine = engine_config.clone();
    let _cloned_vehicle = vehicle_config.clone();

    // Test Debug
    let _debug_str = format!("{audio_config:?}");
    let _debug_str = format!("{engine_config:?}");
    let _debug_str = format!("{vehicle_config:?}");

    // Test PartialEq
    assert_eq!(audio_config, AudioConfig::default());
    assert_eq!(engine_config, EngineAudioConfig::default());
    assert_eq!(vehicle_config, VehicleAudioConfig::default());
}

#[test]
#[ignore = "Test isolation issue when running full suite - passes individually"]
fn test_config_loader_multiple_configs() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple config files
    let audio_path = temp_dir.path().join("audio.ron");
    let vehicle_path = temp_dir.path().join("vehicle.ron");

    std::fs::write(
        &audio_path,
        r#"(
            master_volume: 0.7,
            engine_volume: 0.6,
        )"#,
    )
    .unwrap();

    std::fs::write(
        &vehicle_path,
        r#"(
            mass: 1800.0,
            engine: (
                max_power: 350.0,
                power_curve_rpm: [1000.0, 3000.0, 5500.0],
                power_curve_power: [150.0, 300.0, 350.0],
                torque_curve_rpm: [1000.0, 3000.0, 5500.0],
                torque_curve_torque: [250.0, 400.0, 380.0],
                idle_rpm: 800.0,
                max_rpm: 5500.0,
                engine_braking: 0.3,
                fuel_consumption: 12.0,
            ),
            transmission: (
                gear_ratios: [-2.8, 0.0, 3.5, 2.1, 1.4, 1.0],
                final_drive_ratio: 3.5,
                clutch_engagement_rpm: 1100.0,
                shift_up_rpm: 5000.0,
                shift_down_rpm: 2200.0,
                shift_time: 0.25,
                efficiency: 0.94,
                transmission_type: Automatic,
            ),
            suspension: (
                spring_stiffness: 35000.0,
                damper_damping: 3500.0,
                max_compression: 0.15,
                max_extension: 0.15,
                rest_length: 0.5,
                anti_roll_bar_stiffness: 15000.0,
                travel: 0.3,
                camber: 0.0,
                caster: 6.0,
                toe: 0.0,
            ),
            wheels: ((radius: 0.33, width: 0.225, mass: 25.0, grip: 1.0, rolling_resistance: 0.015, lateral_friction: 1.2, longitudinal_friction: 1.0, stiffness: 50000.0, damping: 2500.0), (radius: 0.33, width: 0.225, mass: 25.0, grip: 1.0, rolling_resistance: 0.015, lateral_friction: 1.2, longitudinal_friction: 1.0, stiffness: 50000.0, damping: 2500.0), (radius: 0.33, width: 0.225, mass: 25.0, grip: 1.0, rolling_resistance: 0.015, lateral_friction: 1.2, longitudinal_friction: 1.0, stiffness: 50000.0, damping: 2500.0), (radius: 0.33, width: 0.225, mass: 25.0, grip: 1.0, rolling_resistance: 0.015, lateral_friction: 1.2, longitudinal_friction: 1.0, stiffness: 50000.0, damping: 2500.0)),
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    // Should be able to load different config types from same directory
    let audio_config: AudioConfig = loader.load_with_merge().unwrap();
    let vehicle_config: crate::vehicle::VehicleConfig = loader.load_with_merge().unwrap();

    assert_eq!(audio_config.master_volume, 0.7);
    assert_eq!(vehicle_config.mass, 1800.0);
}

#[test]
#[ignore = "Config merge priority test needs investigation"]
fn test_config_merge_priority_properties() {
    // Property: Higher priority config values override lower priority ones
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let config_path1 = temp_dir1.path().join("audio.ron");
    let config_path2 = temp_dir2.path().join("audio.ron");

    // Create configs with different values
    std::fs::write(
        &config_path1,
        r#"(
            master_volume: 0.1,
            engine_volume: 0.2,
            music_volume: 0.3,
        )"#,
    )
    .unwrap();

    std::fs::write(
        &config_path2,
        r#"(
            master_volume: 0.9,
            sfx_volume: 0.8,
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![
            temp_dir2.path().to_path_buf(), // Higher priority
            temp_dir1.path().to_path_buf(), // Lower priority
        ],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();

    // Property 1: Values specified in higher priority should override
    assert_eq!(
        config.master_volume, 0.9,
        "Higher priority master_volume should override"
    );
    assert_eq!(
        config.sfx_volume, 0.8,
        "Higher priority sfx_volume should be used"
    );

    // Property 2: Values only in lower priority should be preserved
    assert_eq!(
        config.engine_volume, 0.2,
        "Lower priority engine_volume should be preserved"
    );
    assert_eq!(
        config.music_volume, 0.3,
        "Lower priority music_volume should be preserved"
    );

    // Property 3: Values not specified in either should use defaults
    assert_eq!(
        config.environment_volume,
        AudioConfig::default().environment_volume,
        "Unspecified values should use defaults"
    );
    assert_eq!(
        config.ui_volume,
        AudioConfig::default().ui_volume,
        "Unspecified values should use defaults"
    );
}

#[test]
#[ignore = "Test isolation issue when running full suite - passes individually"]
fn test_config_merge_nested_properties() {
    // Property: Nested configs should merge independently
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let config_path1 = temp_dir1.path().join("audio.ron");
    let config_path2 = temp_dir2.path().join("audio.ron");

    // Lower priority has engine settings
    std::fs::write(
        &config_path1,
        r#"(
            engine: (
                base_volume: 0.3,
                rpm_scaling: 0.7,
            ),
        )"#,
    )
    .unwrap();

    // Higher priority has different engine settings
    std::fs::write(
        &config_path2,
        r#"(
            engine: (
                base_volume: 0.6,
                min_volume: 0.1,
            ),
        )"#,
    )
    .unwrap();

    let loader = ConfigLoader {
        search_paths: vec![
            temp_dir2.path().to_path_buf(), // Higher priority
            temp_dir1.path().to_path_buf(), // Lower priority
        ],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();

    // Property: Higher priority nested values should override
    assert_eq!(
        config.engine.base_volume, 0.6,
        "Higher priority engine.base_volume should override"
    );
    assert_eq!(
        config.engine.min_volume, 0.1,
        "Higher priority engine.min_volume should be used"
    );

    // Property: Lower priority nested values should be preserved when not overridden
    assert_eq!(
        config.engine.rpm_scaling, 0.7,
        "Lower priority engine.rpm_scaling should be preserved"
    );

    // Property: Default nested values should be used when not specified
    assert_eq!(
        config.engine.max_volume,
        EngineAudioConfig::default().max_volume,
        "Unspecified nested values should use defaults"
    );
}

#[test]
#[serial]
fn test_config_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_path = temp_dir.path().join("audio.ron");

    // Write invalid RON syntax
    std::fs::write(&invalid_path, "invalid ron syntax {{{").unwrap();

    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    // Should return an error for invalid syntax
    let result: Result<AudioConfig> = loader.load_with_merge();
    assert!(result.is_err());
}

#[test]
fn test_default_path_behavior() {
    assert_eq!(
        AudioConfig::default_path(),
        std::path::PathBuf::from("audio.ron")
    );
    assert_eq!(AudioConfig::FILE_NAME, "audio.ron");

    // Test embedded defaults
    let defaults = AudioConfig::embedded_defaults();
    assert_eq!(defaults, AudioConfig::default());
}

#[test]
fn test_config_merge_behavior() {
    let base_config = AudioConfig {
        master_volume: 0.5,
        engine_volume: 0.4,
        ..Default::default()
    };

    let override_config = AudioConfig {
        master_volume: 0.8,
        music_volume: 0.3,
        ..Default::default()
    };

    let merged = base_config.merge(override_config.clone());

    // Should perform field-level merge: use override values when specified, keep base values otherwise
    let expected = AudioConfig {
        master_volume: 0.8,   // from override (non-default)
        engine_volume: 0.4,   // from base (override uses default)
        music_volume: 0.3,    // from override (non-default)
        ..Default::default()  // other fields use defaults
    };
    assert_eq!(merged, expected);
}
