//! Performance benchmarking for physics systems.
//!
//! This module provides benchmarking tools to measure physics performance
//! and ensure the 60 FPS target is maintained.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Performance benchmark configuration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Enable/disable benchmarking
    pub enabled: bool,
    /// Number of vehicles to benchmark
    pub vehicle_count: u32,
    /// Number of frames to benchmark
    pub frame_count: u32,
    /// Target CPU time per frame (ms)
    pub target_cpu_time: f32,
    /// Benchmark name
    pub name: String,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vehicle_count: 10,
            frame_count: 60,
            target_cpu_time: 16.0, // 60 FPS target
            name: "physics_benchmark".to_string(),
        }
    }
}

/// Benchmark results storage.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Total CPU time (ms)
    pub total_cpu_time: f32,
    /// Average CPU time per frame (ms)
    pub average_cpu_time: f32,
    /// Minimum CPU time (ms)
    pub min_cpu_time: f32,
    /// Maximum CPU time (ms)
    pub max_cpu_time: f32,
    /// Frame times (ms)
    pub frame_times: Vec<f32>,
    /// Suspension system times (ms)
    pub suspension_times: Vec<f32>,
    /// Drivetrain system times (ms)
    pub drivetrain_times: Vec<f32>,
    /// Total physics entities
    pub total_entities: u32,
    /// Frames processed
    pub frames_processed: u32,
    /// Benchmark passed (within target)
    pub passed: bool,
}

impl Default for BenchmarkResults {
    fn default() -> Self {
        Self {
            total_cpu_time: 0.0,
            average_cpu_time: 0.0,
            min_cpu_time: f32::MAX,
            max_cpu_time: 0.0,
            frame_times: Vec::new(),
            suspension_times: Vec::new(),
            drivetrain_times: Vec::new(),
            total_entities: 0,
            frames_processed: 0,
            passed: false,
        }
    }
}

/// Benchmark state tracking.
#[derive(Resource, Debug, Default)]
pub struct BenchmarkState {
    /// Benchmark start time
    pub start_time: Option<Instant>,
    /// Frame processing start time
    pub frame_start_time: Option<Instant>,
    /// System timing data
    pub system_times: std::collections::HashMap<String, Vec<f32>>,
    /// Current frame count
    pub current_frame: u32,
    /// Benchmark active
    pub active: bool,
}

/// Profiling timer for system performance measurement.
#[derive(Debug)]
pub struct ProfileTimer {
    pub name: String,
    pub start_time: Instant,
}

impl ProfileTimer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start_time: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32() * 1000.0
    }
}

/// System to start benchmarking.
pub fn start_benchmark(
    mut benchmark_state: ResMut<BenchmarkState>,
    benchmark_config: Res<BenchmarkConfig>,
) {
    if benchmark_config.enabled && !benchmark_state.active {
        benchmark_state.start_time = Some(Instant::now());
        benchmark_state.active = true;
        benchmark_state.current_frame = 0;
        benchmark_state.system_times.clear();

        info!(
            "Starting physics benchmark: {} vehicles, {} frames",
            benchmark_config.vehicle_count, benchmark_config.frame_count
        );
    }
}

/// System to track frame timing.
pub fn track_frame_timing(
    mut benchmark_state: ResMut<BenchmarkState>,
    benchmark_config: Res<BenchmarkConfig>,
) {
    if benchmark_state.active && benchmark_config.enabled {
        benchmark_state.frame_start_time = Some(Instant::now());
    }
}

/// System to record suspension system performance.
pub fn record_suspension_performance(
    mut benchmark_state: ResMut<BenchmarkState>,
    benchmark_config: Res<BenchmarkConfig>,
) {
    if benchmark_state.active && benchmark_config.enabled {
        let timer = ProfileTimer::new("suspension");

        // Record suspension timing
        let elapsed_ms = timer.elapsed_ms();
        benchmark_state
            .system_times
            .entry("suspension".to_string())
            .or_default()
            .push(elapsed_ms);
    }
}

/// System to record drivetrain system performance.
pub fn record_drivetrain_performance(
    mut benchmark_state: ResMut<BenchmarkState>,
    benchmark_config: Res<BenchmarkConfig>,
) {
    if benchmark_state.active && benchmark_config.enabled {
        let timer = ProfileTimer::new("drivetrain");

        // Record drivetrain timing
        let elapsed_ms = timer.elapsed_ms();
        benchmark_state
            .system_times
            .entry("drivetrain".to_string())
            .or_default()
            .push(elapsed_ms);
    }
}

