//! Tests for vegetation GPU resource cleanup system

#[cfg(test)]
mod tests {
    use super::super::components::*;
    use super::super::systems::*;
    use bevy::prelude::*;

    #[test]
    fn test_vegetation_gpu_resources_creation() {
        let mut gpu_resources = VegetationGpuResources::new();

        assert!(!gpu_resources.resources_allocated);
        assert!(!gpu_resources.needs_cleanup());
        assert_eq!(gpu_resources.memory_usage(), 0);

        // Allocate resources
        gpu_resources.allocate_resources(
            Some(1),     // instance_buffer_id
            Some(2),     // atlas_texture_id
            Some(3),     // quadtree_node_id
            1024 * 1024, // 1MB memory usage
        );

        assert!(gpu_resources.resources_allocated);
        assert!(gpu_resources.needs_cleanup());
        assert_eq!(gpu_resources.memory_usage(), 1024 * 1024);
        assert_eq!(gpu_resources.instance_buffer_id, Some(1));
        assert_eq!(gpu_resources.atlas_texture_id, Some(2));
        assert_eq!(gpu_resources.quadtree_node_id, Some(3));
    }

    #[test]
    fn test_vegetation_gpu_memory_tracker() {
        let mut tracker = VegetationGpuMemoryTracker::default();

        assert_eq!(tracker.total_vram_usage, 0);
        assert_eq!(tracker.active_instances, 0);
        assert_eq!(tracker.peak_vram_usage, 0);
        assert_eq!(tracker.cleanup_operations, 0);

        // Allocate memory
        tracker.allocate_memory(1024 * 1024); // 1MB
        assert_eq!(tracker.total_vram_usage, 1024 * 1024);
        assert_eq!(tracker.active_instances, 1);
        assert_eq!(tracker.peak_vram_usage, 1024 * 1024);

        // Allocate more memory
        tracker.allocate_memory(2 * 1024 * 1024); // 2MB
        assert_eq!(tracker.total_vram_usage, 3 * 1024 * 1024);
        assert_eq!(tracker.active_instances, 2);
        assert_eq!(tracker.peak_vram_usage, 3 * 1024 * 1024);

        // Deallocate some memory
        tracker.deallocate_memory(1024 * 1024); // 1MB
        assert_eq!(tracker.total_vram_usage, 2 * 1024 * 1024);
        assert_eq!(tracker.active_instances, 1);
        assert_eq!(tracker.peak_vram_usage, 3 * 1024 * 1024); // Peak unchanged
        assert_eq!(tracker.cleanup_operations, 1);
    }

    #[test]
    fn test_memory_budget_checking() {
        let mut tracker = VegetationGpuMemoryTracker::default();

        // Within budget initially
        assert!(tracker.is_within_memory_budget());
        assert_eq!(tracker.memory_usage_percentage(), 0.0);

        // Allocate 256MB (half of 512MB budget)
        tracker.allocate_memory(256 * 1024 * 1024);
        assert!(tracker.is_within_memory_budget());
        assert!((tracker.memory_usage_percentage() - 50.0).abs() < 0.01);

        // Allocate 300MB more (556MB total - over budget)
        tracker.allocate_memory(300 * 1024 * 1024);
        assert!(!tracker.is_within_memory_budget());
        assert!(tracker.memory_usage_percentage() > 100.0);
    }

    #[test]
    fn test_gpu_resources_cleanup_marking() {
        let mut gpu_resources = VegetationGpuResources::new();

        // Allocate resources
        gpu_resources.allocate_resources(Some(1), Some(2), Some(3), 1024);
        assert!(gpu_resources.needs_cleanup());

        // Mark as cleaned
        gpu_resources.mark_cleaned();
        assert!(!gpu_resources.needs_cleanup());
        assert!(!gpu_resources.resources_allocated);
        assert_eq!(gpu_resources.memory_usage(), 0);
        assert_eq!(gpu_resources.instance_buffer_id, None);
        assert_eq!(gpu_resources.atlas_texture_id, None);
        assert_eq!(gpu_resources.quadtree_node_id, None);
    }

    #[test]
    fn test_vegetation_gpu_cleanup_system_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(VegetationGpuMemoryTracker::default())
            .add_systems(Update, vegetation_gpu_cleanup_system);

        // Create test entity with GPU resources
        let entity = app
            .world_mut()
            .spawn((VegetationLOD::new(), VegetationGpuResources::new()))
            .id();

        // Run one frame
        app.update();

        // Remove the GPU resources component to trigger cleanup detection
        app.world_mut()
            .entity_mut(entity)
            .remove::<VegetationGpuResources>();

        // Run another frame to trigger cleanup
        app.update();

        // Verify cleanup was recorded
        let tracker = app.world().resource::<VegetationGpuMemoryTracker>();
        assert!(tracker.cleanup_operations > 0);
    }

    #[test]
    fn test_vegetation_memory_budget_emergency_culling() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(VegetationGpuMemoryTracker::default())
            .add_systems(Update, vegetation_memory_budget_system);

        // Create test entities with LOD
        let entity1 = app
            .world_mut()
            .spawn((
                VegetationLOD::from_distance(250.0), // Will be culled
                VegetationGpuResources::new(),
            ))
            .id();

        let entity2 = app
            .world_mut()
            .spawn((
                VegetationLOD::from_distance(100.0), // Won't be culled
                VegetationGpuResources::new(),
            ))
            .id();

        // Set memory tracker to over budget
        {
            let mut tracker = app.world_mut().resource_mut::<VegetationGpuMemoryTracker>();
            tracker.total_vram_usage = 600 * 1024 * 1024; // 600MB > 512MB budget
        }

        // Run the system
        app.update();

        // Check that distant entity was culled
        let lod1 = app.world().entity(entity1).get::<VegetationLOD>().unwrap();
        let lod2 = app.world().entity(entity2).get::<VegetationLOD>().unwrap();

        assert_eq!(lod1.detail_level, VegetationDetailLevel::Culled);
        assert_ne!(lod2.detail_level, VegetationDetailLevel::Culled);
    }

    #[test]
    fn test_app_exit_cleanup_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(VegetationGpuMemoryTracker::default())
            .add_systems(Update, vegetation_app_exit_cleanup_system);

        // Create test entities with GPU resources
        let mut gpu_resources1 = VegetationGpuResources::new();
        gpu_resources1.allocate_resources(Some(1), Some(2), Some(3), 1024);

        let mut gpu_resources2 = VegetationGpuResources::new();
        gpu_resources2.allocate_resources(Some(4), Some(5), Some(6), 2048);

        app.world_mut()
            .spawn((VegetationLOD::new(), gpu_resources1));
        app.world_mut()
            .spawn((VegetationLOD::new(), gpu_resources2));

        // Run the cleanup system
        app.update();

        // Verify all GPU resources were cleaned up
        let mut gpu_query = app.world_mut().query::<&VegetationGpuResources>();
        assert_eq!(gpu_query.iter(app.world()).count(), 0);

        // Verify tracker was reset
        let tracker = app.world().resource::<VegetationGpuMemoryTracker>();
        assert_eq!(tracker.total_vram_usage, 0);
        assert_eq!(tracker.active_instances, 0);
    }
}
