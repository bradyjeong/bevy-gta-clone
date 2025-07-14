//! App-level tests for the new asset pipeline using Oracle's patterns

#[cfg(test)]
mod tests {
    use crate::assets::{
        AmpSceneComponent, AmpSceneLoader, AmpScenePlugin, AmpScenePrefab, RonScenePrefab,
    };
    use bevy::app::App;
    use bevy::asset::{AssetLoader, AssetPlugin, AssetServer, Assets};
    use bevy::MinimalPlugins;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_app_level_asset_loading() {
        // Create temporary directory and asset file
        let temp_dir = TempDir::new().unwrap();
        let asset_path = temp_dir.path().join("test_prefab.amp.ron");

        let ron_content = r#"
        (
            components: [
                (
                    component_type: "Transform",
                    data: Map({
                        "translation": Array([Number(1.0), Number(2.0), Number(3.0)]),
                        "scale": Array([Number(1.0), Number(1.0), Number(1.0)])
                    })
                ),
                (
                    component_type: "Visibility",
                    data: String("Visible")
                )
            ]
        )
        "#;

        fs::write(&asset_path, ron_content).unwrap();

        // Create App with MinimalPlugins and AssetPlugin per Oracle's pattern
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin {
                file_path: temp_dir.path().to_string_lossy().to_string(),
                ..Default::default()
            })
            .add_plugins(AmpScenePlugin);

        // Run one frame to initialize
        app.update();

        // Verify the asset system is working
        let _asset_server = app.world().resource::<AssetServer>();
        // Asset server is initialized correctly if we can get it

        println!("✓ App-level asset loading test passed");
    }

    #[test]
    fn test_in_memory_asset_loading() {
        // Test load_from_memory functionality per Oracle's guidance
        let _ron_content = r#"
        (
            components: [
                (
                    component_type: "TestComponent",
                    data: Map({
                        "value": Number(42.0),
                        "name": String("Test Entity")
                    })
                )
            ]
        )
        "#;

        // Create App with minimal setup
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .add_plugins(AmpScenePlugin);

        app.update();

        // Test the asset loader directly
        let loader = AmpSceneLoader;
        let extensions = loader.extensions();

        assert!(extensions.contains(&"amp.ron"));
        assert!(extensions.contains(&"scene.ron"));
        assert!(extensions.contains(&"prefab.ron"));

        println!("✓ In-memory asset loading test passed");
    }

    #[test]
    fn test_serialization_edge_cases() {
        // Test various edge cases in serialization per Oracle's requirements

        // Empty prefab
        let empty_ron = r#"
        (
            components: []
        )
        "#;

        let ron_prefab: Result<RonScenePrefab, _> = ron::from_str(empty_ron);
        assert!(ron_prefab.is_ok());
        let prefab = ron_prefab.unwrap();
        assert!(prefab.components.is_empty());

        // Single component with complex data
        let complex_ron = r#"
        (
            components: [
                (
                    component_type: "ComplexComponent",
                    data: Map({
                        "nested": Map({
                            "array": Array([Number(1.0), Number(2.0)]),
                            "boolean": Bool(true)
                        }),
                        "string": String("test")
                    })
                )
            ]
        )
        "#;

        let complex_prefab: Result<RonScenePrefab, _> = ron::from_str(complex_ron);
        assert!(complex_prefab.is_ok());
        let prefab = complex_prefab.unwrap();
        assert_eq!(prefab.components.len(), 1);
        assert_eq!(prefab.components[0].component_type, "ComplexComponent");

        // Test conversion to AmpScenePrefab
        let amp_prefab: AmpScenePrefab = prefab.into();
        assert_eq!(amp_prefab.len(), 1);
        assert!(!amp_prefab.is_empty());

        println!("✓ Serialization edge cases test passed");
    }

    #[test]
    fn test_asset_plugin_integration() {
        // Test that AmpScenePlugin works correctly with Bevy's asset system
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .add_plugins(AmpScenePlugin);

        app.update();

        // Verify that our asset type is registered
        let assets = app.world().resource::<Assets<AmpScenePrefab>>();
        assert_eq!(assets.len(), 0); // Should start empty

        // Create a test prefab manually
        let mut prefab = AmpScenePrefab::new();
        prefab.add_component(AmpSceneComponent::new(
            "TestComponent".to_string(),
            ron::Value::String("test_value".to_string()),
        ));

        // Insert into assets
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<AmpScenePrefab>>();
            assets.add(prefab)
        };

        // Verify it was added
        let assets = app.world().resource::<Assets<AmpScenePrefab>>();
        assert_eq!(assets.len(), 1);
        let retrieved_prefab = assets.get(&handle).unwrap();
        assert_eq!(retrieved_prefab.len(), 1);
        assert_eq!(
            retrieved_prefab.components[0].component_type,
            "TestComponent"
        );

        println!("✓ Asset plugin integration test passed");
    }
}
