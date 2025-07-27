//! Tests for vegetation LOD system

#[cfg(test)]
mod tests {
    use crate::vegetation::*;
    use bevy::asset::Handle;
    use bevy::prelude::*;
    use std::any::TypeId;

    #[test]
    fn test_vegetation_detail_level_default() {
        let level = VegetationDetailLevel::default();
        assert_eq!(level, VegetationDetailLevel::Full);
    }

    #[test]
    fn test_vegetation_lod_new() {
        let lod = VegetationLOD::new();
        assert_eq!(lod.detail_level, VegetationDetailLevel::Full);
        assert_eq!(lod.distance_to_player, 0.0);
        assert_eq!(lod.last_update_frame, 0);
    }

    #[test]
    fn test_vegetation_lod_from_distance() {
        // Test distance thresholds for each LOD level
        let close_lod = VegetationLOD::from_distance(30.0);
        assert_eq!(close_lod.detail_level, VegetationDetailLevel::Full);
        assert_eq!(close_lod.distance_to_player, 30.0);

        let medium_lod = VegetationLOD::from_distance(100.0);
        assert_eq!(medium_lod.detail_level, VegetationDetailLevel::Medium);
        assert_eq!(medium_lod.distance_to_player, 100.0);

        let billboard_lod = VegetationLOD::from_distance(200.0);
        assert_eq!(billboard_lod.detail_level, VegetationDetailLevel::Billboard);
        assert_eq!(billboard_lod.distance_to_player, 200.0);

        let culled_lod = VegetationLOD::from_distance(400.0);
        assert_eq!(culled_lod.detail_level, VegetationDetailLevel::Culled);
        assert_eq!(culled_lod.distance_to_player, 400.0);
    }

    #[test]
    fn test_vegetation_lod_update_from_distance() {
        let mut lod = VegetationLOD::new();

        lod.update_from_distance(75.0, 42);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Medium);
        assert_eq!(lod.distance_to_player, 75.0);
        assert_eq!(lod.last_update_frame, 42);
    }

    #[test]
    fn test_vegetation_lod_should_be_visible() {
        let mut lod = VegetationLOD::new();

        // Test visible levels
        lod.update_from_distance(30.0, 0);
        assert!(lod.should_be_visible());

        lod.update_from_distance(100.0, 0);
        assert!(lod.should_be_visible());

        lod.update_from_distance(200.0, 0);
        assert!(lod.should_be_visible());

        // Test culled level
        lod.update_from_distance(400.0, 0);
        assert!(!lod.should_be_visible());
    }

    #[test]
    fn test_vegetation_mesh_lod_get_mesh_for_level() {
        let full_mesh = Handle::<Mesh>::default();
        let medium_mesh = Handle::<Mesh>::default();
        let billboard_mesh = Handle::<Mesh>::default();

        let mesh_lod = VegetationMeshLOD::new(
            full_mesh.clone(),
            medium_mesh.clone(),
            billboard_mesh.clone(),
        );

        assert_eq!(
            mesh_lod.get_mesh_for_level(VegetationDetailLevel::Full),
            Some(full_mesh)
        );
        assert_eq!(
            mesh_lod.get_mesh_for_level(VegetationDetailLevel::Medium),
            Some(medium_mesh)
        );
        assert_eq!(
            mesh_lod.get_mesh_for_level(VegetationDetailLevel::Billboard),
            Some(billboard_mesh)
        );
        assert_eq!(
            mesh_lod.get_mesh_for_level(VegetationDetailLevel::Culled),
            None
        );
    }

    #[test]
    fn test_vegetation_billboard_new() {
        let texture = Handle::<Image>::default();
        let size = Vec2::new(2.0, 3.0);

        let billboard = VegetationBillboard::new(texture.clone(), size);
        assert_eq!(billboard.texture, texture);
        assert_eq!(billboard.size, size);
        assert!(billboard.always_face_camera);
        assert_eq!(billboard.original_scale, Vec3::ONE);
    }

    #[test]
    fn test_vegetation_lod_stats_default() {
        let stats = VegetationLODStats::default();
        assert_eq!(stats.full_count, 0);
        assert_eq!(stats.medium_count, 0);
        assert_eq!(stats.billboard_count, 0);
        assert_eq!(stats.culled_count, 0);
        assert_eq!(stats.total_entities, 0);
    }

    // Integration test with Bevy app
    #[test]
    fn test_vegetation_lod_plugin_integration() {
        let mut app = App::new();

        // Add minimal plugins required for testing
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());

        // Add our vegetation LOD plugin
        app.add_plugins(VegetationLODPlugin);

        // Verify resources are registered
        assert!(app.world().contains_resource::<VegetationLODStats>());

        // Verify types are registered for reflection
        let type_registry = app.world().resource::<AppTypeRegistry>();
        let registry = type_registry.read();

        assert!(registry.contains(TypeId::of::<VegetationDetailLevel>()));
        assert!(registry.contains(TypeId::of::<VegetationLOD>()));
        assert!(registry.contains(TypeId::of::<VegetationMeshLOD>()));
        assert!(registry.contains(TypeId::of::<VegetationBillboard>()));
    }

    #[test]
    fn test_vegetation_lod_distance_thresholds() {
        // Test exact boundary conditions
        let mut lod = VegetationLOD::new();

        // Test boundaries between levels
        lod.update_from_distance(49.9, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Full);

        lod.update_from_distance(50.0, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Medium);

        lod.update_from_distance(149.9, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Medium);

        lod.update_from_distance(150.0, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Billboard);

        lod.update_from_distance(299.9, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Billboard);

        lod.update_from_distance(300.0, 0);
        assert_eq!(lod.detail_level, VegetationDetailLevel::Culled);
    }
}
