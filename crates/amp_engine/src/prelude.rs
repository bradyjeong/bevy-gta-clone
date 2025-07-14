// Re-export all modules for convenient access

pub use crate::spatial::*;
// Re-export specific items to avoid glob conflicts
pub use crate::memory::*;
pub use crate::world::WorldManager;

#[cfg(feature = "bevy16")]
pub use crate::plugins::{AAAPlugin, AAAPlugins, PluginStage};

#[cfg(feature = "bevy16")]
pub use crate::assets::*;

#[cfg(feature = "bevy16")]
// Re-export GPU module contents (avoiding Window/Color conflicts)
pub use crate::gpu::{GpuContext, GpuError, Surface, SurfaceManager};
