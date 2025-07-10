//! GPU abstraction layer for the Amp game engine
//!
//! This module provides a high-level interface over wgpu for GPU operations,
//! including device creation, surface management, and render pass abstraction.

#![deny(missing_docs)]

pub mod context;
pub mod error;
pub mod surface;

pub use context::*;
pub use error::*;
pub use surface::*;

/// Re-export commonly used wgpu types
pub use wgpu::{
    Color, CommandEncoder, Device, Extent3d, Features, Limits, PresentMode, Queue, RenderPass,
    Surface, SurfaceConfiguration, TextureFormat, TextureView,
};

/// Re-export winit types for window management
pub use winit::{event_loop::EventLoop, window::Window};
