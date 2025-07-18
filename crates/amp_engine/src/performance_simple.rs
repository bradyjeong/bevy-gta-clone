/*!
# Performance Strike: Simplified Implementation

A simplified version of Oracle's performance strike specifications that focuses on
key optimizations while maintaining compatibility with existing amp_engine systems.

## Key Features

1. **Performance Budgets**: Monitor system execution times
2. **Frame-based Scheduling**: Reduce update frequency for non-critical systems
3. **Performance Metrics**: Real-time performance tracking
4. **Optimization Recommendations**: Actionable performance improvements

## Target Performance

- **Target**: <3.0ms median CPU frame time
- **Integration**: Compatible with existing amp_engine architecture
- **Validation**: Simple benchmarking system

## Usage

```rust
use amp_engine::performance_simple::*;

// Add to your app
app.add_plugins(SimplePerformancePlugin);

// Monitor performance
let metrics = app.world.resource::<PerformanceMetrics>();
println!("Frame time: {:.2}ms", metrics.last_frame_time_ms);
```
*/

use std::collections::HashMap;
use std::time::Instant;

#[cfg(feature = "entity_debug")]
use tracing::{debug, info};

// Math types
use glam::Vec3;

// Core performance monitoring

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Target frame time in milliseconds
    pub target_frame_time_ms: f64,
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
    /// Performance budgets for different categories
    pub budgets: PerformanceBudgets,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_frame_time_ms: 3.0,
            monitoring_enabled: true,
            budgets: PerformanceBudgets::default(),
        }
    }
}

/// Performance budgets for different system categories
#[derive(Debug, Clone)]
pub struct PerformanceBudgets {
    /// Transform system budget in milliseconds
    pub transform_budget_ms: f64,
    /// Physics system budget in milliseconds
    pub physics_budget_ms: f64,
    /// LOD system budget in milliseconds
    pub lod_budget_ms: f64,
    /// Rendering system budget in milliseconds
    pub rendering_budget_ms: f64,
    /// Audio system budget in milliseconds
    pub audio_budget_ms: f64,
    /// AI/NPC system budget in milliseconds
    pub ai_budget_ms: f64,
}

impl Default for PerformanceBudgets {
    fn default() -> Self {
        Self {
            transform_budget_ms: 0.75,
            physics_budget_ms: 0.5,
            lod_budget_ms: 0.4,
            rendering_budget_ms: 1.0,
            audio_budget_ms: 0.2,
            ai_budget_ms: 0.15,
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Last frame time in milliseconds
    pub last_frame_time_ms: f64,
    /// Average frame time over last 100 frames
    pub average_frame_time_ms: f64,
    /// System execution times
    pub system_times: HashMap<String, f64>,
    /// Performance violations
    pub budget_violations: Vec<BudgetViolation>,
    /// Frame counter
    pub frame_count: u64,
}

/// Performance budget violation
#[derive(Debug, Clone)]
pub struct BudgetViolation {
    pub system_name: String,
    pub actual_time_ms: f64,
    pub budget_ms: f64,
    pub frame_number: u64,
}

/// Frame counter for scheduler optimization
#[derive(Debug, Default)]
pub struct FrameCounter {
    pub frame_number: u64,
}

/// Optimized distance cache
#[derive(Debug)]
pub struct SimpleDistanceCache {
    /// Entity positions
    positions: HashMap<u32, Vec3>,
    /// Cached distances
    distances: HashMap<u32, f32>,
    /// Cache timestamp
    last_update: Instant,
    /// Time-to-live for cache entries
    ttl_seconds: f32,
}

impl Default for SimpleDistanceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleDistanceCache {
    /// Create new distance cache
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            distances: HashMap::new(),
            last_update: Instant::now(),
            ttl_seconds: 0.1, // 100ms TTL
        }
    }

    /// Get or calculate distance
    pub fn get_distance(&mut self, entity_id: u32, position: Vec3, camera_pos: Vec3) -> f32 {
        // Check if cache is valid
        if self.last_update.elapsed().as_secs_f32() < self.ttl_seconds {
            if let Some(&cached_distance) = self.distances.get(&entity_id) {
                return cached_distance;
            }
        }

        // Calculate and cache new distance
        let distance = position.distance(camera_pos);
        self.positions.insert(entity_id, position);
        self.distances.insert(entity_id, distance);

        distance
    }

    /// Clear expired cache entries
    pub fn update_cache(&mut self) {
        if self.last_update.elapsed().as_secs_f32() >= self.ttl_seconds {
            self.distances.clear();
            self.positions.clear();
            self.last_update = Instant::now();
        }
    }
}

