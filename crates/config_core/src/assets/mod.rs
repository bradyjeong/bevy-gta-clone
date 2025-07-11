//! Generic typed RON → Bevy Asset loader.
use bevy::asset::{AssetLoadFailedEvent, AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Generic asset loader for RON files
pub struct RonAssetLoader<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for RonAssetLoader<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> AssetLoader for RonAssetLoader<T>
where
    T: Asset + DeserializeOwned + Send + Sync + 'static,
{
    type Asset = T;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<T, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let asset: T = ron::from_str(content).map_err(|e| {
            error!(
                "Failed to parse RON asset '{}': {}",
                load_context.path().display(),
                e
            );
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Resource handle for configuration assets
#[derive(Resource)]
pub struct ConfigHandle<T: Asset> {
    pub handle: Handle<T>,
    pub data: Option<T>,
}

/// Resource holding the path to the configuration file
#[derive(Resource)]
pub struct ConfigPath(pub String);

impl<T: Asset> Default for ConfigHandle<T> {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            data: None,
        }
    }
}

impl<T: Asset> ConfigHandle<T> {
    /// Get the configuration data if available
    pub fn get(&self) -> Option<&T> {
        self.data.as_ref()
    }

    /// Check if the configuration is loaded
    pub fn is_loaded(&self) -> bool {
        self.data.is_some()
    }
}

/// Plugin for loading RON configurations as Bevy assets
pub struct RonAssetPlugin<T: Asset + TypePath + Clone + Send + Sync + 'static> {
    path: &'static str,
    _marker: PhantomData<T>,
}

impl<T: Asset + TypePath + Clone + Send + Sync + 'static> RonAssetPlugin<T> {
    pub const fn new(path: &'static str) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }
}

impl<T> Plugin for RonAssetPlugin<T>
where
    T: Asset + TypePath + Clone + Send + Sync + 'static + DeserializeOwned,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<T>()
            .init_resource::<ConfigHandle<T>>()
            .init_asset_loader::<RonAssetLoader<T>>()
            .insert_resource(ConfigPath(self.path.to_string()))
            .add_systems(
                Startup,
                load_config::<T>.run_if(resource_exists::<ConfigHandle<T>>),
            )
            .add_systems(
                Update,
                publish_config::<T>.run_if(resource_exists::<ConfigHandle<T>>),
            );
    }
}

/// Startup system to load configuration files
fn load_config<T: Asset>(
    mut config_handle: ResMut<ConfigHandle<T>>,
    asset_server: Res<AssetServer>,
    config_path: Res<ConfigPath>,
) {
    config_handle.handle = asset_server.load(&config_path.0);
    info!("Loading config: {}", config_path.0);
}

/// Update system to handle configuration hot-reload
fn publish_config<T: Asset + Clone>(
    mut config_handle: ResMut<ConfigHandle<T>>,
    assets: Res<Assets<T>>,
    mut asset_events: EventReader<AssetEvent<T>>,
    mut load_failed_events: EventReader<AssetLoadFailedEvent<T>>,
) {
    // Handle successful loads and modifications
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                if *id == config_handle.handle.id() {
                    if let Some(config) = assets.get(&config_handle.handle) {
                        config_handle.data = Some(config.clone());
                        info!("Config {} reloaded", std::any::type_name::<T>());
                    } else {
                        warn!(
                            "Config {} loaded but not found in assets",
                            std::any::type_name::<T>()
                        );
                    }
                }
            }
            AssetEvent::Removed { id } => {
                if *id == config_handle.handle.id() {
                    config_handle.data = None;
                    warn!("Config {} removed", std::any::type_name::<T>());
                }
            }
            _ => {}
        }
    }

    // Handle loading failures
    for event in load_failed_events.read() {
        if event.id == config_handle.handle.id() {
            error!(
                "Failed to load config {}: {}",
                std::any::type_name::<T>(),
                event.error
            );
            config_handle.data = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
    struct TestConfig {
        pub value: i32,
        pub name: String,
    }

    #[test]
    fn test_ron_asset_loader_creation() {
        let loader = RonAssetLoader::<TestConfig>::default();
        assert_eq!(loader.extensions(), &["ron"]);
    }

    #[test]
    fn test_config_handle_default() {
        let handle = ConfigHandle::<TestConfig>::default();
        assert!(!handle.is_loaded());
        assert!(handle.get().is_none());
    }

    #[test]
    fn test_config_handle_with_data() {
        let handle = ConfigHandle::<TestConfig> {
            data: Some(TestConfig {
                value: 42,
                name: "test".to_string(),
            }),
            ..Default::default()
        };
        assert!(handle.is_loaded());
        assert!(handle.get().is_some());
        assert_eq!(handle.get().unwrap().value, 42);
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = RonAssetPlugin::<TestConfig>::new("test/path.ron");
        // Just verify plugin can be created
        assert_eq!(plugin.path, "test/path.ron");
    }
}
