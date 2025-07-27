//! Integration test for hierarchical world streaming system
//!
//! This test verifies that the hierarchical world generation system works correctly
//! with the biome generation and quadtree spatial indexing.

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    
    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_world_lod_manager_hierarchical_loading() {
        let mut manager = WorldLODManager::default();
        
        // Test updating active position
        let position = Vec3::new(1000.0, 0.0, 500.0);
        manager.update_active_position(position, 0.0);
        
        assert_eq!(manager.active_position, position);
        assert_eq!(manager.active_coords.len(), 5); // All LOD levels
        
        // Test chunk loading candidates
        let chunks_to_load = manager.get_chunks_to_load();
        assert!(!chunks_to_load.is_empty());
        
        // Should have chunks for all LOD levels
        let mut lod_levels_found = std::collections::HashSet::new();
        for coord in &chunks_to_load {
            lod_levels_found.insert(coord.level);
        }
        
        assert!(lod_levels_found.contains(&LODLevel::Macro));
        assert!(lod_levels_found.contains(&LODLevel::Region));
        assert!(lod_levels_found.contains(&LODLevel::Local));
        assert!(lod_levels_found.contains(&LODLevel::Detail));
        assert!(lod_levels_found.contains(&LODLevel::Micro));
    }
    
    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_world_coord_hierarchical_relationships() {
        // Test parent-child relationships
        let micro_coord = WorldCoord::new(LODLevel::Micro, 8, 12);
        let detail_parent = micro_coord.get_parent().unwrap();
        
        assert_eq!(detail_parent.level, LODLevel::Detail);
        assert_eq!(detail_parent.x, 2); // 8/4 = 2
        assert_eq!(detail_parent.z, 3); // 12/4 = 3
        
        // Test that parent's children contain the original coord
        let children = detail_parent.get_children();
        assert_eq!(children.len(), 16); // 4x4 subdivision
        assert!(children.contains(&micro_coord));
        
        // Test streaming radius scaling
        assert!(micro_coord.get_streaming_radius() < detail_parent.get_streaming_radius());
    }
    
    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_chunk_generation_priority() {
        let coord = WorldCoord::new(LODLevel::Detail, 0, 0);
        let chunk = WorldChunk::new(coord, 0.0);
        
        // Initially has infinite distance
        assert_eq!(chunk.distance_to_active, f32::INFINITY);
        
        // Priority should be calculated correctly
        let priority = chunk.get_generation_priority();
        assert!(priority > 0.0);
        
        // Higher LOD levels should have higher priority at same distance
        let micro_coord = WorldCoord::new(LODLevel::Micro, 0, 0);
        let micro_chunk = WorldChunk::new(micro_coord, 0.0);
        assert!(micro_chunk.get_generation_priority() > chunk.get_generation_priority());
    }
    
    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_memory_usage_tracking() {
        let manager = WorldLODManager::default();
        let usage = manager.get_memory_usage();
        
        // Initially should have no chunks loaded
        assert_eq!(usage.total_chunks, 0);
        assert_eq!(usage.macro_chunks, 0);
        assert_eq!(usage.region_chunks, 0);
        assert_eq!(usage.local_chunks, 0);
        assert_eq!(usage.detail_chunks, 0);
        assert_eq!(usage.micro_chunks, 0);
        
        // Should have reasonable max limits
        assert!(usage.macro_max > 0);
        assert!(usage.region_max > 0);
        assert!(usage.local_max > 0);  
        assert!(usage.detail_max > 0);
        assert!(usage.micro_max > 0);
    }
    
    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_spatial_index_integration() {
        let manager = WorldLODManager::default();
        
        // Test that spatial index is initialized
        assert_eq!(manager.spatial_index.root_nodes.len(), 0);
        
        // Query empty index should return empty results
        let entities = manager.query_entities_in_radius(Vec3::ZERO, 100.0, LODLevel::Local);
        assert!(entities.is_empty());
        
        // Get streaming coords should work
        let coords = manager.get_streaming_coords();
        assert!(coords.is_empty()); // No active position set yet
    }
    
    #[test]
    fn test_active_entity_marker() {
        // Test that ActiveEntity component can be created
        let entity = ActiveEntity;
        
        // Should be a zero-sized type
        assert_eq!(std::mem::size_of::<ActiveEntity>(), 0);
    }
}
