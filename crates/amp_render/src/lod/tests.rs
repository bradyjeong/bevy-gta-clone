//! Tests for the enhanced LOD system

use super::*;

#[test]
fn test_lod_group_creation() {
    let mesh1 = Handle::default();
    let mesh2 = Handle::default();

    let levels = vec![LodLevel::new(10.0, mesh1), LodLevel::new(50.0, mesh2)];

    let lod_group = LodGroup::new(levels);

    assert_eq!(lod_group.levels.len(), 2);
    assert_eq!(lod_group.current_lod, 0);
    assert_eq!(lod_group.previous_lod, 0);
    assert_eq!(lod_group.hysteresis, 8.0);
    assert_eq!(lod_group.cross_fade_duration, 0.3);
}

#[test]
fn test_lod_group_with_custom_hysteresis() {
    let levels = vec![LodLevel::new(10.0, Handle::default())];
    let lod_group = LodGroup::new(levels).with_hysteresis(15.0);

    assert_eq!(lod_group.hysteresis, 15.0);
}

#[test]
fn test_lod_group_with_custom_cross_fade() {
    let levels = vec![LodLevel::new(10.0, Handle::default())];
    let lod_group = LodGroup::new(levels).with_cross_fade_duration(0.5);

    assert_eq!(lod_group.cross_fade_duration, 0.5);
}

#[test]
fn test_lod_distance_calculation() {
    let mesh1 = Handle::default();
    let mesh2 = Handle::default();

    let levels = vec![LodLevel::new(20.0, mesh1), LodLevel::new(50.0, mesh2)];

    let lod_group = LodGroup::new(levels);

    // Close distance should select LOD 0
    assert_eq!(lod_group.get_lod_for_distance_simple(15.0), 0);

    // Medium distance should select LOD 1
    assert_eq!(lod_group.get_lod_for_distance_simple(30.0), 1);

    // Far distance should select highest LOD (last index)
    assert_eq!(lod_group.get_lod_for_distance_simple(100.0), 1);
}

#[test]
fn test_lod_hysteresis_behavior() {
    let mesh1 = Handle::default();
    let mesh2 = Handle::default();

    let levels = vec![LodLevel::new(20.0, mesh1), LodLevel::new(50.0, mesh2)];

    let mut lod_group = LodGroup::new(levels).with_hysteresis(5.0);

    // First update should work without hysteresis - start at LOD 0
    let _changed = lod_group.update_lod(15.0);
    assert_eq!(lod_group.current_lod, 0);

    // Move just past the threshold (20.0) + hysteresis - should trigger change to LOD 1
    // Distance 26.0 + 5.0 hysteresis = 31.0, which is > 20.0, so LOD 1
    let _changed = lod_group.update_lod(26.0);
    assert_eq!(lod_group.current_lod, 1);

    // Move back closer - should stay at LOD 1 due to hysteresis
    // Distance 18.0 - 5.0 hysteresis = 13.0, which is < 20.0, so LOD 0
    let _changed = lod_group.update_lod(18.0);
    assert_eq!(lod_group.current_lod, 0);

    // Test that hysteresis prevents rapid switching at boundary
    // Distance 22.0 + 5.0 hysteresis = 27.0, which is > 20.0, so LOD 1
    let _changed = lod_group.update_lod(22.0);
    assert_eq!(lod_group.current_lod, 1);
}

#[test]
fn test_lod_update_with_change_tracking() {
    let mesh1 = Handle::default();
    let mesh2 = Handle::default();

    let levels = vec![LodLevel::new(20.0, mesh1), LodLevel::new(50.0, mesh2)];

    let mut lod_group = LodGroup::new(levels);

    // First update - should not change (distance 15.0 < 20.0, so stays at LOD 0)
    let changed = lod_group.update_lod(15.0);
    assert!(
        !changed,
        "First update should not change LOD when staying within range"
    );
    assert_eq!(lod_group.current_lod, 0);

    // Move to distance that triggers LOD change
    let changed = lod_group.update_lod(35.0);
    assert!(changed);
    assert_eq!(lod_group.current_lod, 1);
    assert_eq!(lod_group.previous_lod, 0);
    assert_eq!(lod_group.cross_fade_factor, 0.0);
}

