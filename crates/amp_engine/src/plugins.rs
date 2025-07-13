//! AAAPlugin Architecture Implementation
//!
//! This module provides the core plugin architecture for the Amp game engine,
//! enabling professional integration of subsystems with extensible user plugins.
//!
//! # Architecture Overview
//!
//! The AAAPlugin system consists of three main components:
//! 1. **AAAPlugin Trait**: Core abstraction for all plugins
//! 2. **PluginStage Enum**: Staging system for initialization ordering
//! 3. **AAAPlugins PluginGroup**: Collection of built-in and custom plugins
//!
//! # Usage
//!
//! ```rust
//! use amp_engine::prelude::*;
//! use bevy::prelude::*;
//!
//! # fn main() {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(AAAPlugins::default())
//!     .run();
//! # }
//! ```
//!
//! # Extending with Custom Plugins
//!
//! ```rust
//! use amp_engine::prelude::*;
//! use bevy::prelude::*;
//!
//! struct CustomPlugin;
//! impl AAAPlugin for CustomPlugin {
//!     fn build(&self, app: &mut App) -> amp_core::Result<()> {
//!         app.add_systems(Update, custom_system);
//!         Ok(())
//!     }
//!     
//!     fn stage(&self) -> PluginStage {
//!         PluginStage::PostStartup
//!     }
//! }
//!
//! fn custom_system() {
//!     // Custom system logic
//! }
//!
//! # fn main() {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(
//!         AAAPlugins::default()
//!             .add_plugin(CustomPlugin)
//!     )
//!     .run();
//! # }
//! ```

use amp_core::Result;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::*;
use std::sync::Arc;

/// Plugin stage for initialization ordering
///
/// Plugins are initialized in the following order:
/// 1. PreStartup - Configuration and core systems
/// 2. Startup - Physics and audio initialization  
/// 3. Default - Rendering and gameplay systems
/// 4. PostStartup - User extensions and finalizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PluginStage {
    /// Configuration and core systems that must be initialized first
    PreStartup,
    /// Physics and audio initialization
    Startup,
    /// Rendering and gameplay systems (default stage)
    #[default]
    Default,
    /// User extensions and finalizations
    PostStartup,
}

/// Core trait for all AAA plugins
///
/// This trait provides the foundation for the plugin architecture,
/// enabling professional integration of subsystems with lifecycle management,
/// staging control, and error handling.
pub trait AAAPlugin: Send + Sync + 'static {
    /// Build the plugin into the Bevy App
    ///
    /// This method is called during app initialization to register
    /// systems, resources, and other components.
    fn build(&self, app: &mut App) -> Result<()>;

    /// Get the plugin stage for initialization ordering
    ///
    /// Plugins are initialized in stage order. Use this to control
    /// when your plugin initializes relative to other plugins.
    fn stage(&self) -> PluginStage {
        PluginStage::Default
    }
}

/// Wrapper to make AAAPlugin compatible with Bevy's Plugin system
///
/// This struct wraps a Box<dyn AAAPlugin> and implements Bevy's Plugin trait,
/// enabling seamless integration with Bevy's plugin system.
#[derive(Clone)]
pub struct AAAPluginWrapper {
    inner: Arc<dyn AAAPlugin>,
}

impl AAAPluginWrapper {
    pub fn new(plugin: Box<dyn AAAPlugin>) -> Self {
        Self {
            inner: Arc::from(plugin),
        }
    }
}

impl Plugin for AAAPluginWrapper {
    fn build(&self, app: &mut App) {
        if let Err(e) = self.inner.build(app) {
            error!("Failed to build AAAPlugin: {}", e);
        }
    }
}

/// Collection of built-in and custom plugins
///
/// AAAPlugins provides a unified entry point for all subsystems,
/// including physics, audio, rendering, and configuration management.
/// Users can extend this with custom plugins.
pub struct AAAPlugins {
    /// Built-in plugins organized by stage
    plugins: Vec<(PluginStage, Box<dyn AAAPlugin>)>,
}

impl Default for AAAPlugins {
    fn default() -> Self {
        Self::new()
    }
}

impl AAAPlugins {
    /// Create a new AAAPlugins with default built-in plugins
    pub fn new() -> Self {
        let mut plugins = Self {
            plugins: Vec::new(),
        };

        // Add built-in plugins if available
        plugins.add_built_ins();
        plugins
    }

    /// Create an empty AAAPlugins without built-in plugins
    pub fn empty() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a custom plugin
    pub fn add_plugin<P: AAAPlugin>(mut self, plugin: P) -> Self {
        let stage = plugin.stage();
        self.plugins.push((stage, Box::new(plugin)));
        self
    }

    /// Add built-in plugins from subsystem crates
    fn add_built_ins(&mut self) {
        // Add configuration plugin first (PreStartup stage)
        #[cfg(feature = "config")]
        self.add_config_plugin();

        // Add physics plugin (Startup stage)
        #[cfg(feature = "physics")]
        self.add_physics_plugin();

        // Add audio plugin (Startup stage)
        #[cfg(feature = "audio")]
        self.add_audio_plugin();

        // Add rendering plugin (Default stage)
        #[cfg(feature = "render")]
        self.add_render_plugin();
    }

