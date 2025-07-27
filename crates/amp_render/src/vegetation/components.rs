//! Vegetation LOD components for GPU rendering

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Vegetation LOD detail levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum VegetationDetailLevel {
    Full,
    Medium,
    Billboard,
    Culled,
}

impl Default for VegetationDetailLevel {
    fn default() -> Self {
        VegetationDetailLevel::Full
    }
}

/// Vegetation LOD component with distance tracking
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct VegetationLOD {
    pub detail_level: VegetationDetailLevel,
    pub distance_to_player: f32,
    pub last_update_frame: u64,
}

impl Default for VegetationLOD {
    fn default() -> Self {
        Self {
            detail_level: VegetationDetailLevel::Full,
            distance_to_player: 0.0,
            last_update_frame: 0,
        }
    }
}

impl VegetationLOD {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_distance(distance: f32) -> Self {
        let mut lod = Self::new();
        lod.update_from_distance(distance, 0);
        lod
    }

    pub fn update_from_distance(&mut self, distance: f32, frame: u64) {
        self.distance_to_player = distance;
        self.last_update_frame = frame;

        // Update detail level based on distance thresholds
        self.detail_level = match distance {
            d if d < 50.0 => VegetationDetailLevel::Full,
            d if d < 150.0 => VegetationDetailLevel::Medium,
            d if d < 300.0 => VegetationDetailLevel::Billboard,
            _ => VegetationDetailLevel::Culled,
        };
    }

    pub fn should_be_visible(&self) -> bool {
        !matches!(self.detail_level, VegetationDetailLevel::Culled)
    }
}

/// Mesh LOD component for vegetation entities
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct VegetationMeshLOD {
    pub full_mesh: Handle<Mesh>,
    pub medium_mesh: Handle<Mesh>,
    pub billboard_mesh: Handle<Mesh>,
}

impl Default for VegetationMeshLOD {
    fn default() -> Self {
        Self {
            full_mesh: Handle::default(),
            medium_mesh: Handle::default(),
            billboard_mesh: Handle::default(),
        }
    }
}

impl VegetationMeshLOD {
    pub fn new(
        full_mesh: Handle<Mesh>,
        medium_mesh: Handle<Mesh>,
        billboard_mesh: Handle<Mesh>,
    ) -> Self {
        Self {
            full_mesh,
            medium_mesh,
            billboard_mesh,
        }
    }

    pub fn get_mesh_for_level(&self, level: VegetationDetailLevel) -> Option<Handle<Mesh>> {
        match level {
            VegetationDetailLevel::Full => Some(self.full_mesh.clone()),
            VegetationDetailLevel::Medium => Some(self.medium_mesh.clone()),
            VegetationDetailLevel::Billboard => Some(self.billboard_mesh.clone()),
            VegetationDetailLevel::Culled => None,
        }
    }
}

/// Billboard component for vegetation entities
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct VegetationBillboard {
    pub texture: Handle<Image>,
    pub size: Vec2,
    pub always_face_camera: bool,
    pub original_scale: Vec3,
}

impl Default for VegetationBillboard {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            size: Vec2::new(1.0, 1.0),
            always_face_camera: true,
            original_scale: Vec3::ONE,
        }
    }
}

impl VegetationBillboard {
    pub fn new(texture: Handle<Image>, size: Vec2) -> Self {
        Self {
            texture,
            size,
            always_face_camera: true,
            original_scale: Vec3::ONE,
        }
    }
}

/// Resource for tracking vegetation LOD performance
#[derive(Resource, Debug, Clone)]
pub struct VegetationLODStats {
    pub full_count: u32,
    pub medium_count: u32,
    pub billboard_count: u32,
    pub culled_count: u32,
    pub total_entities: u32,
}

impl Default for VegetationLODStats {
    fn default() -> Self {
        Self {
            full_count: 0,
            medium_count: 0,
            billboard_count: 0,
            culled_count: 0,
            total_entities: 0,
        }
    }
}

/// Billboard resources for vegetation rendering
#[derive(Resource)]
pub struct VegetationBillboardResources {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// GPU resource tracking component for vegetation entities
///
/// This component tracks GPU resources that need cleanup when the entity is despawned
/// to prevent VRAM leaks during long gaming sessions.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct VegetationGpuResources {
    /// GPU instance buffer ID (if using instanced rendering)
    pub instance_buffer_id: Option<u32>,
    /// GPU texture atlas ID for vegetation textures
    pub atlas_texture_id: Option<u32>,
    /// Quadtree node ID for spatial partitioning
    pub quadtree_node_id: Option<u32>,
    /// GPU memory usage in bytes
    pub gpu_memory_usage: u64,
    /// Whether GPU resources are currently allocated
    pub resources_allocated: bool,
}

