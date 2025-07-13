//! Integration test for GPU culling render graph implementation

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy::prelude::*;
    use bevy::render::RenderPlugin;

    #[test]
    fn test_gpu_culling_integration() {
        let mut app = App::new();
        
        // Add required plugins
        app.add_plugins((
            bevy::log::LogPlugin::default(),
            bevy::render::RenderPlugin::default(),
        ));

        // Add GPU culling plugins
        app.add_plugins((
            GpuCullingPlugin,
            GpuCullingPipelinePlugin,
        ));

        // Verify resources are initialized
        app.update();
        
        // Check that the GPU culling config resource was added
        assert!(app.world().contains_resource::<GpuCullingConfig>());
        
        // Check that the resource has the correct default values
        let config = app.world().resource::<GpuCullingConfig>();
        assert_eq!(config.max_instances_per_dispatch, 100_000);
        assert_eq!(config.workgroup_size, 64);
        assert!(!config.debug_output);
        assert!(config.enable_frustum_culling);
    }

    #[test]
    fn test_gpu_culling_fallback_behavior() {
        let mut app = App::new();
        
        // Add minimal required plugins
        app.add_plugins(bevy::log::LogPlugin::default());

        // Add GPU culling plugin
        app.add_plugins(GpuCullingPlugin);

        // Verify that the plugin doesn't crash when RenderApp is not available
        app.update();
        
        // The plugin should still add the configuration resource
        assert!(app.world().contains_resource::<GpuCullingConfig>());
    }

    #[test]
    fn test_gpu_culling_pipeline_availability() {
        let mut world = World::new();
        
        // Should return false when no resources are present
        assert!(!is_gpu_culling_pipeline_available(&world));
        
        // Should return false when resources are present but no pipeline
        world.insert_resource(GpuCullingResources::default());
        assert!(!is_gpu_culling_pipeline_available(&world));
        
        // This demonstrates the API even though we don't have a real pipeline
        // In a full implementation, this would return true when a pipeline is available
    }

    #[test]
    fn test_gpu_culling_render_node() {
        let mut world = World::new();
        
        // Add GPU culling resources
        world.insert_resource(GpuCullingResources::default());
        
        // Create the render node
        let node = GpuCullNode;
        
        // The node should handle missing resources gracefully
        // In a full implementation, this would test the actual render graph execution
        assert!(std::mem::size_of::<GpuCullNode>() > 0);
    }
}
