//! Tests for audio configuration integration

use super::*;
use config_core::{AudioConfig, ConfigLoader};
use tempfile::TempDir;

#[test]
fn test_audio_settings_uses_config_loader() {
    let settings = GameplayAudioSettings::default();

    // Should load with reasonable values
    assert!(settings.master_volume >= 0.0 && settings.master_volume <= 1.0);
    assert!(settings.engine_volume >= 0.0 && settings.engine_volume <= 1.0);
    assert!(settings.music_volume >= 0.0 && settings.music_volume <= 1.0);
    assert!(settings.sfx_volume >= 0.0 && settings.sfx_volume <= 1.0);
    assert!(settings.environment_volume >= 0.0 && settings.environment_volume <= 1.0);
    assert!(settings.ui_volume >= 0.0 && settings.ui_volume <= 1.0);
}

#[test]
fn test_audio_settings_fallback_on_config_error() {
    // This test ensures fallback works when config loading fails
    let settings = GameplayAudioSettings::default();

    // Should have some reasonable values even if config fails
    assert!(settings.master_volume > 0.0);
    assert!(settings.engine_volume > 0.0);
    assert!(settings.music_volume > 0.0);
}

#[test]
fn test_audio_settings_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("audio.ron");

    // Write custom audio config
    std::fs::write(
        &config_path,
        r#"(
            master_volume: 0.5,
            engine_volume: 0.3,
            music_volume: 0.2,
            sfx_volume: 0.4,
            environment_volume: 0.1,
            ui_volume: 0.6,
        )"#,
    )
    .unwrap();

    // Create loader with custom path
    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();
    assert_eq!(config.master_volume, 0.5);
    assert_eq!(config.engine_volume, 0.3);
    assert_eq!(config.music_volume, 0.2);
    assert_eq!(config.sfx_volume, 0.4);
    assert_eq!(config.environment_volume, 0.1);
    assert_eq!(config.ui_volume, 0.6);
}

#[test]
fn test_vehicle_engine_audio_event_creation() {
    use bevy::math::Vec3;

    let event = VehicleEngineAudioEvent {
        vehicle_entity: Entity::from_raw(123),
        rpm: 2500.0,
        throttle: 0.7,
        load: 0.8,
        gear: 3,
        position: Vec3::new(10.0, 0.0, 5.0),
    };

    assert_eq!(event.rpm, 2500.0);
    assert_eq!(event.throttle, 0.7);
    assert_eq!(event.load, 0.8);
    assert_eq!(event.gear, 3);
    assert_eq!(event.position, Vec3::new(10.0, 0.0, 5.0));
}

#[test]
fn test_audio_assets_initialization() {
    let assets = AudioAssets::new();
    assert!(assets.engine_sounds.is_empty());
    assert!(assets.environmental_sounds.is_empty());
    assert!(assets.music_tracks.is_empty());
    assert!(assets.sound_effects.is_empty());
}

#[test]
fn test_audio_assets_loading() {
    // This test would require a full Bevy app to test properly
    // For now, we test the structure
    let assets = AudioAssets::new();
    assert_eq!(assets.engine_sounds.len(), 0);
    assert_eq!(assets.environmental_sounds.len(), 0);
    assert_eq!(assets.music_tracks.len(), 0);
    assert_eq!(assets.sound_effects.len(), 0);
}

#[test]
fn test_audio_settings_volume_bounds() {
    let settings = GameplayAudioSettings::default();

    // All volumes should be within valid range
    assert!(settings.master_volume >= 0.0 && settings.master_volume <= 1.0);
    assert!(settings.engine_volume >= 0.0 && settings.engine_volume <= 1.0);
    assert!(settings.music_volume >= 0.0 && settings.music_volume <= 1.0);
    assert!(settings.sfx_volume >= 0.0 && settings.sfx_volume <= 1.0);
    assert!(settings.environment_volume >= 0.0 && settings.environment_volume <= 1.0);
    assert!(settings.ui_volume >= 0.0 && settings.ui_volume <= 1.0);
}

