//! GPU context management

use crate::gpu::error::GpuError;
use wgpu::*;
use winit::window::Window;

/// GPU context that manages the wgpu instance, adapter, device, and queue
pub struct GpuContext {
    /// The wgpu instance
    pub instance: Instance,
    /// The graphics adapter
    pub adapter: Adapter,
    /// The logical device
    pub device: Device,
    /// The command queue
    pub queue: Queue,
}

impl GpuContext {
    /// Create a new GPU context
    pub async fn new(window: &Window) -> Result<Self, GpuError> {
        // Create wgpu instance
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::default(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window)?;

        // Request adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                GpuError::AdapterCreation("Failed to find suitable adapter".to_string())
            })?;

        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("amp_engine_gpu_device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    /// Get adapter information
    pub fn adapter_info(&self) -> AdapterInfo {
        self.adapter.get_info()
    }

    /// Get device features
    pub fn device_features(&self) -> Features {
        self.device.features()
    }

    /// Get device limits
    pub fn device_limits(&self) -> Limits {
        self.device.limits()
    }

    /// Submit command buffers to the queue
    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(&self, commands: I) {
        self.queue.submit(commands);
    }

    /// Write buffer data
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data)
    }

    // Write texture data (temporarily disabled for wgpu 26.0.0 compatibility)
    // pub fn write_texture(
    //     &self,
    //     destination: ImageCopyTexture,
    //     data: &[u8],
    //     layout: ImageDataLayout,
    //     size: Extent3d,
    // ) {
    //     self.queue.write_texture(destination, data, layout, size)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_context_creation() {
        // This test would require a window, so we'll just test the error types
        let err = GpuError::InstanceCreation("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
