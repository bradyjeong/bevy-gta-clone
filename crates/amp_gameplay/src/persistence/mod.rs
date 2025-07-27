//! Game persistence system for save/load functionality
//!
//! This module provides comprehensive save and load functionality for the game state,
//! including player data, vehicle states, world state, and game progress.

pub mod load_system;
pub mod plugin;
pub mod save_system;
pub mod serializable;

pub use load_system::*;
pub use plugin::*;
pub use save_system::*;
pub use serializable::*;
