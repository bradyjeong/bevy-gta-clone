//! GPU Culling Phase 2 Unit Tests
//!
//! Tests the GPU culling integration components and ensures they work correctly.

use amp_render::prelude::*;

/// Unit test for GPU culling integration components
#[test]
fn test_gpu_culling_integration_components() {
    // Test GpuCullingResults functionality
    let mut results = GpuCullingResults {
        visibility_data: vec![1, 0, 5, 3], // visible, hidden, visible+LOD2, visible+LOD1
        ..Default::default()
    };

    assert!(results.is_visible(0)); // 1 = 001 -> visible, LOD 0
    assert!(!results.is_visible(1)); // 0 = 000 -> hidden
    assert!(results.is_visible(2)); // 5 = 101 -> visible, LOD 2
    assert!(results.is_visible(3)); // 3 = 011 -> visible, LOD 1

    assert_eq!(results.get_lod_level(0), 0); // 1 >> 1 & 0x3 = 0
    assert_eq!(results.get_lod_level(2), 2); // 5 >> 1 & 0x3 = 2
    assert_eq!(results.get_lod_level(3), 1); // 3 >> 1 & 0x3 = 1

    results.update_stats();
    assert_eq!(results.total_instances, 4);
    assert_eq!(results.visible_instances, 3);
    #[cfg(feature = "gpu_culling")]
    assert!((results.stats.culling_efficiency() - 0.25).abs() < 0.001); // 25% culled

    // Test GpuCulledBatch functionality
    let batch = GpuCulledBatch {
        original_count: 1000,
        visible_count: 600,
    };
    assert!((batch.culling_efficiency() - 0.4).abs() < 0.001); // 40% culled
}

/// Test visibility data encoding/decoding
#[test]
fn test_visibility_data_encoding() {
    let mut results = GpuCullingResults::default();

    // Test all possible LOD levels with visibility
    for lod in 0..4 {
        let encoded = 1 | (lod << 1); // visible + LOD level
        results.visibility_data.push(encoded);
    }

    // Add hidden instance
    results.visibility_data.push(0);

    // Verify encoding/decoding
    for (i, &expected_lod) in [0, 1, 2, 3].iter().enumerate() {
        assert!(results.is_visible(i), "Instance {i} should be visible");
        assert_eq!(
            results.get_lod_level(i),
            expected_lod,
            "Instance {i} should have LOD {expected_lod}"
        );
    }

    assert!(
        !results.is_visible(4),
        "Hidden instance should not be visible"
    );
    assert_eq!(
        results.get_lod_level(4),
        0,
        "Hidden instance LOD should be 0"
    );
}

/// Test culling efficiency calculations
#[test]
fn test_culling_efficiency() {
    // Test 100% visible (0% culled)
    let mut results = GpuCullingResults {
        visibility_data: vec![1; 100],
        ..Default::default()
    };
    results.update_stats();
    #[cfg(feature = "gpu_culling")]
    assert_eq!(results.stats.culling_efficiency(), 0.0);

    // Test 50% visible (50% culled)
    results.visibility_data = [1, 0].repeat(50);
    results.update_stats();
    #[cfg(feature = "gpu_culling")]
    assert_eq!(results.stats.culling_efficiency(), 0.5);

    // Test 100% culled (0% visible)
    results.visibility_data = vec![0; 100];
    results.update_stats();
    #[cfg(feature = "gpu_culling")]
    assert_eq!(results.stats.culling_efficiency(), 1.0);

    // Test empty case
    results.clear(); // Use clear() instead of just clearing visibility_data
    #[cfg(feature = "gpu_culling")]
    assert_eq!(results.stats.culling_efficiency(), 0.0);
}

/// Test batch culling efficiency
#[test]
fn test_batch_culling_efficiency() {
    // Test normal case
    let batch = GpuCulledBatch {
        original_count: 1000,
        visible_count: 700,
    };
    assert_eq!(batch.culling_efficiency(), 0.3); // 30% culled

    // Test edge cases
    let all_visible = GpuCulledBatch {
        original_count: 100,
        visible_count: 100,
    };
    assert_eq!(all_visible.culling_efficiency(), 0.0);

    let all_culled = GpuCulledBatch {
        original_count: 100,
        visible_count: 0,
    };
    assert_eq!(all_culled.culling_efficiency(), 1.0);

    let empty_batch = GpuCulledBatch {
        original_count: 0,
        visible_count: 0,
    };
    assert_eq!(empty_batch.culling_efficiency(), 0.0);
}

