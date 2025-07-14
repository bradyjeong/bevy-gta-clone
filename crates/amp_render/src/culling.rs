//! GPU-based frustum and distance culling
//!
//! Provides efficient culling systems for large numbers of instances.

use crate::ExtractedInstance;
use bevy::prelude::*;
use glam::Vec4;
use thiserror::Error;

/// Errors that can occur during culling operations
#[derive(Debug, Error)]
pub enum CullingError {
    /// Invalid camera matrix (contains NaN or infinity)
    #[error("Invalid camera matrix: {reason}")]
    InvalidCameraMatrix { reason: String },

    /// Camera transform is invalid
    #[error("Camera transform is invalid: {reason}")]
    InvalidCameraTransform { reason: String },

    /// Projection matrix is invalid
    #[error("Projection matrix is invalid: {reason}")]
    InvalidProjectionMatrix { reason: String },
}

/// Culling configuration resource
#[derive(Resource, Debug, Clone)]
pub struct CullingConfig {
    /// Maximum render distance for objects
    pub max_distance: f32,
    /// Enable frustum culling
    pub enable_frustum_culling: bool,
    /// Enable distance culling
    pub enable_distance_culling: bool,
}

impl Default for CullingConfig {
    fn default() -> Self {
        Self {
            max_distance: 1000.0,
            enable_frustum_culling: true,
            enable_distance_culling: true,
        }
    }
}

/// Camera projection configuration for frustum culling
#[derive(Resource, Debug, Clone)]
pub struct CameraProjectionConfig {
    /// Field of view in radians (default: 45°)
    pub fov: f32,
    /// Aspect ratio (default: 16:9)
    pub aspect_ratio: f32,
    /// Near plane distance
    pub near: f32,
    /// Far plane distance
    pub far: f32,
}

impl Default for CameraProjectionConfig {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_4, // 45°
            aspect_ratio: 16.0 / 9.0,         // 16:9
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Component for objects that can be culled
#[derive(Component, Debug, Clone)]
pub struct Cullable {
    /// Bounding sphere radius
    pub radius: f32,
    /// Culling distance override
    pub max_distance: Option<f32>,
}

impl Cullable {
    /// Create a new cullable component
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            max_distance: None,
        }
    }

    /// Set custom culling distance
    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = Some(distance);
        self
    }
}

/// System to perform distance culling on extracted instances
pub fn distance_culling_system(
    culling_config: Res<CullingConfig>,
    mut instances: Query<&mut ExtractedInstance>,
) {
    if !culling_config.enable_distance_culling {
        return;
    }

    for mut instance in instances.iter_mut() {
        instance.update_visibility(culling_config.max_distance);
    }
}

/// System to perform frustum culling
///
/// CPU fallback implementation using 6-plane frustum intersection
/// Features robust error handling for invalid camera matrices
pub fn frustum_culling_system(
    culling_config: Res<CullingConfig>,
    projection_config: Res<CameraProjectionConfig>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&Projection>)>,
    mut instances: Query<(&mut ExtractedInstance, &Cullable)>,
) {
    if !culling_config.enable_frustum_culling {
        return;
    }

    let Some((_camera, camera_transform, camera_projection)) = cameras.iter().next() else {
        return;
    };

    // Extract frustum planes from camera with error handling
    let view = camera_transform.compute_matrix().inverse();

    // Use actual camera projection if available, otherwise use config defaults
    let projection = if let Some(Projection::Perspective(persp)) = camera_projection {
        Mat4::perspective_lh(persp.fov, persp.aspect_ratio, persp.near, persp.far)
    } else {
        // Fallback to configured defaults
        Mat4::perspective_lh(
            projection_config.fov,
            projection_config.aspect_ratio,
            projection_config.near,
            projection_config.far,
        )
    };

    let view_proj = projection * view;

    // Use robust frustum extraction with error handling
    let frustum_planes = match extract_frustum_planes_safe(view_proj) {
        Ok(planes) => planes,
        Err(e) => {
            // Log error but don't panic - skip culling this frame
            warn!("Frustum culling error: {}", e);
            return;
        }
    };

    for (mut instance, cullable) in instances.iter_mut() {
        if !instance.visible {
            continue;
        }

        let position = instance.transform.w_axis.truncate();
        let radius = cullable.radius;

        // Test sphere against all 6 frustum planes
        let mut inside_frustum = true;
        for plane in &frustum_planes {
            let distance = plane.xyz().dot(position) + plane.w;
            if distance < -radius {
                inside_frustum = false;
                break;
            }
        }

        instance.visible = inside_frustum;
    }
}

/// Test if sphere is inside frustum (optimized version for Sprint 9)
pub fn sphere_in_frustum(position: Vec3, radius: f32, planes: &[Vec4; 6]) -> bool {
    for plane in planes {
        let distance = plane.xyz().dot(position) + plane.w;
        if distance < -radius {
            return false;
        }
    }
    true
}

