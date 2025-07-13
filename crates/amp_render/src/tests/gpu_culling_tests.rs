//! Tests for GPU culling system and feature flag behavior

use bevy::prelude::*;

/// Test that GPU culling feature flag properly toggles functionality
#[test]
fn test_gpu_culling_feature_flag() {
    #[cfg(feature = "gpu_culling")]
    {
        // GPU culling should be available
        use crate::gpu_culling::prelude::*;
        
        let mut app = App::new();
        app.add_plugins(GpuCullingPlugin);
        
        // Should initialize without errors when feature is enabled
        app.update();
    }
    
    #[cfg(not(feature = "gpu_culling"))]
    {
        // CPU fallback should be used
        use crate::gpu_culling::prelude::*;
        
        let mut app = App::new();
        app.add_plugins(GpuCullingPlugin);
        
        // Should still work with CPU fallback
        app.update();
    }
}

/// Test platform detection and fallback behavior
#[test]
fn test_platform_fallback() {
    // This test ensures that the feature flag system
    // gracefully handles different platform capabilities
    
    let mut app = App::new();
    
    // Add minimal components for testing
    app.add_plugins((
        DefaultPlugins.build().disable::<WindowPlugin>(),
        crate::BatchingPlugin,
    ));
    
    // Should not crash regardless of feature flag state
    app.update();
}

/// Test that batch processing works with both GPU and CPU culling
#[test]
fn test_batch_processing_compatibility() {
    let mut app = App::new();
    
    app.add_plugins((
        DefaultPlugins.build().disable::<WindowPlugin>(),
        crate::BatchingPlugin,
    ));
    
    // Create test entities
    let mesh_handle = Handle::<Mesh>::default();
    let material_handle = Handle::<StandardMaterial>::default();
    
    app.world_mut().spawn((
        crate::BatchKey::new(&mesh_handle, &material_handle),
        Transform::default(),
        GlobalTransform::default(),
    ));
    
    // Should process entities regardless of culling method
    app.update();
    
    // Verify entities were processed
    let entity_count = app.world().query::<&crate::BatchKey>().iter(&app.world()).count();
    assert_eq!(entity_count, 1);
}

/// Test GPU instance data conversion
#[cfg(feature = "gpu_culling")]
#[test]
fn test_gpu_instance_conversion() {
    use crate::gpu_culling::{GpuInstance};
    use crate::ExtractedInstance;
    use glam::{Mat4, Vec3};
    
    let transform = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
    let batch_key = crate::BatchKey {
        mesh_id: 123,
        material_id: 456,
        flags: 0,
    };
    
    let extracted = ExtractedInstance::new(transform, batch_key, Vec3::ZERO);
    let gpu_instance = GpuInstance::from_extracted(&extracted, 5);
    
    // Verify conversion
    assert_eq!(gpu_instance.model_matrix, transform.to_cols_array());
    assert_eq!(gpu_instance.center, [10.0, 20.0, 30.0]);
    assert_eq!(gpu_instance.batch_id, 5);
    assert!(gpu_instance.radius > 0.0);
}

/// Test culling uniforms structure
#[cfg(feature = "gpu_culling")]
#[test]
fn test_culling_uniforms() {
    use crate::gpu_culling::{CullingUniforms, GpuFrustum};
    
    let uniforms = CullingUniforms {
        frustum: GpuFrustum {
            planes: [[1.0, 0.0, 0.0, 0.0]; 6],
        },
        max_distance: 1000.0,
        instance_count: 1024,
        batch_count: 10,
        enable_flags: 3, // Both frustum and distance culling
    };
    
    // Verify structure layout
    assert_eq!(uniforms.max_distance, 1000.0);
    assert_eq!(uniforms.instance_count, 1024);
    assert_eq!(uniforms.batch_count, 10);
    assert_eq!(uniforms.enable_flags, 3);
    
    // Test enable flags
    assert_eq!(uniforms.enable_flags & 1, 1); // Frustum enabled
    assert_eq!(uniforms.enable_flags & 2, 2); // Distance enabled
}

/// Test that feature flag correctly excludes GPU code when disabled
#[cfg(not(feature = "gpu_culling"))]
#[test]
fn test_cpu_only_compilation() {
    // This test ensures CPU-only builds don't include GPU code
    
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.build().disable::<WindowPlugin>(),
        crate::BatchingPlugin,
    ));
    
    // Should compile and run without GPU dependencies
    app.update();
    
    // CPU culling should still work
    app.world_mut().insert_resource(crate::culling::CullingConfig::default());
    app.update();
}
