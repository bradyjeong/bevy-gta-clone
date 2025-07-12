//! Level-of-detail (LOD) management system
//!
//! Enhanced LOD system with hysteresis, smooth transitions, and BatchManager integration.
//! Provides distance-based switching with efficient change tracking.

use crate::{BatchKey, ExtractedInstance};
use bevy::prelude::*;
use smallvec::SmallVec;

#[cfg(test)]
mod tests;

/// LOD level configuration
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Distance threshold for this LOD
    pub distance: f32,
    /// Mesh for this LOD level
    pub mesh: Handle<Mesh>,
    /// Material for this LOD level (optional override)
    pub material: Option<Handle<StandardMaterial>>,
}

impl LodLevel {
    /// Create a new LOD level
    pub fn new(distance: f32, mesh: Handle<Mesh>) -> Self {
        Self {
            distance,
            mesh,
            material: None,
        }
    }

    /// Set custom material for this LOD
    pub fn with_material(mut self, material: Handle<StandardMaterial>) -> Self {
        self.material = Some(material);
        self
    }
}

/// Enhanced LOD group component with hysteresis and smooth transitions
#[derive(Component, Debug)]
pub struct LodGroup {
    /// Available LOD levels (up to 4 for optimal performance)
    pub levels: SmallVec<[LodLevel; 4]>,
    /// Currently active LOD index
    pub current_lod: usize,
    /// Previous LOD index for transition detection
    pub previous_lod: usize,
    /// Hysteresis distance for LOD switching stability
    pub hysteresis: f32,
    /// Last calculated distance for hysteresis
    pub last_distance: f32,
    /// Cross-fade factor for smooth transitions (0.0 to 1.0)
    pub cross_fade_factor: f32,
    /// Cross-fade duration in seconds
    pub cross_fade_duration: f32,
}

/// Marker component for entities with changed LOD
///
/// Added to entities when their LOD level changes to optimize extraction.
/// Automatically removed after processing.
#[derive(Component)]
pub struct ChangedLod {
    /// Previous LOD index
    pub from_lod: usize,
    /// New LOD index  
    pub to_lod: usize,
    /// Frame when change occurred
    pub changed_frame: u32,
}

impl LodGroup {
    /// Create a new LOD group with default hysteresis
    pub fn new(mut levels: Vec<LodLevel>) -> Self {
        // Sort levels by distance (closest first)
        levels.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        Self {
            levels: SmallVec::from_vec(levels),
            current_lod: 0,
            previous_lod: 0,
            hysteresis: 8.0, // Default 8m hysteresis
            last_distance: 0.0,
            cross_fade_factor: 0.0,
            cross_fade_duration: 0.3, // 300ms default cross-fade
        }
    }

    /// Create LOD group with custom hysteresis
    pub fn with_hysteresis(mut self, hysteresis: f32) -> Self {
        self.hysteresis = hysteresis;
        self
    }

    /// Create LOD group with custom cross-fade duration
    pub fn with_cross_fade_duration(mut self, duration: f32) -> Self {
        self.cross_fade_duration = duration;
        self
    }

    /// Get the appropriate LOD level for distance with hysteresis
    pub fn get_lod_for_distance(&self, distance: f32) -> usize {
        // For first calculation, don't apply hysteresis
        let adjusted_distance = if self.last_distance == 0.0 {
            distance
        } else if distance > self.last_distance {
            // Moving away - apply hysteresis to prevent rapid switching
            distance + self.hysteresis
        } else {
            // Moving closer - apply hysteresis to prevent rapid switching
            distance - self.hysteresis
        };

        for (index, level) in self.levels.iter().enumerate() {
            if adjusted_distance <= level.distance {
                return index;
            }
        }
        // Return highest LOD (lowest quality) if beyond all thresholds
        self.levels.len().saturating_sub(1)
    }

    /// Get LOD for distance without hysteresis (for testing)
    pub fn get_lod_for_distance_simple(&self, distance: f32) -> usize {
        for (index, level) in self.levels.iter().enumerate() {
            if distance <= level.distance {
                return index;
            }
        }
        // Return highest LOD (lowest quality) if beyond all thresholds
        self.levels.len().saturating_sub(1)
    }

    /// Update current LOD with hysteresis and change tracking
    pub fn update_lod(&mut self, distance: f32) -> bool {
        let new_lod = self.get_lod_for_distance(distance);
        let changed = new_lod != self.current_lod;

        if changed {
            self.previous_lod = self.current_lod;
            self.current_lod = new_lod;
            self.cross_fade_factor = 0.0; // Start new cross-fade
        }

        self.last_distance = distance;
        changed
    }

    /// Update cross-fade factor for smooth transitions
    pub fn update_cross_fade(&mut self, delta_time: f32) {
        if self.current_lod != self.previous_lod && self.cross_fade_factor < 1.0 {
            self.cross_fade_factor += delta_time / self.cross_fade_duration;
            self.cross_fade_factor = self.cross_fade_factor.min(1.0);
        }
    }

