//! GPU-specific error types

use thiserror::Error;

/// GPU-specific errors
#[derive(Error, Debug)]
pub enum GpuError {
    /// Failed to create wgpu instance
    #[error("Failed to create wgpu instance: {0}")]
    InstanceCreation(String),

    /// Failed to create wgpu adapter
    #[error("Failed to create wgpu adapter: {0}")]
    AdapterCreation(String),

    /// Failed to create wgpu device
    #[error("Failed to create wgpu device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),

    /// Failed to create surface
    #[error("Failed to create surface: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    /// Surface configuration error
    #[error("Surface configuration error: {0}")]
    SurfaceConfiguration(String),

    /// Render pass error
    #[error("Render pass error: {0}")]
    RenderPass(String),

    /// Shader compilation error
    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),
}

impl From<GpuError> for amp_core::Error {
    fn from(err: GpuError) -> Self {
        amp_core::Error::gpu(err.to_string())
    }
}
