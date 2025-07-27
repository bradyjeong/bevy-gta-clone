//! Advanced input system abstractions
//!
//! This module provides high-level input abstractions that sit above Bevy's raw input system.
//! Features include:
//! - Context-sensitive input handling
//! - Hot-reloadable key bindings  
//! - Input event abstraction layer
//! - Performance optimization for high-frequency input events

#[cfg(feature = "unstable_advanced_input")]
pub mod abstractions;
#[cfg(feature = "unstable_advanced_input")]
pub mod actions;
#[cfg(feature = "unstable_advanced_input")]
pub mod context;
#[cfg(feature = "unstable_advanced_input")]
pub mod events;

#[cfg(feature = "unstable_advanced_input")]
pub use abstractions::*;
#[cfg(feature = "unstable_advanced_input")]
pub use actions::*;
#[cfg(feature = "unstable_advanced_input")]
pub use context::*;
#[cfg(feature = "unstable_advanced_input")]
pub use events::*;