    /// Get the current LOD level
    pub fn current_level(&self) -> Option<&LodLevel> {
        self.levels.get(self.current_lod)
    }

    /// Get the previous LOD level (for cross-fading)
    pub fn previous_level(&self) -> Option<&LodLevel> {
        self.levels.get(self.previous_lod)
    }

    /// Check if currently cross-fading between LODs
    pub fn is_cross_fading(&self) -> bool {
        self.current_lod != self.previous_lod && self.cross_fade_factor < 1.0
    }

    /// Get batch key for current LOD level with material override
    pub fn get_batch_key(&self, base_material: &Handle<StandardMaterial>) -> Option<BatchKey> {
        self.current_level().map(|level| {
            let material = level.material.as_ref().unwrap_or(base_material);
            BatchKey::new(&level.mesh, material)
        })
    }
}

/// Resource for global LOD configuration
#[derive(Resource)]
pub struct LodConfig {
    /// Enable LOD system
    pub enabled: bool,
    /// LOD bias multiplier (higher = switch sooner)
    pub bias: f32,
    /// Minimum LOD distance
    pub min_distance: f32,
    /// Enable cross-fade transitions
    pub cross_fade_enabled: bool,
    /// Maximum instances per frame for LOD updates (performance limit)
    pub max_updates_per_frame: usize,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bias: 1.0,
            min_distance: 1.0,
            cross_fade_enabled: true,
            max_updates_per_frame: 1000,
        }
    }
}

/// Enhanced LOD update system with hysteresis and change tracking
pub fn update_lod_system(
    lod_config: Res<LodConfig>,
    time: Res<Time>,
    cameras: Query<&Transform, With<Camera>>,
    mut commands: Commands,
    mut lod_groups: Query<(Entity, &mut LodGroup, &Transform), Without<Camera>>,
) {
    if !lod_config.enabled {
        return;
    }

    // Get primary camera position
    let camera_position = cameras
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or_default();

    let delta_time = time.delta_secs();
    let mut updates_this_frame = 0;

    for (entity, mut lod_group, transform) in lod_groups.iter_mut() {
        // Performance limit: only update a certain number per frame
        if updates_this_frame >= lod_config.max_updates_per_frame {
            break;
        }

        let distance = camera_position.distance(transform.translation);
        let adjusted_distance = (distance * lod_config.bias).max(lod_config.min_distance);

        // Update LOD with change tracking
        let changed = lod_group.update_lod(adjusted_distance);

        // Update cross-fade animation
        if lod_config.cross_fade_enabled {
            lod_group.update_cross_fade(delta_time);
        }

        // Add ChangedLod marker for efficient extraction
        if changed {
            commands.entity(entity).insert(ChangedLod {
                from_lod: lod_group.previous_lod,
                to_lod: lod_group.current_lod,
                changed_frame: time.elapsed().as_millis() as u32,
            });
        }

        updates_this_frame += 1;
    }
}

/// System to integrate LOD changes with BatchManager
pub fn lod_batch_integration_system(
    mut commands: Commands,
    changed_lod_query: Query<(Entity, &LodGroup, &ChangedLod), With<ChangedLod>>,
) {
    for (entity, _lod_group, _changed_lod) in changed_lod_query.iter() {
        // LOD change triggers batch migration in BatchManager
        // The BatchManager will handle moving instances between batches
        // This is handled automatically through the ExtractedInstance batch_key changes

        // Remove the ChangedLod marker after processing
        commands.entity(entity).remove::<ChangedLod>();
    }
}

/// System to update extracted instances with LOD information
pub fn lod_extraction_system(
    lod_groups: Query<&LodGroup>,
    mut instances: Query<&mut ExtractedInstance>,
) {
    // Enhanced LOD extraction with batch key updates
    for lod_group in lod_groups.iter() {
        for mut instance in instances.iter_mut() {
            // Update batch key if LOD changed (this triggers BatchManager migration)
            if let Some(current_level) = lod_group.current_level() {
                // Update the batch key to reflect the new LOD mesh/material
                // This automatically triggers BatchManager to move the instance
                let material = current_level.material.clone().unwrap_or_else(|| {
                    // Use existing material from batch key if no LOD override
                    // This is a simplified approach - in practice we'd need to track original materials
                    Handle::default()
                });

                instance.batch_key = BatchKey::new(&current_level.mesh, &material);
            }
        }
    }
}

/// Plugin for enhanced LOD systems
pub struct LodSystemPlugin;

impl Plugin for LodSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodConfig>().add_systems(
            PostUpdate,
            (
                update_lod_system,
                lod_batch_integration_system,
                lod_extraction_system,
            ),
        );
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::lod::{
        ChangedLod, LodConfig, LodGroup, LodLevel, LodSystemPlugin, lod_batch_integration_system,
        lod_extraction_system, update_lod_system,
    };
}
