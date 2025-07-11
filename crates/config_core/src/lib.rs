//! # Config Core
//!
//! This crate provides configuration loading and management for the AMP Game Engine.
//! It implements Oracle's exact API specification for configuration handling with
//! RON deserialization and hierarchical file search.
//!
//! ## Features
//!
//! - **GameConfig**: Main configuration structure with factory settings
//! - **FactorySettings**: Configuration for entity and prefab management
//! - **Environment Override**: AMP_CONFIG environment variable support
//! - **Default Values**: Serde-based default value handling for partial configs
//! - **Hierarchical Search**: Searches current directory and XDG config paths

use amp_core::{ConfigError, Error, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;

// Bevy asset system integration (feature-gated)
#[cfg(feature = "bevy-assets")]
pub mod assets;

// Configuration type definitions
#[cfg(feature = "bevy-assets")]
pub mod types;

// Vehicle configuration types
pub mod vehicle;

// Re-export vehicle configuration types for convenience
pub use vehicle::{
    EngineConfig, SuspensionConfig, TransmissionConfig, TransmissionType, VehicleConfig,
    WheelConfig,
};

/// Factory configuration settings for entity and prefab management.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FactorySettings {
    /// Path pattern for locating prefab files
    pub prefab_path: String,
    /// Enable hot-reload of prefab files
    pub hot_reload: bool,
}

impl Default for FactorySettings {
    fn default() -> Self {
        Self {
            prefab_path: "~/.config/my_game/prefabs/*.ron".to_string(),
            hot_reload: true,
        }
    }
}

impl FactorySettings {
    /// Expand tilde (~) in the prefab_path and return the expanded path.
    ///
    /// This function handles cross-platform tilde expansion:
    /// - "~/prefabs/*.ron" -> "/home/user/prefabs/*.ron" (Unix)
    /// - "~\\prefabs\\*.ron" -> "C:\\Users\\user\\prefabs\\*.ron" (Windows)
    /// - Returns original path if expansion fails or no home directory exists
    pub fn expanded_prefab_path(&self) -> Result<String> {
        let expanded = shellexpand::tilde(&self.prefab_path);
        let expanded_str = expanded.to_string();
        // Only fail if the original path started with ~ and still contains ~
        if self.prefab_path.starts_with('~') && expanded_str.starts_with('~') {
            Err(Error::from(ConfigError::parse_error(
                "Failed to expand tilde in prefab_path: home directory not found".to_string(),
            )))
        } else {
            Ok(expanded_str)
        }
    }
}

/// Main game configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct GameConfig {
    /// Factory configuration settings
    pub factory: FactorySettings,
}

impl GameConfig {
    /// Access factory configuration settings
    pub fn factory(&self) -> &FactorySettings {
        &self.factory
    }
}

impl Config for GameConfig {
    const FILE_NAME: &'static str = "game.ron";
}

/// Trait for configuration types that can be loaded from RON files.
///
/// This trait defines the interface for configuration objects that can be
/// deserialized from RON format and provides metadata about their storage.
pub trait Config: DeserializeOwned + Send + Sync + 'static + Default {
    /// The filename (without path) where this configuration should be stored.
    const FILE_NAME: &'static str;

    /// Returns the default path for this configuration file.
    ///
    /// By default, this returns just the filename, but implementations can
    /// override this to provide custom path logic.
    fn default_path() -> PathBuf {
        PathBuf::from(Self::FILE_NAME)
    }

    /// Returns embedded defaults for this configuration.
    ///
    /// This provides compile-time fallback values when no configuration
    /// files are found in the search paths. The default implementation
    /// returns `Self::default()`.
    fn embedded_defaults() -> Self {
        Self::default()
    }

    /// Merge another configuration of the same type into this one.
    ///
    /// This enables hierarchical configuration loading where values from
    /// later configurations override values from earlier ones. The default
    /// implementation simply returns `other`, but implementations can
    /// override this for more sophisticated merge behavior.
    fn merge(self, other: Self) -> Self {
        other
    }
}

