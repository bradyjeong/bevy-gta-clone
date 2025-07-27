pub mod async_mesh_generation;
pub mod async_road_plugin;
pub mod events;
pub mod materials;
pub mod mesh_attachment;
/// Road mesh generation and rendering systems
///
/// This module provides high-performance mesh generation for roads including:
/// - Road surface mesh generation with proper UV mapping
/// - Lane marking mesh generation (solid and dashed lines)
/// - Intersection mesh generation for different intersection types
/// - Mesh caching for performance optimization
/// - Integration with Bevy's rendering pipeline
pub mod mesh_generation;

// Re-exports for public API
pub use async_mesh_generation::*;
pub use async_road_plugin::*;
pub use events::*;
pub use materials::*;
pub use mesh_attachment::*;
pub use mesh_generation::*;
