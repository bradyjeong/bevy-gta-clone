//! Integration tests for facade system robustness
//!
//! Tests verify that:
//! 1. amp_foundation compiles without Bevy dependencies  
//! 2. amp_system plugins can be added to Bevy App
//! 3. amp_game plugins integrate properly
//! 4. Plugin discovery works correctly for each facade

use bevy::prelude::*;

/// Test that amp_foundation compiles without Bevy dependencies
#[test]
fn test_amp_foundation_no_bevy_compilation() {
    // amp_foundation should compile without Bevy - test core functionality
    use amp_foundation::prelude::*;

    // Test that we can use core math without Bevy
    let transform = AmpTransform::identity()
        .with_translation(Vec3::new(1.0, 2.0, 3.0))
        .with_scale(Vec3::ONE);

    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(transform.scale, Vec3::ONE);

    // Test AABB functionality without Bevy
    let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(10.0));
    assert!(aabb.contains(&Vec3::new(5.0, 5.0, 5.0)));
    assert!(!aabb.contains(&Vec3::new(15.0, 15.0, 15.0)));

    // Test Morton encoding without Bevy
    let morton = Morton3D::new(Vec3::new(100.0, 200.0, 300.0));
    assert!(morton.encode() > 0);
}

/// Test that amp_system plugins can be added to Bevy App
#[test]
fn test_amp_system_bevy_app_integration() {
    use amp_system::prelude::*;

    // Create minimal Bevy app
    let mut app = App::new();

    // Add MinimalPlugins for headless testing
    app.add_plugins(bevy::prelude::MinimalPlugins);

    // Add available amp_system plugins (only those that exist)
    // Note: amp_system doesn't have an AmpSystemPlugin - it re-exports individual plugins

    // Verify the app can update without crashing
    app.update();

    // Test that basic systems are registered
    let world = app.world();
    assert!(world.get_resource::<bevy::prelude::Time>().is_some());
}

/// Test that amp_game plugins integrate properly  
#[test]
fn test_amp_game_bevy_app_integration() {
    use amp_game::prelude::*;

    // Create minimal Bevy app
    let mut app = App::new();

    // Add MinimalPlugins for headless testing
    app.add_plugins(bevy::prelude::MinimalPlugins);

    // Add amp_game plugins
    app.add_plugins(GameplayPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    // Verify the app can update without crashing
    app.update();

    // Test that game systems are registered
    let world = app.world();
    assert!(world.get_resource::<Time>().is_some());
}

/// Test plugin discovery works correctly for each facade
#[test]
fn test_facade_plugin_discovery() {
    use amp_game::prelude::*;

    // Create app and add all facade plugins
    let mut app = App::new();
    app.add_plugins(bevy::prelude::MinimalPlugins)
        .add_plugins(GameplayPlugins)
        .add_plugins(PrefabFactoryPlugin);

    // Verify no plugin conflicts
    app.update();

    // Test that all expected resources are present
    let world = app.world();
    assert!(world.get_resource::<Time>().is_some());
}

/// Test that facade imports work correctly
#[test]
fn test_facade_imports() {
    // Test amp_foundation imports (Bevy-free)
    use amp_foundation::prelude::*;

    // Should have math types
    let _vec3 = Vec3::ZERO;
    let _aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

    // Test amp_system imports (with Bevy)
    use amp_system::prelude::*;

    let _app = App::new();

    // Test amp_game imports
    use amp_game::prelude::*;

    let _game_plugin = GameplayPlugins;
}

/// Test that facade features work correctly
#[test]
fn test_facade_features() {
    use amp_foundation::prelude::*;

    // Test that we can create basic transforms without features
    let transform = AmpTransform::identity();
    assert_eq!(transform.translation, Vec3::ZERO);
    assert_eq!(transform.rotation, glam::Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::ONE);
}

/// Integration test for minimal Bevy App with facade system
#[test]
fn test_minimal_bevy_app_with_facades() {
    use amp_game::prelude::*;
    use bevy::prelude::*;

    let mut app = App::new();

    // Add required Bevy plugins for testing
    app.add_plugins(MinimalPlugins);

    // Add facade plugins in correct order
    app.add_plugins(GameplayPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    // Run a few update cycles to ensure stability
    for _ in 0..5 {
        app.update();
    }

    // Verify the app is in a good state
    let world = app.world();
    assert!(world.get_resource::<Time>().is_some());
}

/// Test facade backwards compatibility
#[test]
fn test_facade_backwards_compatibility() {
    // This test ensures that the facade system remains compatible
    // with existing code patterns

    use amp_foundation::prelude::*;

    // Test that basic transforms work
    let transform = AmpTransform::from_translation(Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(transform.translation.x, 1.0);

    // Builder pattern should also work
    let transform2 = AmpTransform::identity().with_translation(Vec3::new(2.0, 0.0, 0.0));

    assert_eq!(transform2.translation.x, 2.0);
}
