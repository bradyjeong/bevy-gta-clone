//! Asset pipeline for Bevy integration
//!
//! Provides asset loading and management for Amp scene prefabs using Bevy's asset system.

pub mod loader;
pub mod plugin;
pub mod scene;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod hot_reload_test;

#[cfg(test)]
mod app_level_tests;

pub use loader::*;
pub use plugin::*;
pub use scene::*;