/// Test bounds checking for visibility queries
#[test]
fn test_visibility_bounds_checking() {
    let results = GpuCullingResults {
        visibility_data: vec![1, 0, 5], // 3 instances
        ..Default::default()
    };

    // Valid indices
    assert!(results.is_visible(0));
    assert!(!results.is_visible(1));
    assert!(results.is_visible(2));

    // Out of bounds indices should return false/0
    assert!(!results.is_visible(3));
    assert!(!results.is_visible(100));
    assert_eq!(results.get_lod_level(3), 0);
    assert_eq!(results.get_lod_level(100), 0);
}

/// Integration test verifying the complete pipeline
#[test]
fn test_gpu_culling_pipeline_integration() {
    // Simulate a complete GPU culling operation
    let mut results = GpuCullingResults::default();

    // Simulate 10k instances with realistic culling
    let instance_count = 10_000;
    results.visibility_data.reserve(instance_count);

    // Simulate distance-based culling (approximately 30% visible)
    for i in 0..instance_count {
        let distance_factor = (i % 10) as f32 / 10.0;
        let visible = distance_factor < 0.3; // 30% visible

        if visible {
            let lod = if distance_factor < 0.1 {
                0
            } else if distance_factor < 0.2 {
                1
            } else {
                2
            };
            results.visibility_data.push(1 | (lod << 1));
        } else {
            results.visibility_data.push(0);
        }
    }

    results.update_stats();

    // Verify results
    assert_eq!(results.total_instances, instance_count as u32);
    assert!(results.visible_instances > 0);
    assert!(results.visible_instances < results.total_instances);

    #[cfg(feature = "gpu_culling")]
    {
        let efficiency = results.stats.culling_efficiency();
        assert!(efficiency > 0.6 && efficiency < 0.8); // Should cull 60-80%
    }

    #[cfg(feature = "gpu_culling")]
    {
        let efficiency = results.stats.culling_efficiency();
        println!(
            "GPU Culling Pipeline Test: {} instances processed, {} visible ({:.1}% culled)",
            results.total_instances,
            results.visible_instances,
            efficiency * 100.0
        );
    }
    #[cfg(not(feature = "gpu_culling"))]
    {
        println!(
            "GPU Culling Pipeline Test: {} instances processed, {} visible",
            results.total_instances, results.visible_instances
        );
    }
}

/// Performance test for GPU culling data structures
#[test]
fn test_gpu_culling_performance() {
    let start = std::time::Instant::now();

    let mut results = GpuCullingResults::default();
    let instance_count = 100_000;

    // Test allocation and population performance
    results.visibility_data.reserve(instance_count);
    for i in 0..instance_count {
        let visible = (i % 4) != 0; // 75% visible
        let lod = (i % 3) as u32; // Cycle through LOD 0-2

        if visible {
            results.visibility_data.push(1 | (lod << 1));
        } else {
            results.visibility_data.push(0);
        }
    }

    // Test query performance
    let mut visible_count = 0;
    for i in 0..instance_count {
        if results.is_visible(i) {
            visible_count += 1;
            let _lod = results.get_lod_level(i); // Access LOD data
        }
    }

    results.update_stats();

    let elapsed = start.elapsed();
    println!(
        "Performance test: {}k instances processed in {:.3}ms ({:.1} instances/μs)",
        instance_count / 1000,
        elapsed.as_secs_f64() * 1000.0,
        instance_count as f64 / elapsed.as_micros() as f64
    );

    // Performance assertion - should process 100k instances in under 10ms
    assert!(
        elapsed.as_millis() < 10,
        "GPU culling data operations should be fast"
    );
    assert_eq!(visible_count, results.visible_instances as usize);
}
