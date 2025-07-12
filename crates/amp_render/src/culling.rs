//! GPU-based frustum and distance culling
//!
//! Provides efficient culling systems for large numbers of instances.

use crate::ExtractedInstance;
use bevy::prelude::*;
use glam::Vec4;

/// Culling configuration resource
#[derive(Resource)]
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
#[derive(Resource, Clone)]
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
#[derive(Component)]
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

    // Extract frustum planes from camera
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
    let frustum_planes = extract_frustum_planes(view_proj);

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

/// Extract 6 frustum planes from view-projection matrix
pub fn extract_frustum_planes(view_proj: Mat4) -> [Vec4; 6] {
    let m = view_proj.to_cols_array_2d();

    [
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
    ]
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

/// Re-exports for convenience
pub mod prelude {
    pub use crate::culling::{
        CameraProjectionConfig, Cullable, CullingConfig, CullingSystemPlugin,
        distance_culling_system, extract_frustum_planes, frustum_culling_system,
    };
}
