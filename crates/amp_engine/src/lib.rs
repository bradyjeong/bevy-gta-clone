//! # amp_engine
//!
//! Core engine systems for the AMP Game Engine.
//!
//! This crate provides:
//! - Spatial partitioning and indexing systems
//! - World management and streaming
//! - GPU abstraction layer
//! - Performance monitoring and benchmarking

pub mod batch;
pub mod gpu;
pub mod hud;
pub mod memory;
pub mod performance_benchmarks;
pub mod performance_integration;
pub mod performance_simple;
pub mod performance_strike;
pub mod plugins;
pub mod prelude;
pub mod spatial;
pub mod spawn_budget;
pub mod test_utils;
pub mod tracing;
pub mod world;
pub mod world_streaming;

// Re-export commonly used items
pub use batch::*;
pub use gpu::*;
pub use hud::*;
pub use memory::*;
pub use plugins::*;
pub use spatial::*;
pub use spawn_budget::*;
pub use world::*;
pub use world_streaming::*;