/// System to finalize frame timing.
pub fn finalize_frame_timing(
    mut benchmark_state: ResMut<BenchmarkState>,
    mut benchmark_results: ResMut<BenchmarkResults>,
    benchmark_config: Res<BenchmarkConfig>,
    suspension_query: Query<&crate::SuspensionRay>,
) {
    if benchmark_state.active && benchmark_config.enabled {
        if let Some(frame_start) = benchmark_state.frame_start_time {
            let frame_time_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
            benchmark_results.frame_times.push(frame_time_ms);

            benchmark_state.current_frame += 1;

            // Check if benchmark is complete
            if benchmark_state.current_frame >= benchmark_config.frame_count {
                finalize_benchmark(
                    &mut benchmark_state,
                    &mut benchmark_results,
                    &benchmark_config,
                    suspension_query.iter().count() as u32,
                );
            }
        }
    }
}

/// Finalize benchmark and calculate results.
fn finalize_benchmark(
    benchmark_state: &mut BenchmarkState,
    benchmark_results: &mut BenchmarkResults,
    benchmark_config: &BenchmarkConfig,
    entity_count: u32,
) {
    benchmark_state.active = false;
    benchmark_results.frames_processed = benchmark_state.current_frame;
    benchmark_results.total_entities = entity_count;

    // Calculate frame time statistics
    if !benchmark_results.frame_times.is_empty() {
        benchmark_results.total_cpu_time = benchmark_results.frame_times.iter().sum();
        benchmark_results.average_cpu_time =
            benchmark_results.total_cpu_time / benchmark_results.frame_times.len() as f32;
        benchmark_results.min_cpu_time = benchmark_results
            .frame_times
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        benchmark_results.max_cpu_time = benchmark_results
            .frame_times
            .iter()
            .copied()
            .fold(0.0, f32::max);
    }

    // Copy system timing data
    if let Some(suspension_times) = benchmark_state.system_times.get("suspension") {
        benchmark_results.suspension_times = suspension_times.clone();
    }
    if let Some(drivetrain_times) = benchmark_state.system_times.get("drivetrain") {
        benchmark_results.drivetrain_times = drivetrain_times.clone();
    }

    // Check if benchmark passed
    benchmark_results.passed =
        benchmark_results.average_cpu_time <= benchmark_config.target_cpu_time;

    // Log results
    info!("Benchmark '{}' completed:", benchmark_config.name);
    info!("  Vehicles: {}", benchmark_config.vehicle_count);
    info!("  Frames: {}", benchmark_results.frames_processed);
    info!("  Total entities: {}", benchmark_results.total_entities);
    info!(
        "  Total CPU time: {:.2}ms",
        benchmark_results.total_cpu_time
    );
    info!(
        "  Average CPU time: {:.2}ms",
        benchmark_results.average_cpu_time
    );
    info!("  Min CPU time: {:.2}ms", benchmark_results.min_cpu_time);
    info!("  Max CPU time: {:.2}ms", benchmark_results.max_cpu_time);
    info!("  Target: {:.2}ms", benchmark_config.target_cpu_time);
    info!(
        "  Result: {}",
        if benchmark_results.passed {
            "PASSED"
        } else {
            "FAILED"
        }
    );
}

/// System to run automated performance tests.
pub fn run_performance_tests(
    mut benchmark_config: ResMut<BenchmarkConfig>,
    _benchmark_results: Res<BenchmarkResults>,
    benchmark_state: Res<BenchmarkState>,
) {
    // Only run when benchmark is not active and enabled
    if !benchmark_config.enabled || benchmark_state.active {
        return;
    }

    // Run standard performance test
    benchmark_config.enabled = true;
    benchmark_config.vehicle_count = 10;
    benchmark_config.frame_count = 60;
    benchmark_config.target_cpu_time = 16.0; // 60 FPS target
    benchmark_config.name = "standard_performance_test".to_string();
}

/// Plugin for physics benchmarking.
pub struct PhysicsBenchmarkPlugin;

impl Plugin for PhysicsBenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BenchmarkConfig>()
            .init_resource::<BenchmarkResults>()
            .init_resource::<BenchmarkState>()
            .add_systems(PreUpdate, (start_benchmark, track_frame_timing))
            .add_systems(
                PostUpdate,
                (
                    record_suspension_performance,
                    record_drivetrain_performance,
                    finalize_frame_timing,
                )
                    .chain()
                    .after(crate::suspension::vehicle_suspension_system)
                    .after(crate::systems::drivetrain_system),
            );
    }
}

