/// Advanced Road System for AAA-level open world game
///
/// This module provides a comprehensive road network system with:
/// - Spline-based road generation using Catmull-Rom curves
/// - Multiple road types (Highway, MainStreet, SideStreet, Alley)
/// - Intersection management and traffic flow
/// - Road mesh generation with proper UV mapping
/// - Lane markings and road surface materials
/// - Performance optimization for large road networks
/// - Integration with physics for vehicle pathfinding
pub mod components;
pub mod network;
pub mod plugin;
pub mod resources;
pub mod systems;

#[cfg(test)]
mod tests;

// Re-exports for public API
pub use components::*;
pub use network::*;
pub use plugin::RoadPlugin;
pub use resources::*;