impl Default for VegetationGpuResources {
    fn default() -> Self {
        Self {
            instance_buffer_id: None,
            atlas_texture_id: None,
            quadtree_node_id: None,
            gpu_memory_usage: 0,
            resources_allocated: false,
        }
    }
}

impl VegetationGpuResources {
    /// Create new GPU resources tracking component
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate GPU resources and update tracking
    pub fn allocate_resources(
        &mut self,
        instance_buffer_id: Option<u32>,
        atlas_texture_id: Option<u32>,
        quadtree_node_id: Option<u32>,
        memory_usage: u64,
    ) {
        self.instance_buffer_id = instance_buffer_id;
        self.atlas_texture_id = atlas_texture_id;
        self.quadtree_node_id = quadtree_node_id;
        self.gpu_memory_usage = memory_usage;
        self.resources_allocated = true;
    }

    /// Check if resources need cleanup
    pub fn needs_cleanup(&self) -> bool {
        self.resources_allocated
            && (self.instance_buffer_id.is_some()
                || self.atlas_texture_id.is_some()
                || self.quadtree_node_id.is_some())
    }

    /// Get total GPU memory usage
    pub fn memory_usage(&self) -> u64 {
        self.gpu_memory_usage
    }

    /// Mark resources as cleaned up
    pub fn mark_cleaned(&mut self) {
        self.resources_allocated = false;
        self.instance_buffer_id = None;
        self.atlas_texture_id = None;
        self.quadtree_node_id = None;
        self.gpu_memory_usage = 0;
    }
}

impl Drop for VegetationGpuResources {
    fn drop(&mut self) {
        if self.needs_cleanup() {
            // Log that resources need cleanup during drop
            // The actual cleanup will be handled by the cleanup systems
            // to ensure proper coordination with Bevy's ECS
            #[cfg(debug_assertions)]
            {
                bevy::log::debug!(
                    "VegetationGpuResources dropping with allocated resources: instance_buffer={:?}, atlas_texture={:?}, quadtree_node={:?}, memory={}B",
                    self.instance_buffer_id,
                    self.atlas_texture_id,
                    self.quadtree_node_id,
                    self.gpu_memory_usage
                );
            }
        }
    }
}

/// Resource for tracking global vegetation GPU memory usage
#[derive(Resource, Debug, Clone)]
pub struct VegetationGpuMemoryTracker {
    /// Total VRAM allocated for vegetation (bytes)
    pub total_vram_usage: u64,
    /// Number of active vegetation GPU instances
    pub active_instances: u32,
    /// Peak VRAM usage during session
    pub peak_vram_usage: u64,
    /// Number of cleanup operations performed
    pub cleanup_operations: u32,
}

impl Default for VegetationGpuMemoryTracker {
    fn default() -> Self {
        Self {
            total_vram_usage: 0,
            active_instances: 0,
            peak_vram_usage: 0,
            cleanup_operations: 0,
        }
    }
}

impl VegetationGpuMemoryTracker {
    /// Add GPU memory usage tracking
    pub fn allocate_memory(&mut self, bytes: u64) {
        self.total_vram_usage += bytes;
        self.active_instances += 1;
        self.peak_vram_usage = self.peak_vram_usage.max(self.total_vram_usage);
    }

    /// Remove GPU memory usage tracking
    pub fn deallocate_memory(&mut self, bytes: u64) {
        self.total_vram_usage = self.total_vram_usage.saturating_sub(bytes);
        self.active_instances = self.active_instances.saturating_sub(1);
        self.cleanup_operations += 1;
    }

    /// Check if memory usage is within acceptable limits
    pub fn is_within_memory_budget(&self) -> bool {
        const MAX_VEGETATION_VRAM_MB: u64 = 512; // 512MB limit
        self.total_vram_usage < (MAX_VEGETATION_VRAM_MB * 1024 * 1024)
    }

    /// Get memory usage as percentage of budget
    pub fn memory_usage_percentage(&self) -> f32 {
        const MAX_VEGETATION_VRAM_MB: u64 = 512; // 512MB limit
        let budget_bytes = MAX_VEGETATION_VRAM_MB * 1024 * 1024;
        (self.total_vram_usage as f32 / budget_bytes as f32) * 100.0
    }
}
