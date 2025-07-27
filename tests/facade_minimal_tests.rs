//! Minimal facade integration tests
//!
//! These tests verify the core facade functionality that can compile
//! without requiring the full game engine dependencies.

/// Test that amp_foundation compiles and works without Bevy dependencies
#[test]
fn test_amp_foundation_no_bevy() {
    use amp_foundation::prelude::*;

    // Test core math functionality
    let transform = AmpTransform::identity()
        .with_translation(Vec3::new(1.0, 2.0, 3.0))
        .with_scale(Vec3::ONE);

    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(transform.scale, Vec3::ONE);

    // Test AABB functionality
    let aabb = Aabb::new(Vec3::ZERO, Vec3::splat(10.0));
    assert!(aabb.contains_point(&Vec3::new(5.0, 5.0, 5.0)));
    assert!(!aabb.contains_point(&Vec3::new(15.0, 15.0, 15.0)));

    // Test Morton encoding
    let morton_code = Morton3D::encode(Vec3::new(100.0, 200.0, 300.0));
    assert!(morton_code > 0);
}

/// Test that amp_foundation math operations work correctly
#[test]
fn test_amp_foundation_math() {
    use amp_foundation::prelude::*;

    // Test transform composition
    let transform1 = AmpTransform::from_translation(Vec3::new(1.0, 0.0, 0.0));
    let transform2 = AmpTransform::from_scale(Vec3::splat(2.0));

    assert_eq!(transform1.translation.x, 1.0);
    assert_eq!(transform2.scale, Vec3::splat(2.0));

    // Test AABB operations
    let aabb1 = Aabb::new(Vec3::ZERO, Vec3::splat(5.0));
    let aabb2 = Aabb::new(Vec3::splat(2.0), Vec3::splat(7.0));

    // Should be able to check containment and intersection
    assert!(aabb1.contains_point(&Vec3::new(2.5, 2.5, 2.5)));
    assert!(aabb2.contains_point(&Vec3::new(5.0, 5.0, 5.0)));
}

/// Test that amp_foundation error handling works
#[test]
fn test_amp_foundation_error_handling() {
    use amp_foundation::prelude::*;

    // Test that Result type works
    let result: Result<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);

    let error_result: Result<i32> = Err(Error::IoError("test error".to_string()));
    assert!(error_result.is_err());
}

/// Test that facade imports work without conflicts
#[test]
fn test_facade_imports_compatibility() {
    // amp_foundation should work standalone
    {
        use amp_foundation::prelude::*;
        let _vec3 = Vec3::ZERO;
        let _aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let _transform = AmpTransform::identity();
    }

    // amp_system should provide Bevy integration
    {
        use amp_system::prelude::*;
        let _app = App::new();
        let _vec3 = Vec3::ZERO;
        let _transform = Transform::default();
    }
}

/// Test minimal Bevy app creation with amp_system
#[test]
fn test_minimal_bevy_app() {
    use amp_system::prelude::*;

    let mut app = App::new();
    app.add_plugins(bevy::prelude::MinimalPlugins);

    // Should be able to update without crashing
    app.update();

    // Verify basic resources exist
    let world = app.world();
    assert!(world.get_resource::<bevy::prelude::Time>().is_some());
}

/// Test that facade features work with different configurations
#[test]
fn test_facade_features() {
    // Test without features
    {
        use amp_foundation::prelude::*;
        let transform = AmpTransform::identity();
        assert_eq!(transform.translation, Vec3::ZERO);
    }

    // Test with basic features
    {
        use amp_system::prelude::*;
        let app = App::new();
        // Should compile and work
        drop(app);
    }
}
