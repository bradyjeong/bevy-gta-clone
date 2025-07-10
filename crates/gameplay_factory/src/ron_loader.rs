//! RON (Rusty Object Notation) loader for prefab definitions

use crate::{ComponentInit, Error, Prefab, PrefabSource};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// RON-based prefab loader
///
/// # Deprecation Notice
///
/// This loader is deprecated in favor of the new Bevy asset pipeline integration.
/// Use `amp_engine::assets::AmpSceneLoader` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use amp_engine::assets::AmpSceneLoader with Bevy's asset pipeline instead"
)]
#[cfg(feature = "legacy_ron_loader")]
#[derive(Debug)]
pub struct RonLoader {
    /// RON string content
    content: String,
}

#[cfg(feature = "legacy_ron_loader")]
#[allow(deprecated)]
impl RonLoader {
    /// Create a new RON loader from string content
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Create a new RON loader from a file path
    ///
    /// # Future Improvements
    ///
    /// TODO: Add async version `from_file_async()` for asset pipeline integration
    /// TODO: Consider streaming large RON files instead of loading entirely into memory
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path).map_err(|io_err| {
            // Preserve the original io::Error in the error chain
            Error::resource_load(path, format!("Failed to read RON file: {io_err}"))
        })?;
        Ok(Self::new(content))
    }
}

#[cfg(feature = "legacy_ron_loader")]
#[allow(deprecated)]
impl PrefabSource for RonLoader {
    fn load(&self) -> Result<Prefab, Error> {
        let ron_prefab: RonPrefab = ron::from_str(&self.content)
            .map_err(|e| Error::serialization(format!("Failed to parse RON: {e}")))?;

        Ok(ron_prefab.into())
    }
}

/// RON-serializable prefab definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonPrefab {
    /// Component definitions
    pub components: Vec<RonComponent>,
}

impl From<RonPrefab> for Prefab {
    fn from(ron_prefab: RonPrefab) -> Self {
        let mut prefab = Prefab::new();
        for component in ron_prefab.components {
            prefab.add_component(Box::new(component));
        }
        prefab
    }
}

/// RON-serializable component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonComponent {
    /// Component type name
    pub component_type: String,
    /// Component data as RON value
    pub data: ron::Value,
}

impl ComponentInit for RonComponent {
    fn init(&self, cmd: &mut Commands, entity: Entity) -> Result<(), Error> {
        // Use the component registry to deserialize and insert the component
        match crate::call_component_deserializer(&self.component_type, &self.data, cmd, entity) {
            Ok(()) => Ok(()),
            Err(e) => {
                // If the component is not found, provide a more helpful error message
                let available_types = crate::registered_components();
                Err(Error::validation(format!(
                    "Component type '{}' not found in registry. Available types: {}. Original error: {}",
                    self.component_type,
                    available_types.join(", "),
                    e
                )))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use rstest::*;
    use serial_test::serial;

    #[rstest]
    fn test_ron_loader_basic() {
        let ron_content = r#"
        RonPrefab(
            components: [
                RonComponent(
                    component_type: "Transform",
                    data: Map({"x": Number(1.0), "y": Number(2.0), "z": Number(3.0)})
                ),
                RonComponent(
                    component_type: "Health",
                    data: Number(100.0)
                )
            ]
        )
        "#;

        let loader = RonLoader::new(ron_content.to_string());
        let prefab = loader.load().unwrap();

        assert_eq!(prefab.len(), 2);
        assert!(!prefab.is_empty());
    }

    #[rstest]
    fn test_ron_loader_empty() {
        let ron_content = r#"
        RonPrefab(
            components: []
        )
        "#;

        let loader = RonLoader::new(ron_content.to_string());
        let prefab = loader.load().unwrap();

        assert_eq!(prefab.len(), 0);
        assert!(prefab.is_empty());
    }

    #[rstest]
    fn test_ron_loader_invalid() {
        let ron_content = "invalid ron content";

        let loader = RonLoader::new(ron_content.to_string());
        let result = loader.load();

        assert!(result.is_err());
    }

    #[rstest]
    fn test_ron_loader_from_file_success() {
        use std::fs;

        let ron_content = r#"
        RonPrefab(
            components: [
                RonComponent(
                    component_type: "TestComponent",
                    data: Number(42.0)
                )
            ]
        )
        "#;

        let temp_path = "/tmp/test_prefab.ron";
        fs::write(temp_path, ron_content).unwrap();

        let loader = RonLoader::from_file(temp_path).unwrap();
        let prefab = loader.load().unwrap();

        assert_eq!(prefab.len(), 1);

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[rstest]
    fn test_ron_loader_from_file_not_found() {
        let result = RonLoader::from_file("/nonexistent/file.ron");
        assert!(result.is_err());
    }

    #[rstest]
    fn test_ron_loader_from_file_preserves_error_context() {
        // Test that error context is preserved from the original io::Error
        let result = RonLoader::from_file("/nonexistent/directory/file.ron");

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_string = error.to_string();

        // Verify the error message contains both the resource path and the original error
        assert!(error_string.contains("/nonexistent/directory/file.ron"));
        assert!(error_string.contains("Failed to read RON file"));
        // The original io::Error should be preserved in the error chain
        assert!(
            error_string.contains("No such file or directory")
                || error_string.contains("cannot find the path")
                || error_string.contains("system cannot find")
        );
    }

    #[rstest]
    fn test_ron_loader_from_file_permission_denied() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp_path = "/tmp/test_readonly.ron";
        fs::write(temp_path, "test content").unwrap();

        // Make file unreadable (Unix-specific test)
        if let Ok(mut perms) = fs::metadata(temp_path).map(|m| m.permissions()) {
            perms.set_mode(0o000);
            if fs::set_permissions(temp_path, perms).is_ok() {
                let result = RonLoader::from_file(temp_path);

                assert!(result.is_err());
                let error = result.unwrap_err();
                let error_string = error.to_string();

                // Verify the error message contains both the resource path and permission info
                assert!(error_string.contains(temp_path));
                assert!(error_string.contains("Failed to read RON file"));
                // Should contain some indication of permission error
                assert!(
                    error_string.contains("Permission denied") || error_string.contains("denied")
                );
            }
        }

        // Cleanup
        fs::remove_file(temp_path).ok();
    }

    #[rstest]
    #[serial]
    fn test_ron_component_init() {
        crate::component_registry::clear_registry();
        // Register a test component
        let _ = crate::register_component(
            "TestComponent",
            Box::new(|_value, _cmd, _entity| {
                // Test component that just succeeds
                Ok(())
            }),
        );

        let component = RonComponent {
            component_type: "TestComponent".to_string(),
            data: ron::Value::Number(ron::Number::new(42.0)),
        };

        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmd = Commands::new(&mut queue, &world);
        let entity = cmd.spawn_empty().id();

        let result = component.init(&mut cmd, entity);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_ron_component_as_any() {
        let component = RonComponent {
            component_type: "TestComponent".to_string(),
            data: ron::Value::Number(ron::Number::new(42.0)),
        };

        let any_ref = component.as_any();
        let downcasted = any_ref.downcast_ref::<RonComponent>();
        assert!(downcasted.is_some());
        assert_eq!(downcasted.unwrap().component_type, "TestComponent");
    }
}
