//! Vegetation LOD (Level of Detail) rendering system
//!
//! This module provides GPU-accelerated vegetation rendering with adaptive LOD
//! based on distance to improve performance while maintaining visual quality.

#[cfg(feature = "unstable_vegetation_lod")]
pub mod components;
#[cfg(feature = "unstable_vegetation_lod")]
pub mod plugin;
#[cfg(feature = "unstable_vegetation_lod")]
pub mod systems;

#[cfg(all(test, feature = "unstable_vegetation_lod"))]
mod tests;

#[cfg(all(test, feature = "unstable_vegetation_lod"))]
mod gpu_cleanup_tests;

#[cfg(feature = "unstable_vegetation_lod")]
pub use components::*;
#[cfg(feature = "unstable_vegetation_lod")]
pub use plugin::VegetationLODPlugin;
#[cfg(feature = "unstable_vegetation_lod")]
pub use systems::*;