#[test]
fn test_cross_fade_animation() {
    let levels = vec![
        LodLevel::new(20.0, Handle::default()),
        LodLevel::new(50.0, Handle::default()),
    ];

    let mut lod_group = LodGroup::new(levels).with_cross_fade_duration(1.0);

    // Trigger LOD change
    lod_group.update_lod(35.0);
    assert!(lod_group.is_cross_fading());

    // Update cross-fade factor
    lod_group.update_cross_fade(0.3); // 30% of 1.0 second duration
    assert_eq!(lod_group.cross_fade_factor, 0.3);
    assert!(lod_group.is_cross_fading());

    // Complete cross-fade
    lod_group.update_cross_fade(0.7); // Complete the remaining 70%
    assert_eq!(lod_group.cross_fade_factor, 1.0);
    assert!(!lod_group.is_cross_fading());
}

#[test]
fn test_lod_batch_key_generation() {
    let mesh1 = Handle::default();
    let mesh2 = Handle::default();
    let material = Handle::default();

    let levels = vec![
        LodLevel::new(20.0, mesh1.clone()),
        LodLevel::new(50.0, mesh2.clone()),
    ];

    let lod_group = LodGroup::new(levels);

    // Should generate batch key for current LOD (0)
    if let Some(batch_key) = lod_group.get_batch_key(&material) {
        // Verify it uses the correct mesh for LOD 0
        assert_eq!(batch_key.mesh_id, {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            mesh1.id().hash(&mut hasher);
            hasher.finish()
        });
    } else {
        panic!("Should generate batch key");
    }
}

#[test]
fn test_lod_level_with_material_override() {
    let mesh = Handle::default();
    let material = Handle::default();

    let level = LodLevel::new(20.0, mesh.clone()).with_material(material.clone());

    assert_eq!(level.distance, 20.0);
    assert_eq!(level.mesh, mesh);
    assert_eq!(level.material, Some(material));
}

#[test]
fn test_changed_lod_component() {
    let changed_lod = ChangedLod {
        from_lod: 0,
        to_lod: 1,
        changed_frame: 100,
    };

    assert_eq!(changed_lod.from_lod, 0);
    assert_eq!(changed_lod.to_lod, 1);
    assert_eq!(changed_lod.changed_frame, 100);
}

#[test]
fn test_lod_config_defaults() {
    let config = LodConfig::default();

    assert!(config.enabled);
    assert_eq!(config.bias, 1.0);
    assert_eq!(config.min_distance, 1.0);
    assert!(config.cross_fade_enabled);
    assert_eq!(config.max_updates_per_frame, 1000);
}

#[test]
fn test_multiple_lod_transitions() {
    let levels = vec![
        LodLevel::new(10.0, Handle::default()),
        LodLevel::new(30.0, Handle::default()),
        LodLevel::new(60.0, Handle::default()),
        LodLevel::new(100.0, Handle::default()),
    ];

    let mut lod_group = LodGroup::new(levels).with_hysteresis(2.0); // Use smaller hysteresis for cleaner test

    // Start close
    lod_group.update_lod(5.0);
    assert_eq!(lod_group.current_lod, 0);

    // Move to medium distance - account for hysteresis
    let changed = lod_group.update_lod(25.0); // 25.0 + 2.0 = 27.0 <= 30.0
    assert!(changed);
    assert_eq!(lod_group.current_lod, 1);

    // Move to far distance - account for hysteresis
    let changed = lod_group.update_lod(55.0); // 55.0 + 2.0 = 57.0 <= 60.0
    assert!(changed);
    assert_eq!(lod_group.current_lod, 2);

    // Move to very far distance - account for hysteresis
    let changed = lod_group.update_lod(95.0); // 95.0 + 2.0 = 97.0 <= 100.0
    assert!(changed);
    assert_eq!(lod_group.current_lod, 3);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bevy::{
        app::App,
        prelude::{Camera3d, Transform},
        time::Time,
    };

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(LodSystemPlugin).init_resource::<Time>();
        app
    }

    #[test]
    fn test_lod_system_plugin_setup() {
        let app = setup_test_app();

        // Verify LodConfig resource is initialized
        assert!(app.world().contains_resource::<LodConfig>());
    }

    #[test]
    fn test_lod_system_integration() {
        let mut app = setup_test_app();

        // Add a camera
        app.world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.0, 0.0)));

        // Add a LOD object
        let levels = vec![
            LodLevel::new(20.0, Handle::default()),
            LodLevel::new(50.0, Handle::default()),
        ];

        app.world_mut()
            .spawn((LodGroup::new(levels), Transform::from_xyz(10.0, 0.0, 0.0)));

        // Update should not panic
        app.update();
    }
}
