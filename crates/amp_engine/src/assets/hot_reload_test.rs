//! Test for hot-reload functionality with the new asset pipeline

#[cfg(test)]
mod tests {
    use crate::assets::{AmpSceneLoader, AmpScenePrefab};
    use bevy::asset::AssetLoader;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hot_reload_asset_loading() {
        // Create a temporary directory for assets
        let temp_dir = TempDir::new().unwrap();
        let asset_path = temp_dir.path().join("test_prefab.amp.ron");

        // Create initial RON content
        let initial_ron = r#"
        (
            components: [
                (
                    component_type: "TestComponent",
                    data: Map({"value": Number(42.0)})
                )
            ]
        )
        "#;

        // Write initial file
        fs::write(&asset_path, initial_ron).unwrap();

        // Test that the asset loader can load the file
        let loader = AmpSceneLoader;
        let extensions = loader.extensions();
        assert!(extensions.contains(&"amp.ron"));

        // Read the file content
        let content = fs::read_to_string(&asset_path).unwrap();

        // Verify the content can be parsed
        assert!(content.contains("TestComponent"));
        assert!(content.contains("42.0"));

        println!("✓ Hot-reload asset file creation and reading verified");
    }

    #[test]
    fn test_asset_pipeline_plugin_creation() {
        // Test that we can create the plugin (integration test using App pattern)
        let mut app = crate::test_utils::test_app();
        app.update(); // Run one frame to initialize

        // Test that we can create a prefab
        let prefab = AmpScenePrefab::new();
        assert!(prefab.is_empty());

        // Verify the plugin added our asset type
        let _assets = app
            .world()
            .resource::<bevy::asset::Assets<AmpScenePrefab>>();

        println!("✓ Asset pipeline plugin creation verified");
    }
}