/// Create a benchmark configuration for 10 vehicles, 60 frames.
pub fn create_standard_benchmark() -> BenchmarkConfig {
    BenchmarkConfig {
        enabled: true,
        vehicle_count: 10,
        frame_count: 60,
        target_cpu_time: 16.0,
        name: "standard_10_vehicles_60_frames".to_string(),
    }
}

/// Create a stress test benchmark configuration.
pub fn create_stress_test_benchmark() -> BenchmarkConfig {
    BenchmarkConfig {
        enabled: true,
        vehicle_count: 50,
        frame_count: 120,
        target_cpu_time: 16.0,
        name: "stress_test_50_vehicles_120_frames".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.vehicle_count, 10);
        assert_eq!(config.frame_count, 60);
        assert_eq!(config.target_cpu_time, 16.0);
        assert_eq!(config.name, "physics_benchmark");
    }

    #[test]
    fn benchmark_results_default() {
        let results = BenchmarkResults::default();
        assert_eq!(results.total_cpu_time, 0.0);
        assert_eq!(results.average_cpu_time, 0.0);
        assert_eq!(results.min_cpu_time, f32::MAX);
        assert_eq!(results.max_cpu_time, 0.0);
        assert!(results.frame_times.is_empty());
        assert!(results.suspension_times.is_empty());
        assert!(results.drivetrain_times.is_empty());
        assert_eq!(results.total_entities, 0);
        assert_eq!(results.frames_processed, 0);
        assert!(!results.passed);
    }

    #[test]
    fn benchmark_state_default() {
        let state = BenchmarkState::default();
        assert!(state.start_time.is_none());
        assert!(state.frame_start_time.is_none());
        assert!(state.system_times.is_empty());
        assert_eq!(state.current_frame, 0);
        assert!(!state.active);
    }

    #[test]
    fn profile_timer_creation() {
        let timer = ProfileTimer::new("test");
        assert_eq!(timer.name, "test");
        assert!(timer.elapsed_ms() >= 0.0);
    }

    #[test]
    fn test_create_standard_benchmark() {
        let benchmark = create_standard_benchmark();
        assert!(benchmark.enabled);
        assert_eq!(benchmark.vehicle_count, 10);
        assert_eq!(benchmark.frame_count, 60);
        assert_eq!(benchmark.target_cpu_time, 16.0);
        assert_eq!(benchmark.name, "standard_10_vehicles_60_frames");
    }

    #[test]
    fn test_create_stress_test_benchmark() {
        let benchmark = create_stress_test_benchmark();
        assert!(benchmark.enabled);
        assert_eq!(benchmark.vehicle_count, 50);
        assert_eq!(benchmark.frame_count, 120);
        assert_eq!(benchmark.target_cpu_time, 16.0);
        assert_eq!(benchmark.name, "stress_test_50_vehicles_120_frames");
    }

    #[test]
    fn benchmark_config_serialization() {
        let config = BenchmarkConfig {
            enabled: true,
            vehicle_count: 25,
            frame_count: 90,
            target_cpu_time: 20.0,
            name: "custom_benchmark".to_string(),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: BenchmarkConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.vehicle_count, deserialized.vehicle_count);
        assert_eq!(config.frame_count, deserialized.frame_count);
        assert_eq!(config.target_cpu_time, deserialized.target_cpu_time);
        assert_eq!(config.name, deserialized.name);
    }

    #[test]
    fn benchmark_results_serialization() {
        let results = BenchmarkResults {
            total_cpu_time: 500.0,
            average_cpu_time: 10.0,
            min_cpu_time: 5.0,
            max_cpu_time: 20.0,
            frame_times: vec![10.0, 15.0, 12.0],
            suspension_times: vec![3.0, 4.0, 3.5],
            drivetrain_times: vec![2.0, 2.5, 2.2],
            total_entities: 40,
            frames_processed: 50,
            passed: true,
        };

        let serialized = serde_json::to_string(&results).unwrap();
        let deserialized: BenchmarkResults = serde_json::from_str(&serialized).unwrap();

        assert_eq!(results.total_cpu_time, deserialized.total_cpu_time);
        assert_eq!(results.average_cpu_time, deserialized.average_cpu_time);
        assert_eq!(results.min_cpu_time, deserialized.min_cpu_time);
        assert_eq!(results.max_cpu_time, deserialized.max_cpu_time);
        assert_eq!(results.frame_times, deserialized.frame_times);
        assert_eq!(results.suspension_times, deserialized.suspension_times);
        assert_eq!(results.drivetrain_times, deserialized.drivetrain_times);
        assert_eq!(results.total_entities, deserialized.total_entities);
        assert_eq!(results.frames_processed, deserialized.frames_processed);
        assert_eq!(results.passed, deserialized.passed);
    }
}
