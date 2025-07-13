//! High-performance batch rendering and GPU culling for Amp
//!
//! This crate provides optimized rendering systems for AAA-level performance,
//! including instance batching, GPU culling, and level-of-detail management.

#![allow(deprecated)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unexpected_cfgs)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::explicit_counter_loop)]
#![allow(clippy::uninlined_format_args)]

pub mod batching;
pub mod culling;
pub mod culling_integration;
pub mod lod;
pub mod render_world;

#[cfg(feature = "gpu")]
pub mod gpu_culling_simple;

#[cfg(feature = "gpu_culling")]
pub mod gpu_culling;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use glam::{Mat4, Vec3};
use std::hash::Hash;

/// Unique identifier for grouping instances into batches
///
/// Instances with the same BatchKey can be rendered together
/// for optimal GPU performance. Uses HandleId::id_u64() for fast hashing.
#[derive(Debug, Clone, PartialEq, Eq, Component)]
pub struct BatchKey {
    /// Mesh handle ID for fast hashing
    pub mesh_id: u64,
    /// Material handle ID for fast hashing  
    pub material_id: u64,
    /// Rendering flags (transparency, shadows, etc.)
    pub flags: u32,
}

impl BatchKey {
    /// Create a new batch key from handles
    pub fn new(mesh: &Handle<Mesh>, material: &Handle<StandardMaterial>) -> Self {
        // Use the AssetId directly converted to u64 for fast hashing
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        mesh.id().hash(&mut hasher);
        let mesh_id = hasher.finish();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        material.id().hash(&mut hasher);
        let material_id = hasher.finish();

        Self {
            mesh_id,
            material_id,
            flags: 0,
        }
    }

    /// Set rendering flags
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Check if this batch key is opaque (for render phase selection)
    pub fn is_opaque(&self) -> bool {
        (self.flags & ALPHA_FLAG) == 0
    }
}

impl Hash for BatchKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Fast hashing using u64 IDs
        self.mesh_id.hash(state);
        self.material_id.hash(state);
        self.flags.hash(state);
    }
}

/// Rendering flags for BatchKey
pub const ALPHA_FLAG: u32 = 1 << 0;
pub const SHADOW_FLAG: u32 = 1 << 1;

/// A batch of instances to be rendered together
///
/// Contains instance data and indirect draw parameters
/// for efficient GPU rendering.
#[derive(Debug, Clone)]
pub struct Batch {
    /// Batch identifier
    pub key: BatchKey,
    /// Instance transform matrices
    pub instances: Vec<Mat4>,
    /// Instance count for indirect drawing
    pub instance_count: u32,
    /// Base instance offset
    pub base_instance: u32,
}

impl Batch {
    /// Create a new empty batch
    pub fn new(key: BatchKey) -> Self {
        Self {
            key,
            instances: Vec::new(),
            instance_count: 0,
            base_instance: 0,
        }
    }

    /// Add an instance to this batch
    pub fn add_instance(&mut self, transform: Mat4) {
        self.instances.push(transform);
        self.instance_count = self.instances.len() as u32;
    }

    /// Clear all instances
    pub fn clear(&mut self) {
        self.instances.clear();
        self.instance_count = 0;
    }

    /// Get the number of instances
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

/// Instance data extracted for render world
///
/// Contains the minimal data needed for rendering
/// an instance in the Bevy render pipeline.
#[derive(Debug, Clone, Component)]
pub struct ExtractedInstance {
    /// World transform matrix
    pub transform: Mat4,
    /// Batch key for grouping
    pub batch_key: BatchKey,
    /// Distance from camera (for LOD and culling)
    pub distance: f32,
    /// Whether this instance is visible
    pub visible: bool,
}

impl ExtractedInstance {
    /// Create a new extracted instance
    pub fn new(transform: Mat4, batch_key: BatchKey, camera_position: Vec3) -> Self {
        let instance_position = transform.w_axis.truncate();
        let distance = camera_position.distance(instance_position);

        Self {
            transform,
            batch_key,
            distance,
            visible: true,
        }
    }

    /// Update visibility based on distance culling
    pub fn update_visibility(&mut self, max_distance: f32) {
        self.visible = self.distance <= max_distance;
    }
}

/// Main plugin for the rendering system
///
/// Registers all rendering systems and resources.
pub struct BatchingPlugin;

impl Plugin for BatchingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            batching::BatchingSystemPlugin,
            culling::CullingSystemPlugin,
            culling_integration::CullingIntegrationPlugin,
            lod::LodSystemPlugin,
            render_world::RenderWorldPlugin,
        ));

        #[cfg(feature = "gpu")]
        {
            // TODO: Re-enable after resolving GPU culling simple integration
            // app.add_plugins(gpu_culling_simple::GpuCullingPlugin);
            // app.add_plugins(gpu_culling_simple::GpuCullingSimplePlugin);
        }

        #[cfg(feature = "gpu_culling")]
        app.add_plugins(gpu_culling::GpuCullingPlugin);
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::{
        ALPHA_FLAG, Batch, BatchKey, BatchingPlugin, ExtractedInstance, SHADOW_FLAG,
        batching::prelude::*, culling::prelude::*, culling_integration::prelude::*,
        lod::prelude::*, render_world::prelude::*,
    };

    #[cfg(feature = "gpu")]
    pub use crate::gpu_culling_simple::prelude::*;

    #[cfg(feature = "gpu_culling")]
    pub use crate::gpu_culling::prelude::*;

    #[cfg(feature = "gpu")]
    pub use crate::gpu_culling_simple::prelude::*;
}
