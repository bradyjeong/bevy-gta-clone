//! Road mesh attachment system for attaching generated meshes to entities
//!
//! This module implements the mesh attachment system as specified by Oracle's
//! strategic plan to complete the road mesh rendering pipeline.

use super::events::RoadMeshReady;
use super::materials::{MarkingType, RoadMaterialLibrary, RoadSurfaceType};
use bevy::prelude::*;

/// System to attach road meshes to entities when mesh generation is complete
pub fn attach_road_meshes(
    mut commands: Commands,
    mut mesh_ready_events: EventReader<RoadMeshReady>,
    mut meshes: ResMut<Assets<Mesh>>,
    road_materials: Res<RoadMaterialLibrary>,
    mut entity_query: Query<(Entity, Option<&mut Transform>)>,
) {
    for mesh_ready in mesh_ready_events.read() {
        if let Some(error) = &mesh_ready.error {
            warn!(
                "Skipping mesh attachment for road_id {} due to generation error: {}",
                mesh_ready.road_id, error
            );
            continue;
        }

        // Check if the target entity still exists
        if let Ok((entity, transform)) = entity_query.get_mut(mesh_ready.road_entity) {
            debug!(
                "Attaching road mesh to entity {:?} for road_id {}",
                entity, mesh_ready.road_id
            );

            // Convert MeshData to Bevy Mesh
            let main_mesh_handle = meshes.add(mesh_ready.main_mesh.clone().to_mesh());

            // Get appropriate road surface material
            let road_material = road_materials.get_surface_material(RoadSurfaceType::Asphalt);

            // Attach main road mesh directly to the road entity (as per Oracle's requirements)
            commands
                .entity(entity)
                .insert((Mesh3d(main_mesh_handle), MeshMaterial3d(road_material)));

            // Spawn lane markings as child entities
            for (marking_index, marking_mesh_data) in mesh_ready.marking_meshes.iter().enumerate() {
                let marking_mesh_handle = meshes.add(marking_mesh_data.clone().to_mesh());
                let marking_material = road_materials.get_marking_material(MarkingType::WhiteLine);

                // Create child entity for each lane marking
                let marking_entity = commands
                    .spawn((
                        Mesh3d(marking_mesh_handle),
                        MeshMaterial3d(marking_material),
                        Transform::from_translation(Vec3::new(0.0, 0.001, 0.0)), // Slightly above road surface
                        Name::new(format!(
                            "RoadMarking_{}_Line_{}",
                            mesh_ready.road_id, marking_index
                        )),
                    ))
                    .id();

                // Set as child of the main road entity
                commands.entity(entity).add_child(marking_entity);
            }

            info!(
                "Successfully attached road mesh and {} lane markings to entity {:?} for road_id {} (generation time: {:.2}ms)",
                mesh_ready.marking_meshes.len(),
                entity,
                mesh_ready.road_id,
                mesh_ready.generation_time * 1000.0
            );
        } else {
            warn!(
                "Failed to attach road mesh: entity {:?} for road_id {} no longer exists",
                mesh_ready.road_entity, mesh_ready.road_id
            );
        }
    }
}

/// System to handle cleanup of road meshes when entities are despawned
/// Note: This is a simplified version - a complete implementation would need
/// to track which entities had meshes before they were despawned
pub fn cleanup_despawned_road_meshes(_commands: Commands, _meshes: ResMut<Assets<Mesh>>) {
    // In a complete implementation, we would track entities with meshes
    // and clean them up when they're despawned. For now, this is a placeholder.
    // The actual cleanup would happen through Bevy's built-in asset management.
}

/// Resource to track mesh attachment statistics
#[derive(Resource, Default, Debug)]
pub struct MeshAttachmentStats {
    /// Total successful mesh attachments
    pub successful_attachments: u64,
    /// Total failed attachments (entity not found)
    pub failed_attachments: u64,
    /// Total lane markings attached
    pub lane_markings_attached: u64,
    /// Total meshes cleaned up
    pub meshes_cleaned_up: u64,
    /// Average attachment time (seconds)
    pub average_attachment_time: f32,
    /// Total attachment time for averaging
    total_attachment_time: f32,
}

impl MeshAttachmentStats {
    /// Record a successful mesh attachment
    pub fn record_successful_attachment(&mut self, attachment_time: f32, marking_count: usize) {
        self.successful_attachments += 1;
        self.lane_markings_attached += marking_count as u64;
        self.total_attachment_time += attachment_time;
        self.average_attachment_time =
            self.total_attachment_time / self.successful_attachments as f32;
    }

    /// Record a failed mesh attachment
    pub fn record_failed_attachment(&mut self) {
        self.failed_attachments += 1;
    }

    /// Record mesh cleanup
    pub fn record_mesh_cleanup(&mut self) {
        self.meshes_cleaned_up += 1;
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        let total_attempts = self.successful_attachments + self.failed_attachments;
        if total_attempts == 0 {
            100.0
        } else {
            (self.successful_attachments as f32 / total_attempts as f32) * 100.0
        }
    }
}