/// Extract 6 frustum planes from view-projection matrix (legacy - use extract_frustum_planes_safe)
pub fn extract_frustum_planes(view_proj: Mat4) -> [Vec4; 6] {
    match extract_frustum_planes_safe(view_proj) {
        Ok(planes) => planes,
        Err(e) => {
            // Legacy behavior - panic on error
            panic!("extract_frustum_planes failed: {}", e);
        }
    }
}

/// Extract 6 frustum planes from view-projection matrix with robust error handling
///
/// Returns Result instead of panicking when matrix contains NaN or infinity values
pub fn extract_frustum_planes_safe(view_proj: Mat4) -> Result<[Vec4; 6], CullingError> {
    // Validate input matrix
    let m = view_proj.to_cols_array_2d();

    // Check for NaN or infinity in the matrix
    for (i, row) in m.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if !value.is_finite() {
                return Err(CullingError::InvalidCameraMatrix {
                    reason: format!("Matrix element [{}, {}] is not finite: {}", i, j, value),
                });
            }
        }
    }

    let planes = [
        // Left plane
        Vec4::new(
            m[3][0] + m[0][0],
            m[3][1] + m[0][1],
            m[3][2] + m[0][2],
            m[3][3] + m[0][3],
        ),
        // Right plane
        Vec4::new(
            m[3][0] - m[0][0],
            m[3][1] - m[0][1],
            m[3][2] - m[0][2],
            m[3][3] - m[0][3],
        ),
        // Bottom plane
        Vec4::new(
            m[3][0] + m[1][0],
            m[3][1] + m[1][1],
            m[3][2] + m[1][2],
            m[3][3] + m[1][3],
        ),
        // Top plane
        Vec4::new(
            m[3][0] - m[1][0],
            m[3][1] - m[1][1],
            m[3][2] - m[1][2],
            m[3][3] - m[1][3],
        ),
        // Near plane
        Vec4::new(
            m[3][0] + m[2][0],
            m[3][1] + m[2][1],
            m[3][2] + m[2][2],
            m[3][3] + m[2][3],
        ),
        // Far plane
        Vec4::new(
            m[3][0] - m[2][0],
            m[3][1] - m[2][1],
            m[3][2] - m[2][2],
            m[3][3] - m[2][3],
        ),
    ];

    // Validate resulting planes
    for (i, plane) in planes.iter().enumerate() {
        if !plane.x.is_finite()
            || !plane.y.is_finite()
            || !plane.z.is_finite()
            || !plane.w.is_finite()
        {
            return Err(CullingError::InvalidCameraMatrix {
                reason: format!(
                    "Frustum plane {} contains non-finite values: {:?}",
                    i, plane
                ),
            });
        }
    }

    Ok(planes)
}

/// Plugin for culling systems
pub struct CullingSystemPlugin;

impl Plugin for CullingSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CullingConfig>()
            .init_resource::<CameraProjectionConfig>()
            .add_systems(
                PostUpdate,
                (distance_culling_system, frustum_culling_system).chain(),
            );
    }
}

/// GPU Culling bind-group layout resource
#[cfg(feature = "gpu_culling")]
#[derive(Resource)]
pub struct GpuCullBindGroupLayouts {
    /// Bind group layout for instance data buffer
    pub instance_data: bevy::render::render_resource::BindGroupLayout,
    /// Bind group layout for culling parameters buffer
    pub culling_params: bevy::render::render_resource::BindGroupLayout,
    /// Bind group layout for output visibility buffer
    pub visibility_output: bevy::render::render_resource::BindGroupLayout,
}

/// Plugin for GPU culling resources
#[cfg(feature = "gpu_culling")]
pub struct GpuCullingResourcePlugin;

#[cfg(feature = "gpu_culling")]
impl Plugin for GpuCullingResourcePlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::RenderApp;

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(bevy::render::Render, create_gpu_cull_bind_group_layouts);
        }
    }
}