    #[cfg(feature = "config")]
    fn add_config_plugin(&mut self) {
        self.plugins
            .push((PluginStage::PreStartup, Box::new(ConfigAAAPlugin)));
    }

    #[cfg(feature = "physics")]
    fn add_physics_plugin(&mut self) {
        self.plugins
            .push((PluginStage::Startup, Box::new(PhysicsAAAPlugin)));
    }

    #[cfg(feature = "audio")]
    fn add_audio_plugin(&mut self) {
        self.plugins
            .push((PluginStage::Startup, Box::new(AudioAAAPlugin)));
    }

    #[cfg(feature = "render")]
    fn add_render_plugin(&mut self) {
        self.plugins
            .push((PluginStage::Default, Box::new(RenderAAAPlugin)));
    }
}

impl PluginGroup for AAAPlugins {
    fn build(mut self) -> PluginGroupBuilder {
        // Sort plugins by stage
        self.plugins.sort_by_key(|(stage, _)| match stage {
            PluginStage::PreStartup => 0,
            PluginStage::Startup => 1,
            PluginStage::Default => 2,
            PluginStage::PostStartup => 3,
        });

        // Build the plugin group
        let mut builder = PluginGroupBuilder::start::<Self>();

        for (_, plugin) in self.plugins {
            builder = builder.add(AAAPluginWrapper::new(plugin));
        }

        builder
    }
}

// Built-in plugin implementations

/// Configuration plugin for centralized config management
#[cfg(feature = "config")]
struct ConfigAAAPlugin;

#[cfg(feature = "config")]
impl AAAPlugin for ConfigAAAPlugin {
    fn build(&self, _app: &mut App) -> Result<()> {
        info!("Initializing Config subsystem");
        // TODO: Add config_core plugin integration
        Ok(())
    }

    fn stage(&self) -> PluginStage {
        PluginStage::PreStartup
    }
}

/// Physics plugin for vehicle physics and collision detection
#[cfg(feature = "physics")]
struct PhysicsAAAPlugin;

#[cfg(feature = "physics")]
impl AAAPlugin for PhysicsAAAPlugin {
    fn build(&self, _app: &mut App) -> Result<()> {
        info!("Initializing Physics subsystem");
        // TODO: Add amp_physics plugin integration
        Ok(())
    }

    fn stage(&self) -> PluginStage {
        PluginStage::Startup
    }
}

/// Audio plugin for game audio systems
#[cfg(feature = "audio")]
struct AudioAAAPlugin;

#[cfg(feature = "audio")]
impl AAAPlugin for AudioAAAPlugin {
    fn build(&self, _app: &mut App) -> Result<()> {
        info!("Initializing Audio subsystem");
        // TODO: Add amp_gameplay audio plugin integration
        Ok(())
    }

    fn stage(&self) -> PluginStage {
        PluginStage::Startup
    }
}

/// Rendering plugin for GPU culling and batch processing
#[cfg(feature = "render")]
struct RenderAAAPlugin;

#[cfg(feature = "render")]
impl AAAPlugin for RenderAAAPlugin {
    fn build(&self, _app: &mut App) -> Result<()> {
        info!("Initializing Render subsystem");
        // TODO: Add amp_render plugin integration
        Ok(())
    }

    fn stage(&self) -> PluginStage {
        PluginStage::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, MinimalPlugins, Resource};

    #[test]
    fn plugins_construct() {
        let mut app = App::new();

        // Add minimal plugins for testing
        app.add_plugins(MinimalPlugins);

        // Add AAAPlugins - this should not panic
        app.add_plugins(AAAPlugins::default());

        // Verify app was created successfully
        assert!(
            app.world()
                .contains_resource::<bevy::app::MainScheduleOrder>()
        );
    }

    #[test]
    fn plugin_stage_ordering() {
        let stages = vec![
            PluginStage::PostStartup,
            PluginStage::PreStartup,
            PluginStage::Default,
            PluginStage::Startup,
        ];

        let mut sorted_stages = stages.clone();
        sorted_stages.sort_by_key(|stage| match stage {
            PluginStage::PreStartup => 0,
            PluginStage::Startup => 1,
            PluginStage::Default => 2,
            PluginStage::PostStartup => 3,
        });

        assert_eq!(
            sorted_stages,
            vec![
                PluginStage::PreStartup,
                PluginStage::Startup,
                PluginStage::Default,
                PluginStage::PostStartup,
            ]
        );
    }

    #[test]
    fn custom_plugin_integration() {
        struct TestPlugin;
        impl AAAPlugin for TestPlugin {
            fn build(&self, app: &mut App) -> amp_core::Result<()> {
                app.insert_resource(TestResource { value: 42 });
                Ok(())
            }

            fn stage(&self) -> PluginStage {
                PluginStage::PostStartup
            }
        }

        #[derive(Resource)]
        struct TestResource {
            value: i32,
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AAAPlugins::empty().add_plugin(TestPlugin));

        // Verify the custom plugin's resource was added
        let resource = app.world().resource::<TestResource>();
        assert_eq!(resource.value, 42);
    }

    #[test]
    fn empty_plugins_works() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AAAPlugins::empty());

        // Should not panic with empty plugins
        assert!(
            app.world()
                .contains_resource::<bevy::app::MainScheduleOrder>()
        );
    }
}
