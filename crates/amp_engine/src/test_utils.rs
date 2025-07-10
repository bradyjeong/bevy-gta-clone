//! Test utilities for amp_engine following Oracle's patterns

use crate::assets::AmpScenePlugin;
use bevy::MinimalPlugins;
use bevy::app::App;
use bevy::asset::AssetPlugin;

/// Creates a configured App instance for testing with minimal plugins
///
/// This helper follows Oracle's guidance for consistent App-based testing:
/// - Uses MinimalPlugins for minimal overhead
/// - Includes AssetPlugin for asset system testing
/// - Includes AmpScenePlugin for scene asset testing
/// - Returns a ready-to-use App instance
///
/// # Examples
///
/// ```rust
/// use amp_engine::test_utils::test_app;
///
/// let mut app = test_app();
/// app.update(); // Run one frame
///
/// // Access resources
/// let asset_server = app.world().resource::<AssetServer>();
///
/// // Access world mutably
/// let mut world = app.world_mut();
/// // ... make changes
/// ```
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(AmpScenePlugin);
    app
}

/// Creates a configured App instance for testing with custom asset directory
///
/// Use this when you need to test with specific asset files in a custom directory.
///
/// # Examples
///
/// ```rust
/// use amp_engine::test_utils::test_app_with_assets;
/// use tempfile::TempDir;
///
/// let temp_dir = TempDir::new().unwrap();
/// let mut app = test_app_with_assets(temp_dir.path().to_string_lossy().to_string());
/// app.update(); // Run one frame
/// ```
pub fn test_app_with_assets(asset_dir: String) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_dir,
            ..Default::default()
        })
        .add_plugins(AmpScenePlugin);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AmpScenePrefab;
    use bevy::asset::{AssetServer, Assets};

    #[test]
    fn test_app_creation() {
        let mut app = test_app();
        app.update();

        // Verify basic resources are available
        let _asset_server = app.world().resource::<AssetServer>();
        let _assets = app.world().resource::<Assets<AmpScenePrefab>>();
    }

    #[test]
    fn test_app_with_custom_assets() {
        let mut app = test_app_with_assets("test_assets".to_string());
        app.update();

        // Verify basic resources are available
        let _asset_server = app.world().resource::<AssetServer>();
        let _assets = app.world().resource::<Assets<AmpScenePrefab>>();
    }
}