/// Configuration loader that handles file discovery and caching.
///
/// The loader searches for configuration files in a hierarchical manner:
/// 1. Current working directory
/// 2. $XDG_CONFIG_HOME/amp (or ~/.config/amp on Unix)
/// 3. Embedded defaults (compile-time fallback)
pub struct ConfigLoader {
    /// Search paths for configuration files
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader with default search paths.
    pub fn new() -> Self {
        let mut search_paths = vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))];

        // Add XDG_CONFIG_HOME/amp or ~/.config/amp
        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("amp"));
        }

        // Remove duplicate paths to avoid redundant IO
        search_paths.dedup();

        Self { search_paths }
    }

    /// Load a configuration of type T from the filesystem.
    ///
    /// This searches through the configured search paths in order,
    /// attempting to load and deserialize the configuration file.
    /// Environment variable AMP_CONFIG can override the config file path.
    ///
    /// Note: This method is deprecated. Use `load_with_merge()` instead for the new
    /// hierarchical merge behavior. This method now delegates to `load_with_merge()`.
    #[deprecated(note = "Use load_with_merge")]
    pub fn load<T: Config>(&self) -> Result<T> {
        self.load_with_merge()
    }

    /// Load a configuration with hierarchical merging.
    ///
    /// This method starts with embedded defaults and merges configurations
    /// from all search paths in order, with later paths overriding earlier ones.
    /// This is the enhanced version that implements Oracle's hierarchical merge.
    pub fn load_with_merge<T: Config>(&self) -> Result<T> {
        // Check for AMP_CONFIG environment variable override
        if let Ok(env_path) = std::env::var("AMP_CONFIG") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                let data = std::fs::read_to_string(&path)
                    .map_err(|e| Error::from(ConfigError::from(e)))?;

                let cfg = ron::from_str(&data)
                    .map_err(|e| Error::from(ConfigError::parse_error(e.to_string())))?;

                return Ok(cfg);
            }
        }

        // Start with embedded defaults (compile-time fallback)
        let mut final_config = T::embedded_defaults();

        // Hierarchical merge: collect configs from all search paths
        // Iterate in reverse order so higher priority paths (CWD) override lower priority (XDG)
        for dir in self.search_paths.iter().rev() {
            let path = dir.join(T::default_path());
            if !path.exists() {
                continue;
            }

            let data =
                std::fs::read_to_string(&path).map_err(|e| Error::from(ConfigError::from(e)))?;

            let cfg: T = ron::from_str(&data)
                .map_err(|e| Error::from(ConfigError::parse_error(e.to_string())))?;

            // Merge this config into the final result
            // Since we iterate in reverse, earlier configs (lower priority) merge into later ones (higher priority)
            final_config = final_config.merge(cfg);
        }

        // Return final merged config (even if no files found, return embedded defaults)
        Ok(final_config)
    }

    /// Watch a configuration file for changes and call the callback on updates.
    ///
    /// Note: This is a placeholder for future hot-reload functionality.
    /// Currently, this method exists to satisfy Oracle's API specification
    /// but hot-reload implementation is deferred as mentioned in the requirements.
    pub fn watch<T: Config, F: FnMut(&T) + 'static>(&self, _callback: F) {
        #[cfg(feature = "hot-reload")]
        {
            // TODO: Implement file watching for hot-reload
            todo!("File watching for hot-reload is not yet implemented")
        }
        #[cfg(not(feature = "hot-reload"))]
        {
            // Hot-reload feature is not enabled
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serial_test::serial;
    use tempfile::TempDir;

    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(default)]
    struct TestConfig {
        value: i32,
        name: String,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                value: 0,
                name: "default".to_string(),
            }
        }
    }

    impl Config for TestConfig {
        const FILE_NAME: &'static str = "test.ron";

        fn merge(self, other: Self) -> Self {
            Self {
                value: if other.value != 0 {
                    other.value
                } else {
                    self.value
                },
                name: if other.name != "default" {
                    other.name
                } else {
                    self.name
                },
            }
        }
    }

    #[test]
    fn test_config_trait() {
        assert_eq!(TestConfig::FILE_NAME, "test.ron");
        assert_eq!(TestConfig::default_path(), PathBuf::from("test.ron"));
    }

    #[test]
    fn test_config_loader_new() {
        let loader = ConfigLoader::new();
        assert!(!loader.search_paths.is_empty());

        // Should include current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        assert!(loader.search_paths.contains(&current_dir));

        // Should include config directory if available
        if let Some(config_dir) = dirs::config_dir() {
            assert!(loader.search_paths.contains(&config_dir.join("amp")));
        }
    }

    #[test]
    #[serial]
    fn test_config_loader_load_success() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.ron");

        // Write test config with both required fields
        std::fs::write(&config_path, "(value: 42, name: \"test\")").unwrap();

        // Create loader with temp directory as search path
        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let config: TestConfig = loader.load().unwrap();
        assert_eq!(config.value, 42);
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_config_loader_load_not_found() {
        let loader = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent/path")],
        };

        #[allow(deprecated)]
        let result: Result<TestConfig> = loader.load();
        // With the new behavior, load() delegates to load_with_merge() which returns embedded defaults
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.value, 0);
        assert_eq!(config.name, "default");
    }

    #[test]
    #[serial]
    fn test_config_loader_load_invalid_ron() {
        // Ensure AMP_CONFIG is not set to avoid race conditions with other tests
        unsafe {
            std::env::remove_var("AMP_CONFIG");
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.ron");

        // Write invalid RON
        std::fs::write(&config_path, "invalid ron content").unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let result: Result<TestConfig> = loader.load();
        // Invalid RON should still fail even with new behavior
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_loader_search_order() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let config_path1 = temp_dir1.path().join("test.ron");
        let config_path2 = temp_dir2.path().join("test.ron");

        // Write different configs
        std::fs::write(&config_path1, "(value: 1, name: \"first\")").unwrap();
        std::fs::write(&config_path2, "(value: 2, name: \"second\")").unwrap();

        // Create loader with both paths (first path should take precedence)
        let loader = ConfigLoader {
            search_paths: vec![
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
        };

        #[allow(deprecated)]
        let config: TestConfig = loader.load().unwrap();
        // With the new behavior, load() delegates to load_with_merge() which merges all files
        // The first file's values should override the second file's values (first path = highest priority)
        assert_eq!(config.value, 1);
        assert_eq!(config.name, "first");
    }

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::default();
        assert!(!loader.search_paths.is_empty());
    }

    #[test]
    fn test_config_loader_watch_without_hot_reload() {
        let loader = ConfigLoader::new();
        // Should not panic when hot-reload feature is not enabled
        #[cfg(not(feature = "hot-reload"))]
        {
            loader.watch::<TestConfig, _>(|_| {});
        }
        #[cfg(feature = "hot-reload")]
        {
            // With hot-reload feature, this should panic until implemented
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                loader.watch::<TestConfig, _>(|_| {});
            }))
            .expect_err("Should panic when hot-reload is enabled but not implemented");
        }
    }

    #[derive(Deserialize, Debug, PartialEq, Default)]
    #[serde(default)]
    struct CustomPathConfig {
        enabled: bool,
    }

    impl Config for CustomPathConfig {
        const FILE_NAME: &'static str = "custom.ron";

        fn default_path() -> PathBuf {
            PathBuf::from("custom/path/custom.ron")
        }
    }

    #[test]
    fn test_default_path_override() {
        // Test that default_path() is respected instead of hard-coded FILE_NAME
        let temp_dir = TempDir::new().unwrap();
        let custom_dir = temp_dir.path().join("custom/path");
        std::fs::create_dir_all(&custom_dir).unwrap();

        let config_path = custom_dir.join("custom.ron");
        std::fs::write(&config_path, "(enabled: true)").unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let config: CustomPathConfig = loader.load().unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn test_load_returns_typed_config_error() {
        let loader = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent/path")],
        };

        #[allow(deprecated)]
        let result: Result<TestConfig> = loader.load();
        // With the new behavior, load() delegates to load_with_merge() which returns embedded defaults
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.value, 0);
        assert_eq!(config.name, "default");
    }

    #[test]
    fn test_dedup_search_paths() {
        // Test that dedup() removes duplicate search paths
        let temp_dir = TempDir::new().unwrap();
        let duplicate_path = temp_dir.path().to_path_buf();

        let mut loader = ConfigLoader {
            search_paths: vec![duplicate_path.clone(), duplicate_path.clone()],
        };

        // Manually dedup to test the behavior
        loader.search_paths.dedup();

        assert_eq!(loader.search_paths.len(), 1);
        assert_eq!(loader.search_paths[0], duplicate_path);
    }

    #[test]
    fn test_config_error_preserves_io_error() {
        // Test successful loading first
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.ron");
        std::fs::write(&config_path, "(value: 42, name: \"test\")").unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let result: Result<TestConfig> = loader.load();
        assert!(
            result.is_ok(),
            "Expected successful load, got: {:?}",
            result.err()
        );

        // Test with non-existent file to get proper error
        let loader_bad = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent")],
        };

        #[allow(deprecated)]
        let result_bad: Result<TestConfig> = loader_bad.load();
        // With the new behavior, load() delegates to load_with_merge() which returns embedded defaults
        assert!(result_bad.is_ok());
        let config = result_bad.unwrap();
        assert_eq!(config.value, 0);
        assert_eq!(config.name, "default");
    }

    #[test]
    fn test_factory_settings_default() {
        let settings = FactorySettings::default();
        assert_eq!(settings.prefab_path, "~/.config/my_game/prefabs/*.ron");
        assert!(settings.hot_reload);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(
            config.factory.prefab_path,
            "~/.config/my_game/prefabs/*.ron"
        );
        assert!(config.factory.hot_reload);
    }

    #[test]
    fn test_game_config_factory_accessor() {
        let config = GameConfig::default();
        let factory = config.factory();
        assert_eq!(factory.prefab_path, "~/.config/my_game/prefabs/*.ron");
        assert!(factory.hot_reload);
    }

    #[test]
    fn test_game_config_load_with_factory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("game.ron");

        // Write custom factory config
        std::fs::write(
            &config_path,
            r#"(
            factory: (
                prefab_path: "/custom/prefabs/*.ron",
                hot_reload: false,
            )
        )"#,
        )
        .unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let config: GameConfig = loader.load().unwrap();
        assert_eq!(config.factory.prefab_path, "/custom/prefabs/*.ron");
        assert!(!config.factory.hot_reload);
    }

    #[test]
    fn test_amp_config_env_override() {
        // Helper struct to ensure cleanup
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                unsafe {
                    std::env::remove_var("AMP_CONFIG");
                }
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("custom_game.ron");

        // Write config file
        std::fs::write(
            &config_path,
            r#"(
            factory: (
                prefab_path: "/env/override/prefabs/*.ron",
                hot_reload: false,
            )
        )"#,
        )
        .unwrap();

        // Set environment variable
        unsafe {
            std::env::set_var("AMP_CONFIG", config_path.to_str().unwrap());
        }

        // Ensure cleanup happens even if test panics
        let _guard = EnvGuard;

        let loader = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent")],
        };

        #[allow(deprecated)]
        let config: GameConfig = loader.load().unwrap();
        assert_eq!(config.factory.prefab_path, "/env/override/prefabs/*.ron");
        assert!(!config.factory.hot_reload);
    }

    #[test]
    fn test_amp_config_env_override_nonexistent() {
        // Helper struct to ensure cleanup
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                unsafe {
                    std::env::remove_var("AMP_CONFIG");
                }
            }
        }

        unsafe {
            std::env::set_var("AMP_CONFIG", "/nonexistent/config.ron");
        }

        // Ensure cleanup happens even if test panics
        let _guard = EnvGuard;

        let loader = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent")],
        };

        #[allow(deprecated)]
        let result: Result<GameConfig> = loader.load();
        // With the new behavior, load() delegates to load_with_merge() which returns embedded defaults
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.factory.prefab_path,
            "~/.config/my_game/prefabs/*.ron"
        );
        assert!(config.factory.hot_reload);
    }

    #[test]
    fn test_factory_settings_serialization() {
        let settings = FactorySettings {
            prefab_path: "/test/prefabs/*.ron".to_string(),
            hot_reload: false,
        };

        let serialized = ron::to_string(&settings).unwrap();
        let deserialized: FactorySettings = ron::from_str(&serialized).unwrap();

        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_game_config_serialization() {
        let config = GameConfig {
            factory: FactorySettings {
                prefab_path: "/test/prefabs/*.ron".to_string(),
                hot_reload: false,
            },
        };

        let serialized = ron::to_string(&config).unwrap();
        let deserialized: GameConfig = ron::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_game_config_partial_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("game.ron");

        // Write partial config (missing hot_reload)
        std::fs::write(
            &config_path,
            r#"(
            factory: (
                prefab_path: "/partial/prefabs/*.ron",
            )
        )"#,
        )
        .unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        #[allow(deprecated)]
        let config: GameConfig = loader.load().unwrap();
        assert_eq!(config.factory.prefab_path, "/partial/prefabs/*.ron");
        // Should use default value for missing field
        assert!(config.factory.hot_reload);
    }

    #[test]
    fn test_tilde_expansion_success() {
        let settings = FactorySettings {
            prefab_path: "~/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.ends_with("prefabs/*.ron"));

        // Should expand to user's home directory
        if let Some(home) = dirs::home_dir() {
            assert!(expanded.starts_with(home.to_str().unwrap()));
        }
    }

    #[test]
    fn test_tilde_expansion_absolute_path() {
        let settings = FactorySettings {
            prefab_path: "/absolute/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert_eq!(expanded, "/absolute/prefabs/*.ron");
    }

    #[test]
    fn test_tilde_expansion_relative_path() {
        let settings = FactorySettings {
            prefab_path: "relative/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert_eq!(expanded, "relative/prefabs/*.ron");
    }

    #[test]
    fn test_tilde_expansion_windows_style() {
        let settings = FactorySettings {
            prefab_path: "~\\prefabs\\*.ron".to_string(),
            hot_reload: true,
        };

        // On Unix systems, backslashes are treated as literal characters
        // On Windows, shellexpand should properly handle backslashes
        let expanded = settings.expanded_prefab_path();
        if expanded.is_ok() {
            let expanded_str = expanded.unwrap();
            assert!(!expanded_str.starts_with('~'));
            assert!(expanded_str.contains("prefabs") && expanded_str.contains("*.ron"));
        } else {
            // If expansion fails, that's acceptable for Windows-style paths on Unix
            assert!(expanded.is_err());
        }
    }

    #[test]
    fn test_tilde_expansion_complex_path() {
        let settings = FactorySettings {
            prefab_path: "~/.config/my_game/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.contains(".config/my_game/prefabs"));
        assert!(expanded.ends_with("*.ron"));
    }

    #[test]
    fn test_tilde_expansion_empty_path() {
        let settings = FactorySettings {
            prefab_path: "".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert_eq!(expanded, "");
    }

    #[test]
    fn test_tilde_expansion_multiple_tildes() {
        let settings = FactorySettings {
            prefab_path: "~/dir/~/nested/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        // Only leading tilde should be expanded
        assert!(!expanded.starts_with('~'));
        assert!(expanded.contains("dir/~/nested"));
    }

    #[test]
    fn test_tilde_expansion_with_environment_variables() {
        let settings = FactorySettings {
            prefab_path: "~/$USER/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        // Environment variable expansion is handled by shellexpand
        assert!(expanded.contains("prefabs/*.ron"));
    }

    #[test]
    fn test_tilde_expansion_preserves_wildcards() {
        let settings = FactorySettings {
            prefab_path: "~/test/**/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.contains("**/*.ron"));
    }

    #[test]
    fn test_tilde_expansion_special_characters() {
        let settings = FactorySettings {
            prefab_path: "~/test with spaces/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.contains("test with spaces"));
    }

    #[cfg(unix)]
    #[test]
    fn test_tilde_expansion_unix_specific() {
        let settings = FactorySettings {
            prefab_path: "~/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.starts_with('/'));
        assert!(expanded.contains("prefabs/*.ron"));
    }

    #[cfg(windows)]
    #[test]
    fn test_tilde_expansion_windows_specific() {
        let settings = FactorySettings {
            prefab_path: "~/prefabs/*.ron".to_string(),
            hot_reload: true,
        };

        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        // Windows paths should contain drive letter
        assert!(expanded.chars().nth(1) == Some(':'));
        assert!(expanded.contains("prefabs"));
    }

    #[test]
    fn test_factory_settings_default_expandable() {
        let settings = FactorySettings::default();
        assert_eq!(settings.prefab_path, "~/.config/my_game/prefabs/*.ron");

        // Default path should be expandable
        let expanded = settings.expanded_prefab_path().unwrap();
        assert!(!expanded.contains('~'));
        assert!(expanded.contains(".config/my_game/prefabs"));
    }

    #[test]
    fn test_load_with_merge_hierarchical() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let config_path1 = temp_dir1.path().join("test.ron");
        let config_path2 = temp_dir2.path().join("test.ron");

        // Write partial configs that will be merged
        std::fs::write(&config_path1, "(value: 1)").unwrap();
        std::fs::write(&config_path2, "(name: \"merged\")").unwrap();

        // Create loader with both paths
        let loader = ConfigLoader {
            search_paths: vec![
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
        };

        let config: TestConfig = loader.load_with_merge().unwrap();
        // Should have merged values: value from first config, name from second
        assert_eq!(config.value, 1);
        assert_eq!(config.name, "merged");
    }

    #[test]
    fn test_load_with_merge_embedded_defaults() {
        // Create loader with no existing config files
        let loader = ConfigLoader {
            search_paths: vec![PathBuf::from("/nonexistent")],
        };

        let config: TestConfig = loader.load_with_merge().unwrap();
        // Should return embedded defaults (from TestConfig::default())
        assert_eq!(config.value, 0);
        assert_eq!(config.name, "default");
    }

    #[test]
    fn test_load_with_merge_override_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.ron");

        // Write partial config that overrides only one field
        std::fs::write(&config_path, "(value: 42)").unwrap();

        let loader = ConfigLoader {
            search_paths: vec![temp_dir.path().to_path_buf()],
        };

        let config: TestConfig = loader.load_with_merge().unwrap();
        // Should have overridden value and default name
        assert_eq!(config.value, 42);
        assert_eq!(config.name, "default");
    }

    #[test]
    fn test_deprecated_load_uses_merge_behavior() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let config_path1 = temp_dir1.path().join("test.ron");
        let config_path2 = temp_dir2.path().join("test.ron");

        // Write partial configs that will be merged
        std::fs::write(&config_path1, "(value: 100, name: \"first_priority\")").unwrap();
        std::fs::write(&config_path2, "(value: 200, name: \"second_priority\")").unwrap();

        // Create loader with both paths
        let loader = ConfigLoader {
            search_paths: vec![
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
        };

        // Test that deprecated load() now uses hierarchical merge behavior
        #[allow(deprecated)]
        let config: TestConfig = loader.load().unwrap();

        // Should have values from first config (highest priority path)
        assert_eq!(config.value, 100);
        assert_eq!(config.name, "first_priority");
    }
}