/// Enhanced mesh attachment system with statistics tracking
pub fn attach_road_meshes_with_stats(
    mut commands: Commands,
    mut mesh_ready_events: EventReader<RoadMeshReady>,
    mut meshes: ResMut<Assets<Mesh>>,
    road_materials: Res<RoadMaterialLibrary>,
    mut entity_query: Query<(Entity, Option<&mut Transform>)>,
    mut stats: ResMut<MeshAttachmentStats>,
) {
    for mesh_ready in mesh_ready_events.read() {
        let start_time = std::time::Instant::now();

        if let Some(error) = &mesh_ready.error {
            warn!(
                "Skipping mesh attachment for road_id {} due to generation error: {}",
                mesh_ready.road_id, error
            );
            stats.record_failed_attachment();
            continue;
        }

        // Check if the target entity still exists
        if let Ok((entity, _transform)) = entity_query.get_mut(mesh_ready.road_entity) {
            // Convert MeshData to Bevy Mesh
            let main_mesh_handle = meshes.add(mesh_ready.main_mesh.clone().to_mesh());

            // Get appropriate road surface material
            let road_material = road_materials.get_surface_material(RoadSurfaceType::Asphalt);

            // Attach main road mesh directly to the road entity
            commands
                .entity(entity)
                .insert((Mesh3d(main_mesh_handle), MeshMaterial3d(road_material)));

            // Spawn lane markings as child entities
            for (marking_index, marking_mesh_data) in mesh_ready.marking_meshes.iter().enumerate() {
                let marking_mesh_handle = meshes.add(marking_mesh_data.clone().to_mesh());
                let marking_material = road_materials.get_marking_material(MarkingType::WhiteLine);

                let marking_entity = commands
                    .spawn((
                        Mesh3d(marking_mesh_handle),
                        MeshMaterial3d(marking_material),
                        Transform::from_translation(Vec3::new(0.0, 0.001, 0.0)),
                        Name::new(format!(
                            "RoadMarking_{}_Line_{}",
                            mesh_ready.road_id, marking_index
                        )),
                    ))
                    .id();

                commands.entity(entity).add_child(marking_entity);
            }

            let attachment_time = start_time.elapsed().as_secs_f32();
            stats.record_successful_attachment(attachment_time, mesh_ready.marking_meshes.len());

            debug!(
                "Successfully attached road mesh and {} lane markings to entity {:?} for road_id {} (attachment time: {:.2}ms)",
                mesh_ready.marking_meshes.len(),
                entity,
                mesh_ready.road_id,
                attachment_time * 1000.0
            );
        } else {
            stats.record_failed_attachment();
            warn!(
                "Failed to attach road mesh: entity {:?} for road_id {} no longer exists",
                mesh_ready.road_entity, mesh_ready.road_id
            );
        }
    }
}

/// Debug system for mesh attachment statistics
pub fn debug_mesh_attachment_stats(stats: Res<MeshAttachmentStats>, time: Res<Time>) {
    // Only log stats every 5 seconds to avoid spam
    if (time.elapsed_secs() % 5.0) < time.delta_secs() {
        if stats.successful_attachments > 0 || stats.failed_attachments > 0 {
            info!(
                "🔗 MESH ATTACHMENT STATS:\n\
                📊 Success/Failure:\n\
                • Successful Attachments: {} roads\n\
                • Failed Attachments: {} roads\n\
                • Success Rate: {:.1}%\n\
                🎨 Rendering Components:\n\
                • Lane Markings Attached: {} markings\n\
                • Average Attachment Time: {:.2}ms\n\
                🧹 Cleanup:\n\
                • Meshes Cleaned Up: {} meshes",
                stats.successful_attachments,
                stats.failed_attachments,
                stats.success_rate(),
                stats.lane_markings_attached,
                stats.average_attachment_time * 1000.0,
                stats.meshes_cleaned_up
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_attachment_stats_creation() {
        let stats = MeshAttachmentStats::default();

        assert_eq!(stats.successful_attachments, 0);
        assert_eq!(stats.failed_attachments, 0);
        assert_eq!(stats.lane_markings_attached, 0);
        assert_eq!(stats.success_rate(), 100.0);
    }

    #[test]
    fn test_mesh_attachment_stats_recording() {
        let mut stats = MeshAttachmentStats::default();

        // Record successful attachment
        stats.record_successful_attachment(0.005, 3);

        assert_eq!(stats.successful_attachments, 1);
        assert_eq!(stats.lane_markings_attached, 3);
        assert_eq!(stats.average_attachment_time, 0.005);
        assert_eq!(stats.success_rate(), 100.0);

        // Record failed attachment
        stats.record_failed_attachment();

        assert_eq!(stats.failed_attachments, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_mesh_attachment_stats_cleanup() {
        let mut stats = MeshAttachmentStats::default();

        stats.record_mesh_cleanup();
        stats.record_mesh_cleanup();

        assert_eq!(stats.meshes_cleaned_up, 2);
    }

    #[test]
    fn test_mesh_attachment_stats_success_rate() {
        let mut stats = MeshAttachmentStats::default();

        // No attempts = 100% success rate
        assert_eq!(stats.success_rate(), 100.0);

        // 3 successes, 1 failure = 75% success rate
        stats.record_successful_attachment(0.001, 0);
        stats.record_successful_attachment(0.002, 0);
        stats.record_successful_attachment(0.003, 0);
        stats.record_failed_attachment();

        assert_eq!(stats.success_rate(), 75.0);
    }
}
