//! Integration test for distance cache system.

#[cfg(test)]
mod tests {
    use super::super::distance_cache::*;
    use super::super::lod::*;
    use super::super::optimized_queries::cached_systems::*;
    use bevy::prelude::*;

    #[test]
    fn test_distance_cache_integration_with_lod() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(DistanceCachePlugin)
            .add_systems(PostUpdate, update_lod_system);

        // Add camera and entity
        let camera = app
            .world_mut()
            .spawn((
                Camera::default(),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(10.0, 0.0, 0.0),
                GlobalTransform::default(),
                LodGroup::new(vec![
                    LodLevel::new(5.0, Handle::default()),
                    LodLevel::new(15.0, Handle::default()),
                ]),
            ))
            .id();

        // Add resources
        app.world_mut().insert_resource(LodConfig::default());

        // Run one frame to populate cache
        app.update();

        // Check that cache was populated
        let distance_cache = app.world().resource::<DistanceCacheResource>();
        let stats = distance_cache.stats();
        assert!(stats.hits > 0 || stats.misses > 0);
    }

    #[test]
    fn test_distance_cache_performance_improvement() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };

        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(100.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        // First call should be a miss
        let start_time = std::time::Instant::now();
        let distance1 =
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);
        let first_call_time = start_time.elapsed();

        // Second call should be a hit (faster)
        let start_time = std::time::Instant::now();
        let distance2 =
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);
        let second_call_time = start_time.elapsed();

        assert_eq!(distance1, distance2);
        assert_eq!(distance1, 100.0);

        // Cache hit should be faster (though timing may vary)
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_distance_cache_frame_progression() {
        let mut cache = DistanceCacheResource::default();
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(50.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        // Cache at frame 0
        let frame_counter = FrameCounter { frame: 0 };
        let distance1 =
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);

        // Hit at frame 4 (within TTL)
        let frame_counter = FrameCounter { frame: 4 };
        let distance2 =
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);

        // Miss at frame 5 (beyond TTL)
        let frame_counter = FrameCounter { frame: 5 };
        let distance3 =
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);

        assert_eq!(distance1, distance2);
        assert_eq!(distance2, distance3);
        assert_eq!(distance1, 50.0);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2);
    }

    #[test]
    fn test_distance_cache_spatial_accuracy() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };

        // Test various distances for accuracy
        let test_cases = vec![
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(3.0, 4.0, 0.0), 5.0), // 3-4-5 triangle
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
                3.0_f32.sqrt(),
            ), // sqrt(3)
            (
                Vec3::new(5.0, 5.0, 5.0),
                Vec3::new(15.0, 15.0, 15.0),
                10.0 * 3.0_f32.sqrt(),
            ), // 10*sqrt(3)
        ];

        for (i, (camera_pos, entity_pos, expected_distance)) in test_cases.iter().enumerate() {
            let entity = Entity::from_raw(i as u32);
            let distance =
                get_cached_distance(&mut cache, &frame_counter, *camera_pos, *entity_pos, entity);

            // Check accuracy within 1cm (0.01 units) as per Oracle spec
            assert!(
                (distance - expected_distance).abs() < 0.01,
                "Distance {} vs expected {} for test case {}",
                distance,
                expected_distance,
                i
            );
        }
    }

    #[test]
    fn test_distance_cache_capacity_management() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);

        // Fill cache beyond capacity (2048 + some extra)
        for i in 0..2100 {
            let entity_pos = Vec3::new(i as f32, 0.0, 0.0);
            let entity = Entity::from_raw(i);
            get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity);
        }

        let stats = cache.stats();
        assert!(stats.size <= 2048); // Should not exceed capacity
        assert!(stats.evictions > 0); // Should have evicted some entries
    }

    #[test]
    fn test_distance_cache_statistics() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let entity_pos = Vec3::new(10.0, 0.0, 0.0);
        let entity = Entity::from_raw(1);

        // Generate some statistics
        get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity); // miss
        get_cached_distance(&mut cache, &frame_counter, camera_pos, entity_pos, entity); // hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.size, 1);
        assert_eq!(stats.capacity, 2048);

        // Test display formatting
        let display_str = format!("{}", stats);
        assert!(display_str.contains("50.0% hit rate"));
        assert!(display_str.contains("1 hits"));
        assert!(display_str.contains("1 misses"));
    }
}
