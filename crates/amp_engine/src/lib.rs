pub mod memory;
pub mod prelude;
pub mod spatial;
pub mod world;
pub mod world_streaming;

#[cfg(feature = "bevy16")]
pub mod hud;

#[cfg(feature = "bevy16")]
pub mod batch;

#[cfg(feature = "bevy16")]
pub mod batch_complex;

#[cfg(feature = "perf_trace")]
pub mod tracing;

#[cfg(feature = "bevy16")]
pub mod plugins;

#[cfg(feature = "bevy16")]
pub mod gpu;

#[cfg(feature = "bevy16")]
pub mod assets;

// Oracle's Performance Strike modules - simplified implementation
pub mod performance_simple;

// Oracle's Performance Strike modules (complex - for bevy16 feature)
#[cfg(feature = "bevy16")]
pub mod performance_benchmarks;
#[cfg(feature = "bevy16")]
pub mod performance_integration;
#[cfg(feature = "bevy16")]
pub mod performance_strike;

#[cfg(all(test, feature = "bevy16"))]
pub mod test_utils;

// Oracle's Day 4-5: Global allocator optimization for release builds
#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "windows"),
    not(debug_assertions)
))]
use jemallocator::Jemalloc;

#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "windows"),
    not(debug_assertions)
))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