/// Performance monitoring system
pub fn performance_monitoring_system(
    config: &PerformanceConfig,
    metrics: &mut PerformanceMetrics,
    frame_start: Instant,
) {
    if !config.monitoring_enabled {
        return;
    }

    // Update frame metrics
    let frame_time = frame_start.elapsed().as_secs_f64() * 1000.0;
    metrics.last_frame_time_ms = frame_time;
    metrics.frame_count += 1;

    // Update rolling average
    let alpha = 0.1; // Exponential moving average factor
    metrics.average_frame_time_ms =
        alpha * frame_time + (1.0 - alpha) * metrics.average_frame_time_ms;

    // Check for budget violations
    for (system_name, actual_time) in &metrics.system_times {
        let budget = get_system_budget(system_name, &config.budgets);
        if *actual_time > budget {
            metrics.budget_violations.push(BudgetViolation {
                system_name: system_name.clone(),
                actual_time_ms: *actual_time,
                budget_ms: budget,
                frame_number: metrics.frame_count,
            });
        }
    }

    // Keep only recent violations
    if metrics.budget_violations.len() > 100 {
        metrics.budget_violations.drain(0..50);
    }
}

/// Get system budget
fn get_system_budget(system_name: &str, budgets: &PerformanceBudgets) -> f64 {
    match system_name {
        "transform" => budgets.transform_budget_ms,
        "physics" => budgets.physics_budget_ms,
        "lod" => budgets.lod_budget_ms,
        "rendering" => budgets.rendering_budget_ms,
        "audio" => budgets.audio_budget_ms,
        "ai" => budgets.ai_budget_ms,
        _ => 0.1, // Default budget
    }
}

/// Scheduler optimization utilities
pub mod scheduler {
    use super::*;

    /// Check if system should run on this frame
    pub fn should_run_every_2nd_frame(frame_counter: &FrameCounter) -> bool {
        frame_counter.frame_number % 2 == 0
    }

    /// Check if system should run on this frame
    pub fn should_run_every_4th_frame(frame_counter: &FrameCounter) -> bool {
        frame_counter.frame_number % 4 == 0
    }

    /// Check if system should run on this frame
    pub fn should_run_every_8th_frame(frame_counter: &FrameCounter) -> bool {
        frame_counter.frame_number % 8 == 0
    }
}

/// Performance benchmarking utilities
pub mod benchmarks {
    use super::*;

    /// Simple benchmark result
    #[derive(Debug, Clone)]
    pub struct BenchmarkResult {
        pub name: String,
        pub average_time_ms: f64,
        pub min_time_ms: f64,
        pub max_time_ms: f64,
        pub target_time_ms: f64,
        pub target_met: bool,
    }

    /// Run a simple benchmark
    pub fn benchmark_function<F>(
        name: &str,
        iterations: usize,
        target_ms: f64,
        mut f: F,
    ) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let mut times = Vec::new();

        for _ in 0..iterations {
            let start = Instant::now();
            f();
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            times.push(elapsed);
        }

        let average = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = times.iter().fold(0.0f64, |a, &b| a.max(b));

        BenchmarkResult {
            name: name.to_string(),
            average_time_ms: average,
            min_time_ms: min,
            max_time_ms: max,
            target_time_ms: target_ms,
            target_met: average <= target_ms,
        }
    }

    /// Validate performance targets
    pub fn validate_performance(results: &[BenchmarkResult]) -> bool {
        let passed = results.iter().filter(|r| r.target_met).count();
        let total = results.len();

        #[cfg(feature = "entity_debug")]
        {
            info!("Performance Validation Results:");
            info!("===============================");

            for result in results {
                let status = if result.target_met {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                };
                info!(
                    "{}: {:.2}ms (target: {:.2}ms) {}",
                    result.name, result.average_time_ms, result.target_time_ms, status
                );
            }

            info!("Summary: {}/{} targets met", passed, total);
        }

        passed == total
    }
}

/// Performance optimization recommendations
pub mod recommendations {
    use super::*;

    /// Generate optimization recommendations
    pub fn generate_recommendations(
        metrics: &PerformanceMetrics,
        config: &PerformanceConfig,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check overall performance
        if metrics.average_frame_time_ms > config.target_frame_time_ms {
            recommendations.push(format!(
                "Frame time {:.2}ms exceeds target {:.2}ms - consider optimization",
                metrics.average_frame_time_ms, config.target_frame_time_ms
            ));
        }

        // Check system-specific performance
        for (system_name, actual_time) in &metrics.system_times {
            let budget = get_system_budget(system_name, &config.budgets);
            if *actual_time > budget {
                recommendations.push(format!(
                    "{} system: {:.2}ms > {:.2}ms budget - optimize or reduce frequency",
                    system_name, actual_time, budget
                ));
            }
        }

        // Check for recent violations
        let recent_violations = metrics.budget_violations.len();
        if recent_violations > 10 {
            recommendations.push(format!(
                "{} recent budget violations - review system scheduling",
                recent_violations
            ));
        }

        // General recommendations
        if recommendations.is_empty() {
            recommendations.push(
                "Performance targets met! Consider further optimization for headroom.".to_string(),
            );
        } else {
            recommendations.push(
                "Consider: parallel processing, reduced update frequency, or system batching."
                    .to_string(),
            );
        }

        recommendations
    }
}

