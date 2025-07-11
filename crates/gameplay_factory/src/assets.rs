//! Prefab asset integration for Bevy asset pipeline

use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Error;
use crate::dsl::{DslConfig, parse_prefab_ron};
use crate::prefab::Prefab;

/// Prefab asset that can be loaded through Bevy's asset system
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PrefabAsset {
    /// Component data in RON format
    pub components: HashMap<String, ron::Value>,
    /// Metadata about the prefab
    pub metadata: PrefabMetadata,
}

/// Metadata for prefab assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabMetadata {
    /// Human-readable name for the prefab
    pub name: String,
    /// Version identifier for asset compatibility
    pub version: String,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl Default for PrefabMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Prefab".to_string(),
            version: "1.0.0".to_string(),
            tags: Vec::new(),
        }
    }
}

/// Asset loader for prefab RON files
#[derive(Default)]
pub struct PrefabAssetLoader;

impl AssetLoader for PrefabAssetLoader {
    type Asset = PrefabAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PrefabAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;

        let prefab_asset: PrefabAsset = ron::from_str(content).map_err(|e| {
            error!(
                "Failed to parse prefab asset '{}': {}",
                load_context.path().display(),
                e
            );
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;

        info!(
            "Loaded prefab asset '{}' ({})",
            prefab_asset.metadata.name,
            load_context.path().display()
        );

        Ok(prefab_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["prefab.ron"]
    }
}

/// Plugin for prefab asset loading
#[derive(Default)]
pub struct PrefabAssetPlugin;

impl Plugin for PrefabAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PrefabAsset>()
            .init_asset_loader::<PrefabAssetLoader>();
    }
}

/// Convert PrefabAsset to runtime Prefab using the component registry
pub fn convert_prefab_asset_to_runtime_prefab(
    prefab_asset: &PrefabAsset,
    dsl_config: &DslConfig,
    type_registry: &AppTypeRegistry,
) -> Result<Prefab, Error> {
    // Convert the component map to the format expected by the DSL
    let mut component_ron = String::new();
    component_ron.push_str("{\n");

    for (component_name, component_value) in &prefab_asset.components {
        component_ron.push_str(&format!("    \"{component_name}\": {component_value:?},\n"));
    }

    component_ron.push_str("}\n");

    // Use the existing DSL parsing logic
    let component_map = parse_prefab_ron(&component_ron, type_registry, dsl_config)?;

    // Create prefab from component map using the existing helper
    crate::dsl::create_prefab_from_component_map(&component_map, type_registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{DslConfig, ValidationMode};
    use crate::register_default_components;

    #[test]
    fn test_prefab_asset_creation() {
        let mut components = HashMap::new();
        components.insert("Transform".to_string(), ron::Value::Map(ron::Map::new()));

        let prefab_asset = PrefabAsset {
            components,
            metadata: PrefabMetadata {
                name: "Test Prefab".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["test".to_string()],
            },
        };

        assert_eq!(prefab_asset.metadata.name, "Test Prefab");
        assert_eq!(prefab_asset.components.len(), 1);
    }

    #[test]
    fn test_prefab_asset_loader_extensions() {
        let loader = PrefabAssetLoader;
        assert_eq!(loader.extensions(), &["prefab.ron"]);
    }

    #[test]
    fn test_convert_prefab_asset_to_runtime_prefab() {
        let mut components = HashMap::new();
        components.insert("Transform".to_string(), ron::Value::Map(ron::Map::new()));

        let prefab_asset = PrefabAsset {
            components,
            metadata: PrefabMetadata::default(),
        };

        register_default_components();

        let dsl_config = DslConfig {
            validation_mode: ValidationMode::Skip,
            ..Default::default()
        };
        let type_registry = AppTypeRegistry::default();
        let result =
            convert_prefab_asset_to_runtime_prefab(&prefab_asset, &dsl_config, &type_registry);

        // Should succeed with registered components
        assert!(result.is_ok());
    }
}