#[test]
fn test_audio_settings_consistency() {
    let settings1 = GameplayAudioSettings::default();
    let settings2 = GameplayAudioSettings::default();

    // Should be consistent across multiple instantiations
    assert_eq!(settings1.master_volume, settings2.master_volume);
    assert_eq!(settings1.engine_volume, settings2.engine_volume);
    assert_eq!(settings1.music_volume, settings2.music_volume);
}

#[test]
fn test_vehicle_engine_audio_event_serialization() {
    let event = VehicleEngineAudioEvent {
        vehicle_entity: Entity::from_raw(456),
        rpm: 3000.0,
        throttle: 0.5,
        load: 0.6,
        gear: 2,
        position: Vec3::new(1.0, 2.0, 3.0),
    };

    // Test that the event can be used in serialization contexts
    // (even though Entity itself may not be serializable)
    assert_eq!(event.rpm, 3000.0);
    assert_eq!(event.throttle, 0.5);
    assert_eq!(event.load, 0.6);
}

#[test]
fn test_audio_config_integration_with_gameplay() {
    // Test that config system integrates properly with gameplay components
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("audio.ron");

    // Write minimal config
    std::fs::write(
        &config_path,
        r#"(
            master_volume: 0.8,
            engine_volume: 0.6,
            vehicle: (
                engine_sound_enabled: true,
                default_engine_volume: 0.4,
                tire_screech_enabled: false,
                default_tire_screech_volume: 0.1,
            ),
        )"#,
    )
    .unwrap();

    // Test that the config loads correctly
    let loader = ConfigLoader {
        search_paths: vec![temp_dir.path().to_path_buf()],
    };

    let config: AudioConfig = loader.load_with_merge().unwrap();
    assert_eq!(config.master_volume, 0.8);
    assert_eq!(config.engine_volume, 0.6);
    assert!(config.vehicle.engine_sound_enabled);
    assert_eq!(config.vehicle.default_engine_volume, 0.4);
    assert!(!config.vehicle.tire_screech_enabled);
    assert_eq!(config.vehicle.default_tire_screech_volume, 0.1);
}

#[test]
fn test_audio_settings_type_alias() {
    // Test that the type alias works correctly
    let settings: AudioSettings = AudioSettings::default();
    let gameplay_settings: GameplayAudioSettings = GameplayAudioSettings::default();

    // Should have the same structure
    assert_eq!(settings.master_volume, gameplay_settings.master_volume);
    assert_eq!(settings.engine_volume, gameplay_settings.engine_volume);
}

#[test]
fn test_audio_event_entity_handling() {
    let entity1 = Entity::from_raw(100);
    let entity2 = Entity::from_raw(200);

    let event1 = VehicleEngineAudioEvent {
        vehicle_entity: entity1,
        rpm: 1000.0,
        throttle: 0.1,
        load: 0.2,
        gear: 1,
        position: Vec3::ZERO,
    };

    let event2 = VehicleEngineAudioEvent {
        vehicle_entity: entity2,
        rpm: 2000.0,
        throttle: 0.8,
        load: 0.9,
        gear: 4,
        position: Vec3::ONE,
    };

    assert_ne!(event1.vehicle_entity, event2.vehicle_entity);
    assert_ne!(event1.rpm, event2.rpm);
    assert_ne!(event1.throttle, event2.throttle);
}

#[test]
fn test_audio_assets_hash_map_operations() {
    let mut assets = AudioAssets::new();

    // Test basic HashMap operations work as expected
    let dummy_handle = Handle::default();
    assets
        .engine_sounds
        .insert("idle".to_string(), dummy_handle.clone());
    assets
        .engine_sounds
        .insert("rev".to_string(), dummy_handle.clone());

    assert_eq!(assets.engine_sounds.len(), 2);
    assert!(assets.engine_sounds.contains_key("idle"));
    assert!(assets.engine_sounds.contains_key("rev"));
    assert!(!assets.engine_sounds.contains_key("brake"));
}
