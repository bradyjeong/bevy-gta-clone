//! Surface management and render pass abstraction

use crate::gpu::{error::GpuError, GpuContext};
use wgpu::*;
use winit::window::Window;

/// Surface manager that handles swapchain and render passes
pub struct SurfaceManager<'window> {
    /// The wgpu surface
    pub surface: Surface<'window>,
    /// Surface configuration
    pub config: SurfaceConfiguration,
    /// Current surface size
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> SurfaceManager<'window> {
    /// Select adaptive present mode to eliminate VSync stalls
    ///
    /// Prioritizes mailbox mode for low-latency rendering, falls back to immediate mode
    /// to prevent hard VSync locks that cause frame drops and stuttering.
    fn select_adaptive_present_mode(surface_caps: &SurfaceCapabilities) -> PresentMode {
        // Priority order for smooth, low-latency rendering:
        // 1. Mailbox - Triple buffering, no tearing, minimal latency
        // 2. Immediate - No VSync, prevents hard locks but may tear
        // 3. Fifo - VSync fallback (causes stalls but compatible)
        if surface_caps.present_modes.contains(&PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else if surface_caps.present_modes.contains(&PresentMode::Immediate) {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo // VSync fallback
        }
    }

    /// Create a new surface manager
    pub fn new(
        context: &GpuContext,
        window: &'window Window,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<Self, GpuError> {
        let surface = context.instance.create_surface(window)?;

        let surface_caps = surface.get_capabilities(&context.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: Self::select_adaptive_present_mode(&surface_caps),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&context.device, &config);

        Ok(Self {
            surface,
            config,
            size,
        })
    }

    /// Resize the surface
    pub fn resize(&mut self, context: &GpuContext, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&context.device, &self.config);
        }
    }

    /// Get the current surface texture
    pub fn get_current_texture(&self) -> Result<SurfaceTexture, GpuError> {
        self.surface.get_current_texture().map_err(|e| {
            GpuError::SurfaceConfiguration(format!("Failed to get current texture: {e}"))
        })
    }

    /// Create a simple render pass that clears to the specified color
    pub fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
        clear_color: Color,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("amp_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }

    /// Simple frame rendering that clears to purple (for testing)
    pub fn render_purple_frame(&self, context: &GpuContext) -> Result<(), GpuError> {
        let output = self.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = context
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("amp_render_encoder"),
            });

        {
            let _render_pass = self.begin_render_pass(
                &mut encoder,
                &view,
                Color {
                    r: 0.5,
                    g: 0.0,
                    b: 0.5,
                    a: 1.0,
                }, // Purple
            );
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get surface format
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    /// Get surface size
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_manager_color() {
        let purple = Color {
            r: 0.5,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        };
        assert_eq!(purple.r, 0.5);
        assert_eq!(purple.g, 0.0);
        assert_eq!(purple.b, 0.5);
        assert_eq!(purple.a, 1.0);
    }

    #[test]
    fn test_surface_configuration() {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 800,
            height: 600,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.format, TextureFormat::Bgra8UnormSrgb);
    }
}