/// System to create GPU culling bind group layouts
#[cfg(feature = "gpu_culling")]
pub fn create_gpu_cull_bind_group_layouts(
    mut commands: Commands,
    render_device: Res<bevy::render::renderer::RenderDevice>,
) {
    use bevy::render::render_resource::*;

    // Instance data buffer layout
    let instance_data_layout = render_device.create_bind_group_layout(
        Some("instance_data_layout"),
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    );

    // Culling parameters buffer layout
    let culling_params_layout = render_device.create_bind_group_layout(
        Some("culling_params_layout"),
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    );

    // Visibility output buffer layout
    let visibility_output_layout = render_device.create_bind_group_layout(
        Some("visibility_output_layout"),
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    );

    commands.insert_resource(GpuCullBindGroupLayouts {
        instance_data: instance_data_layout,
        culling_params: culling_params_layout,
        visibility_output: visibility_output_layout,
    });
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::culling::{
        CameraProjectionConfig, Cullable, CullingConfig, CullingError, CullingSystemPlugin,
        distance_culling_system, extract_frustum_planes, extract_frustum_planes_safe,
        frustum_culling_system,
    };

    #[cfg(feature = "gpu_culling")]
    pub use crate::culling::{
        GpuCullBindGroupLayouts, GpuCullingResourcePlugin, create_gpu_cull_bind_group_layouts,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_extraction_error_handling() {
        // Test with NaN values
        let nan_matrix = Mat4::from_cols(
            Vec4::new(f32::NAN, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let result = extract_frustum_planes_safe(nan_matrix);
        assert!(result.is_err());

        if let Err(CullingError::InvalidCameraMatrix { reason }) = result {
            assert!(reason.contains("not finite"));
        } else {
            panic!("Expected InvalidCameraMatrix error");
        }

        // Test with infinity values
        let inf_matrix = Mat4::from_cols(
            Vec4::new(f32::INFINITY, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let result = extract_frustum_planes_safe(inf_matrix);
        assert!(result.is_err());

        // Test with valid matrix
        let valid_matrix =
            Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 1000.0);

        let result = extract_frustum_planes_safe(valid_matrix);
        assert!(result.is_ok());

        let planes = result.unwrap();
        assert_eq!(planes.len(), 6);

        // Verify all planes are finite
        for plane in planes.iter() {
            assert!(plane.x.is_finite());
            assert!(plane.y.is_finite());
            assert!(plane.z.is_finite());
            assert!(plane.w.is_finite());
        }
    }

    #[test]
    fn test_legacy_frustum_extraction_panics() {
        // Test that legacy function still panics for compatibility
        let nan_matrix = Mat4::from_cols(
            Vec4::new(f32::NAN, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let result = std::panic::catch_unwind(|| {
            extract_frustum_planes(nan_matrix);
        });

        assert!(result.is_err());
    }

    #[cfg(feature = "gpu_culling")]
    #[test]
    fn test_gpu_cull_bind_group_layouts() {
        use crate::assert_layout_matches;
        use bevy::render::render_resource::*;

        // Test instance data layout
        let instance_data_desc = BindGroupLayoutDescriptor {
            label: Some("GPU Culling Instance Data"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

        let instance_data_golden = r#"
        {
            "label": "GPU Culling Instance Data",
            "entries": [
                {
                    "binding": 0,
                    "visibility": 4,
                    "ty": {
                        "Buffer": {
                            "ty": {
                                "Storage": {
                                    "read_only": true
                                }
                            },
                            "has_dynamic_offset": false,
                            "min_binding_size": null
                        }
                    },
                    "count": null
                }
            ]
        }
        "#;

        assert_layout_matches!(instance_data_desc, instance_data_golden);

        // Test culling parameters layout
        let culling_params_desc = BindGroupLayoutDescriptor {
            label: Some("GPU Culling Parameters"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

        let culling_params_golden = r#"
        {
            "label": "GPU Culling Parameters",
            "entries": [
                {
                    "binding": 0,
                    "visibility": 4,
                    "ty": {
                        "Buffer": {
                            "ty": "Uniform",
                            "has_dynamic_offset": false,
                            "min_binding_size": null
                        }
                    },
                    "count": null
                }
            ]
        }
        "#;

        assert_layout_matches!(culling_params_desc, culling_params_golden);

        // Test visibility output layout
        let visibility_output_desc = BindGroupLayoutDescriptor {
            label: Some("GPU Culling Visibility Output"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

        let visibility_output_golden = r#"
        {
            "label": "GPU Culling Visibility Output",
            "entries": [
                {
                    "binding": 0,
                    "visibility": 4,
                    "ty": {
                        "Buffer": {
                            "ty": {
                                "Storage": {
                                    "read_only": false
                                }
                            },
                            "has_dynamic_offset": false,
                            "min_binding_size": null
                        }
                    },
                    "count": null
                }
            ]
        }
        "#;

        assert_layout_matches!(visibility_output_desc, visibility_output_golden);
    }
}

#[cfg(feature = "gpu_culling")]
mod shader_compilation {
    /// GPU culling compute shader source
    #[allow(dead_code)]
    pub const CULLING_SHADER_SOURCE: &str = include_str!("../assets/shaders/culling.wgsl");

    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(target_os = "linux")]
        #[test]
        fn gpu_culling_compile() {
            // Create a minimal wgpu instance for shader compilation testing
            // This test verifies that our WGSL shader compiles correctly

            // Use pollster for async runtime in tests
            pollster::block_on(async {
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::METAL,
                    dx12_shader_compiler: Default::default(),
                    flags: wgpu::InstanceFlags::default(),
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                });

                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: None,
                        force_fallback_adapter: false,
                    })
                    .await;

                // If no adapter is available (headless CI), skip the test
                let adapter = match adapter {
                    Some(adapter) => adapter,
                    None => {
                        eprintln!("No wgpu adapter available, skipping shader compilation test");
                        return;
                    }
                };

                let (device, _queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: Some("Test Device"),
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::default(),
                            memory_hints: wgpu::MemoryHints::default(),
                        },
                        None,
                    )
                    .await
                    .expect("Failed to create device");

                // Test shader compilation
                let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("GPU Culling Compute Shader"),
                    source: wgpu::ShaderSource::Wgsl(CULLING_SHADER_SOURCE.into()),
                });

                // If we get here without panicking, the shader compiled successfully
                drop(shader_module);
                println!("GPU culling shader compiled successfully");
            });
        }
    }
}
