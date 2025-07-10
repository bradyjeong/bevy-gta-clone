//! Asset loader for Amp scene prefabs

use super::scene::{AmpSceneComponent, AmpScenePrefab};
use bevy::asset::{AssetLoader, LoadContext};
use bevy::tasks::ConditionalSendFuture;
use serde::{Deserialize, Serialize};

/// Asset loader for Amp scene prefabs
#[derive(Debug, Default)]
pub struct AmpSceneLoader;

/// RON-serializable scene prefab definition (for loading)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonScenePrefab {
    /// Component definitions
    pub components: Vec<RonSceneComponent>,
}

/// RON-serializable component definition (for loading)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonSceneComponent {
    /// Component type name
    pub component_type: String,
    /// Component data as RON value
    pub data: ron::Value,
}

impl From<RonScenePrefab> for AmpScenePrefab {
    fn from(ron_prefab: RonScenePrefab) -> Self {
        let mut prefab = AmpScenePrefab::new();
        for component in ron_prefab.components {
            prefab.add_component(AmpSceneComponent::new(
                component.component_type,
                component.data,
            ));
        }
        prefab
    }
}

impl AssetLoader for AmpSceneLoader {
    type Asset = AmpScenePrefab;
    type Settings = ();
    type Error = AmpSceneLoaderError;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let content = std::str::from_utf8(&bytes)?;

            let ron_prefab: RonScenePrefab =
                ron::from_str(content).map_err(AmpSceneLoaderError::RonParseError)?;

            Ok(ron_prefab.into())
        }
    }

    fn extensions(&self) -> &[&str] {
        &["amp.ron", "scene.ron", "prefab.ron"]
    }
}

/// Error type for scene prefab loading
#[derive(Debug, thiserror::Error)]
pub enum AmpSceneLoaderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("RON parse error: {0}")]
    RonParseError(ron::error::SpannedError),
}
