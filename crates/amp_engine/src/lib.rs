pub mod memory;
pub mod prelude;
pub mod spatial;
pub mod world;

#[cfg(feature = "bevy16")]
pub mod plugins;

#[cfg(feature = "bevy16")]
pub mod gpu;

#[cfg(feature = "bevy16")]
pub mod assets;

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
