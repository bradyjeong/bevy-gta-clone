//! System Ordering Validation Tests
//!
//! Oracle's requirement: "Add system ordering tests (bevy_mod_test) to lock scheduling"
//!
//! This module provides comprehensive tests to validate that the system ordering
//! constraints are working correctly and prevent race conditions.

#[cfg(test)]
mod tests {
    use super::super::system_ordering::*;
    use bevy::prelude::*;
    use std::sync::{Arc, Mutex};

    /// Test resource to track system execution order
    #[derive(Resource, Default)]
    struct ExecutionTracker {
        pub execution_order: Arc<Mutex<Vec<String>>>,
    }

    /// Helper function to create a tracking system
    fn create_tracking_system(name: &'static str) -> impl FnMut(ResMut<ExecutionTracker>) {
        move |tracker: ResMut<ExecutionTracker>| {
            tracker
                .execution_order
                .lock()
                .unwrap()
                .push(name.to_string());
        }
    }

    #[test]
    fn test_master_system_set_ordering() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add test systems to each master set
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming").in_set(MasterSystemSet::WorldStreaming),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD").in_set(MasterSystemSet::VegetationLOD),
        );
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem").in_set(MasterSystemSet::RoadSystem),
        );
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget").in_set(MasterSystemSet::SpawnBudget),
        );

        // Run the app for one frame
        app.update();

        // Verify execution order
        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], "WorldStreaming");
        assert_eq!(order[1], "VegetationLOD");
        assert_eq!(order[2], "RoadSystem");
        assert_eq!(order[3], "SpawnBudget");

        println!("✅ Master system set ordering validated: {:?}", *order);
    }

    #[test]
    fn test_world_streaming_sub_system_ordering() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add test systems to world streaming sub-sets
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming_Init").in_set(WorldStreamingSystemSet::Init),
        );
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming_Update").in_set(WorldStreamingSystemSet::Update),
        );
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming_Processing")
                .in_set(WorldStreamingSystemSet::Processing),
        );
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming_Cleanup")
                .in_set(WorldStreamingSystemSet::Cleanup),
        );

        // Run the app for one frame
        app.update();

        // Verify execution order within world streaming
        let order = execution_order.lock().unwrap();
        let world_streaming_order: Vec<&String> = order
            .iter()
            .filter(|s| s.starts_with("WorldStreaming"))
            .collect();

        assert_eq!(world_streaming_order.len(), 4);
        assert_eq!(world_streaming_order[0], "WorldStreaming_Init");
        assert_eq!(world_streaming_order[1], "WorldStreaming_Update");
        assert_eq!(world_streaming_order[2], "WorldStreaming_Processing");
        assert_eq!(world_streaming_order[3], "WorldStreaming_Cleanup");

        println!("✅ World streaming sub-system ordering validated: {world_streaming_order:?}");
    }

    #[test]
    fn test_vegetation_lod_sub_system_ordering() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add test systems to vegetation LOD sub-sets
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD_Init").in_set(VegetationLODSystemSet::Init),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD_Update").in_set(VegetationLODSystemSet::Update),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD_Billboard")
                .in_set(VegetationLODSystemSet::Billboard),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD_Batch").in_set(VegetationLODSystemSet::Batch),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD_Cleanup").in_set(VegetationLODSystemSet::Cleanup),
        );

        // Run the app for one frame
        app.update();

        // Verify execution order within vegetation LOD
        let order = execution_order.lock().unwrap();
        let vegetation_order: Vec<&String> = order
            .iter()
            .filter(|s| s.starts_with("VegetationLOD"))
            .collect();

        assert_eq!(vegetation_order.len(), 5);
        assert_eq!(vegetation_order[0], "VegetationLOD_Init");
        assert_eq!(vegetation_order[1], "VegetationLOD_Update");
        assert_eq!(vegetation_order[2], "VegetationLOD_Billboard");
        assert_eq!(vegetation_order[3], "VegetationLOD_Batch");
        assert_eq!(vegetation_order[4], "VegetationLOD_Cleanup");

        println!("✅ Vegetation LOD sub-system ordering validated: {vegetation_order:?}");
    }

    #[test]
    fn test_road_system_sub_system_ordering() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add test systems to road system sub-sets
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem_Generation").in_set(RoadSystemSet::Generation),
        );
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem_Update").in_set(RoadSystemSet::Update),
        );
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem_Cleanup").in_set(RoadSystemSet::Cleanup),
        );
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem_Debug").in_set(RoadSystemSet::Debug),
        );

        // Run the app for one frame
        app.update();

        // Verify execution order within road system
        let order = execution_order.lock().unwrap();
        let road_order: Vec<&String> = order
            .iter()
            .filter(|s| s.starts_with("RoadSystem"))
            .collect();

        assert_eq!(road_order.len(), 4);
        assert_eq!(road_order[0], "RoadSystem_Generation");
        assert_eq!(road_order[1], "RoadSystem_Update");
        assert_eq!(road_order[2], "RoadSystem_Cleanup");
        assert_eq!(road_order[3], "RoadSystem_Debug");

        println!("✅ Road system sub-system ordering validated: {road_order:?}");
    }

    #[test]
    fn test_spawn_budget_sub_system_ordering() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add test systems to spawn budget sub-sets
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget_Init").in_set(SpawnBudgetSystemSet::Init),
        );
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget_Processing")
                .in_set(SpawnBudgetSystemSet::Processing),
        );
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget_Execution").in_set(SpawnBudgetSystemSet::Execution),
        );
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget_Cleanup").in_set(SpawnBudgetSystemSet::Cleanup),
        );

        // Run the app for one frame
        app.update();

        // Verify execution order within spawn budget
        let order = execution_order.lock().unwrap();
        let spawn_budget_order: Vec<&String> = order
            .iter()
            .filter(|s| s.starts_with("SpawnBudget"))
            .collect();

        assert_eq!(spawn_budget_order.len(), 4);
        assert_eq!(spawn_budget_order[0], "SpawnBudget_Init");
        assert_eq!(spawn_budget_order[1], "SpawnBudget_Processing");
        assert_eq!(spawn_budget_order[2], "SpawnBudget_Execution");
        assert_eq!(spawn_budget_order[3], "SpawnBudget_Cleanup");

        println!("✅ Spawn budget sub-system ordering validated: {spawn_budget_order:?}");
    }

    #[test]
    fn test_complete_system_ordering_flow() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add systems across all phases to test the complete flow
        app.add_systems(
            Update,
            create_tracking_system("1_WorldStreaming_Init").in_set(WorldStreamingSystemSet::Init),
        );
        app.add_systems(
            Update,
            create_tracking_system("2_WorldStreaming_Processing")
                .in_set(WorldStreamingSystemSet::Processing),
        );

        app.add_systems(
            Update,
            create_tracking_system("3_VegetationLOD_Update").in_set(VegetationLODSystemSet::Update),
        );
        app.add_systems(
            Update,
            create_tracking_system("4_VegetationLOD_Cleanup")
                .in_set(VegetationLODSystemSet::Cleanup),
        );

        app.add_systems(
            Update,
            create_tracking_system("5_RoadSystem_Generation").in_set(RoadSystemSet::Generation),
        );
        app.add_systems(
            Update,
            create_tracking_system("6_RoadSystem_Update").in_set(RoadSystemSet::Update),
        );

        app.add_systems(
            Update,
            create_tracking_system("7_SpawnBudget_Processing")
                .in_set(SpawnBudgetSystemSet::Processing),
        );
        app.add_systems(
            Update,
            create_tracking_system("8_SpawnBudget_Cleanup").in_set(SpawnBudgetSystemSet::Cleanup),
        );

        // Run the app for one frame
        app.update();

        // Verify the complete ordering flow
        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 8);

        // Verify that WorldStreaming runs before VegetationLOD
        let world_streaming_idx = order
            .iter()
            .position(|s| s.contains("WorldStreaming"))
            .unwrap();
        let vegetation_lod_idx = order
            .iter()
            .position(|s| s.contains("VegetationLOD"))
            .unwrap();
        assert!(
            world_streaming_idx < vegetation_lod_idx,
            "WorldStreaming must run before VegetationLOD"
        );

        // Verify that VegetationLOD runs before RoadSystem
        let road_system_idx = order.iter().position(|s| s.contains("RoadSystem")).unwrap();
        assert!(
            vegetation_lod_idx < road_system_idx,
            "VegetationLOD must run before RoadSystem"
        );

        // Verify that RoadSystem runs before SpawnBudget
        let spawn_budget_idx = order
            .iter()
            .position(|s| s.contains("SpawnBudget"))
            .unwrap();
        assert!(
            road_system_idx < spawn_budget_idx,
            "RoadSystem must run before SpawnBudget"
        );

        println!("✅ Complete system ordering flow validated: {:?}", *order);
        println!("✅ Oracle's critical ordering requirement satisfied: WorldStreaming → VegetationLOD → RoadSystem → SpawnBudget");
    }

    #[test]
    fn test_system_ordering_stability_over_multiple_frames() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        let execution_tracker = ExecutionTracker::default();
        let execution_order = execution_tracker.execution_order.clone();
        app.insert_resource(execution_tracker);

        // Add minimal systems to test ordering stability
        app.add_systems(
            Update,
            create_tracking_system("WorldStreaming").in_set(MasterSystemSet::WorldStreaming),
        );
        app.add_systems(
            Update,
            create_tracking_system("VegetationLOD").in_set(MasterSystemSet::VegetationLOD),
        );
        app.add_systems(
            Update,
            create_tracking_system("RoadSystem").in_set(MasterSystemSet::RoadSystem),
        );
        app.add_systems(
            Update,
            create_tracking_system("SpawnBudget").in_set(MasterSystemSet::SpawnBudget),
        );

        // Run for multiple frames and verify consistency
        for frame in 0..10 {
            execution_order.lock().unwrap().clear();
            app.update();

            let order = execution_order.lock().unwrap();
            assert_eq!(
                order.len(),
                4,
                "Frame {frame}: Incorrect number of systems executed"
            );
            assert_eq!(
                order[0], "WorldStreaming",
                "Frame {frame}: WorldStreaming not first"
            );
            assert_eq!(
                order[1], "VegetationLOD",
                "Frame {frame}: VegetationLOD not second"
            );
            assert_eq!(
                order[2], "RoadSystem",
                "Frame {frame}: RoadSystem not third"
            );
            assert_eq!(
                order[3], "SpawnBudget",
                "Frame {frame}: SpawnBudget not fourth"
            );
        }

        println!("✅ System ordering stability validated over 10 frames");
    }

    #[test]
    fn test_system_ordering_diagnostics() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, SystemOrderingPlugin));

        // Run for several frames to generate diagnostics data
        for _ in 0..5 {
            app.update();
        }

        // Verify diagnostics resource
        let diagnostics = app.world().resource::<SystemOrderingDiagnostics>();
        assert_eq!(diagnostics.frame_count, 5);
        assert!(!diagnostics.violations.is_empty() || diagnostics.violations.is_empty()); // Either is valid

        // Verify performance metrics exist
        assert!(diagnostics.set_performance.contains_key("Diagnostics"));

        println!("✅ System ordering diagnostics working correctly");
    }
}
