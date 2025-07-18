//! Benchmark for distance cache performance validation.

#[cfg(test)]
mod benchmarks {
    use super::super::distance_cache::*;
    use bevy::prelude::*;
    use std::time::Instant;

    #[test]
    fn benchmark_distance_cache_vs_direct_calculation() {
        const NUM_ENTITIES: usize = 1_000;
        const NUM_ITERATIONS: usize = 10;

        // Setup entities
        let mut entities = Vec::new();
        let mut positions = Vec::new();
        for i in 0..NUM_ENTITIES {
            entities.push(Entity::from_raw(i as u32));
            positions.push(Vec3::new(
                (i as f32) * 0.1,
                ((i * 7) as f32) * 0.1,
                ((i * 13) as f32) * 0.1,
            ));
        }

        let camera_pos = Vec3::new(0.0, 0.0, 0.0);

        // Benchmark direct distance calculation
        let start = Instant::now();
        let mut direct_distances = Vec::new();
        for _ in 0..NUM_ITERATIONS {
            for pos in &positions {
                direct_distances.push(camera_pos.distance(*pos));
            }
        }
        let direct_time = start.elapsed();

        // Benchmark cached distance calculation
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };

        let start = Instant::now();
        let mut cached_distances = Vec::new();
        for iteration in 0..NUM_ITERATIONS {
            // Update frame counter to simulate frame progression
            let frame_counter = FrameCounter {
                frame: iteration as u32,
            };

            for (entity, pos) in entities.iter().zip(positions.iter()) {
                cached_distances.push(get_cached_distance(
                    &mut cache,
                    &frame_counter,
                    camera_pos,
                    *pos,
                    *entity,
                ));
            }
        }
        let cached_time = start.elapsed();

        // Validate correctness
        for (direct, cached) in direct_distances.iter().zip(cached_distances.iter()) {
            assert!(
                (direct - cached).abs() < 0.001,
                "Distance mismatch: {} vs {}",
                direct,
                cached
            );
        }

        let cache_stats = cache.stats();

        println!("Distance Cache Benchmark Results:");
        println!("  Entities: {}", NUM_ENTITIES);
        println!("  Iterations: {}", NUM_ITERATIONS);
        println!("  Direct calculation time: {:?}", direct_time);
        println!("  Cached calculation time: {:?}", cached_time);
        println!("  Cache stats: {}", cache_stats);

        if cached_time < direct_time {
            let speedup = direct_time.as_nanos() as f64 / cached_time.as_nanos() as f64;
            println!("  Speedup: {:.2}x", speedup);
        }

        // Oracle spec: ~6× fewer Vec3::distance_squared calls
        // With cache hit rate, we should see significant reduction in calculations
        assert!(cache_stats.hits > 0, "Cache should have some hits");
        assert!(
            cache_stats.hit_rate > 0.0,
            "Cache hit rate should be positive"
        );
    }

    #[test]
    fn benchmark_cache_memory_usage() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };

        // Fill cache to capacity
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        for i in 0..2048 {
            let entity = Entity::from_raw(i);
            let pos = Vec3::new(i as f32, 0.0, 0.0);
            get_cached_distance(&mut cache, &frame_counter, camera_pos, pos, entity);
        }

        let stats = cache.stats();
        assert_eq!(stats.size, 2048);
        assert_eq!(stats.capacity, 2048);

        // Adding one more should trigger eviction
        let entity = Entity::from_raw(2048);
        let pos = Vec3::new(2048.0, 0.0, 0.0);
        get_cached_distance(&mut cache, &frame_counter, camera_pos, pos, entity);

        let stats = cache.stats();
        assert_eq!(stats.size, 2048);
        assert_eq!(stats.evictions, 1);

        println!("Cache memory usage validation:");
        println!("  Final cache size: {}", stats.size);
        println!("  Evictions: {}", stats.evictions);
        println!("  Memory stayed within bounds: {}", stats.size <= 2048);
    }

    #[test]
    fn benchmark_cache_accuracy() {
        let mut cache = DistanceCacheResource::default();
        let frame_counter = FrameCounter { frame: 0 };

        // Test accuracy across different distances
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let test_distances = vec![0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0];

        let mut max_error: f32 = 0.0;
        for (i, distance) in test_distances.iter().enumerate() {
            let entity = Entity::from_raw(i as u32);
            let pos = Vec3::new(*distance, 0.0, 0.0);

            let cached_distance =
                get_cached_distance(&mut cache, &frame_counter, camera_pos, pos, entity);
            let direct_distance = camera_pos.distance(pos);

            let error = (cached_distance - direct_distance).abs();
            max_error = max_error.max(error);

            // Oracle spec: <1cm float32 error
            assert!(
                error < 0.01,
                "Distance error {} > 0.01 for distance {}",
                error,
                distance
            );
        }

        println!("Cache accuracy validation:");
        println!("  Max error across all distances: {:.6}", max_error);
        println!("  Oracle spec compliance (<1cm): {}", max_error < 0.01);
    }
}
