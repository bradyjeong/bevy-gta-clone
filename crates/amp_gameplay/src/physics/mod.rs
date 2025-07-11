//! Physics integration for gameplay systems
//!
//! This module provides the bridge between amp_physics algorithms
//! and Bevy gameplay systems.

pub mod plugin;
pub mod resources;

pub use plugin::*;
pub use resources::*;
