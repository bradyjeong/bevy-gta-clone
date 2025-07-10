//! Tests for the asset pipeline

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::assets::{
        AmpSceneComponent, AmpSceneLoader, AmpScenePrefab, RonSceneComponent, RonScenePrefab,
    };
    use bevy::asset::AssetLoader;

    #[test]
    fn test_ron_scene_prefab_creation() {
        let component = RonSceneComponent {
            component_type: "TestComponent".to_string(),
            data: ron::Value::Number(ron::Number::new(42.0)),
        };

        let prefab = RonScenePrefab {
            components: vec![component],
        };

        assert_eq!(prefab.components.len(), 1);
        assert_eq!(prefab.components[0].component_type, "TestComponent");
    }

    #[test]
    fn test_ron_scene_prefab_conversion() {
        let component = RonSceneComponent {
            component_type: "TestComponent".to_string(),
            data: ron::Value::Number(ron::Number::new(42.0)),
        };

        let ron_prefab = RonScenePrefab {
            components: vec![component],
        };

        let amp_prefab: AmpScenePrefab = ron_prefab.into();

        assert_eq!(amp_prefab.len(), 1);
        assert!(!amp_prefab.is_empty());
        assert_eq!(amp_prefab.components[0].component_type, "TestComponent");
    }

    #[test]
    fn test_scene_component_creation() {
        let component = AmpSceneComponent::new(
            "TestComponent".to_string(),
            ron::Value::Number(ron::Number::new(42.0)),
        );

        assert_eq!(component.component_type, "TestComponent");
    }

    #[test]
    fn test_loader_extensions() {
        let loader = AmpSceneLoader;
        let extensions = loader.extensions();

        assert_eq!(extensions.len(), 3);
        assert!(extensions.contains(&"amp.ron"));
        assert!(extensions.contains(&"scene.ron"));
        assert!(extensions.contains(&"prefab.ron"));
    }
}