/// Performance reporting utilities
pub mod reporting {
    use super::*;

    /// Generate performance report
    pub fn generate_report(metrics: &PerformanceMetrics, config: &PerformanceConfig) -> String {
        let mut report = String::new();

        report.push_str("🚀 Performance Strike Report\n");
        report.push_str("============================\n\n");

        // Overall performance
        report.push_str(&format!(
            "Frame Time: {:.2}ms (target: {:.2}ms)\n",
            metrics.average_frame_time_ms, config.target_frame_time_ms
        ));
        report.push_str(&format!("Frame Count: {}\n", metrics.frame_count));
        report.push_str(&format!(
            "Budget Violations: {}\n\n",
            metrics.budget_violations.len()
        ));

        // System performance
        report.push_str("System Performance:\n");
        report.push_str("------------------\n");
        for (system_name, actual_time) in &metrics.system_times {
            let budget = get_system_budget(system_name, &config.budgets);
            let status = if *actual_time <= budget { "✅" } else { "❌" };
            report.push_str(&format!(
                "{}: {:.2}ms / {:.2}ms {}\n",
                system_name, actual_time, budget, status
            ));
        }

        // Recommendations
        report.push_str("\nRecommendations:\n");
        report.push_str("---------------\n");
        for recommendation in recommendations::generate_recommendations(metrics, config) {
            report.push_str(&format!("• {}\n", recommendation));
        }

        report
    }
}

/// Timer utility for measuring system performance
pub struct SystemTimer {
    start: Instant,
    system_name: String,
}

impl SystemTimer {
    /// Start timing a system
    pub fn start(system_name: &str) -> Self {
        Self {
            start: Instant::now(),
            system_name: system_name.to_string(),
        }
    }

    /// Stop timing and record result
    pub fn stop(self, metrics: &mut PerformanceMetrics) {
        let elapsed = self.start.elapsed().as_secs_f64() * 1000.0;
        metrics.system_times.insert(self.system_name, elapsed);
    }
}

/// Simple performance plugin (conceptual - would need Bevy integration)
pub struct SimplePerformancePlugin;

impl SimplePerformancePlugin {
    /// Initialize performance monitoring
    pub fn init() -> (PerformanceConfig, PerformanceMetrics, FrameCounter) {
        (
            PerformanceConfig::default(),
            PerformanceMetrics::default(),
            FrameCounter::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_cache() {
        let mut cache = SimpleDistanceCache::new();
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let camera_pos = Vec3::ZERO;

        let distance = cache.get_distance(1, pos, camera_pos);
        assert!((distance - pos.length()).abs() < 0.001);

        // Should return cached value
        let cached_distance = cache.get_distance(1, pos, camera_pos);
        assert_eq!(distance, cached_distance);
    }

    #[test]
    fn test_scheduler() {
        let mut counter = FrameCounter::default();

        // Frame 0
        assert!(scheduler::should_run_every_2nd_frame(&counter));
        assert!(scheduler::should_run_every_4th_frame(&counter));
        assert!(scheduler::should_run_every_8th_frame(&counter));

        // Frame 1
        counter.frame_number = 1;
        assert!(!scheduler::should_run_every_2nd_frame(&counter));
        assert!(!scheduler::should_run_every_4th_frame(&counter));
        assert!(!scheduler::should_run_every_8th_frame(&counter));

        // Frame 2
        counter.frame_number = 2;
        assert!(scheduler::should_run_every_2nd_frame(&counter));
        assert!(!scheduler::should_run_every_4th_frame(&counter));
        assert!(!scheduler::should_run_every_8th_frame(&counter));
    }

    #[test]
    fn test_benchmark() {
        let result = benchmarks::benchmark_function("test", 10, 1.0, || {
            // Simulate work
            std::thread::sleep(std::time::Duration::from_millis(1));
        });

        assert_eq!(result.name, "test");
        assert!(result.average_time_ms >= 1.0);
        assert!(result.min_time_ms >= 1.0);
        assert!(!result.target_met); // Should exceed 1ms due to overhead
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();
        let config = PerformanceConfig::default();

        // Add some system times
        metrics.system_times.insert("transform".to_string(), 0.5);
        metrics.system_times.insert("physics".to_string(), 0.3);

        // Run monitoring
        performance_monitoring_system(&config, &mut metrics, Instant::now());

        assert!(metrics.frame_count > 0);
        assert!(metrics.budget_violations.is_empty()); // Should be within budget
    }

    #[test]
    fn test_recommendations() {
        let mut metrics = PerformanceMetrics::default();
        let config = PerformanceConfig::default();

        // Add a system that exceeds budget
        metrics.system_times.insert("transform".to_string(), 1.0); // Exceeds 0.75ms budget

        let recommendations = recommendations::generate_recommendations(&metrics, &config);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("transform")));
    }
}
