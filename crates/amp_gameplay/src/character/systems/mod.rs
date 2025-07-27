//! Character systems module
//!
//! Contains all character-related systems for input handling, movement, camera control, and animation.

pub mod animation;
pub mod animation_graph;
// pub mod animation_playback;  // Removed temporarily
pub mod animation_playback_stub;
pub mod asset_loading;
pub mod camera;
pub mod input;
pub mod movement;
pub mod visual_animation;

pub use animation::*;
pub use animation_graph::setup_animation_graph_handles;
// pub use animation_playback::*;  // Temporarily disabled
pub use animation_playback_stub::*;
pub use asset_loading::*;
pub use camera::*;
pub use input::*;
pub use movement::*;
pub use visual_animation::*;
