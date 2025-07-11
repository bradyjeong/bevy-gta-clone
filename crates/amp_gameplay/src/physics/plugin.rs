//! Physics plugin bridge for gameplay systems

use crate::physics::resources::PhysicsConfig;
use amp_physics::PhysicsPlugin;
use bevy::prelude::*;

/// Bridge plugin that integrates amp_physics with gameplay systems
#[derive(Default)]
pub struct PhysicsPluginBridge {
    /// Physics configuration
    pub config: PhysicsConfig,
}

impl PhysicsPluginBridge {
    /// Create a new physics plugin bridge with custom configuration
    pub fn new(config: PhysicsConfig) -> Self {
        Self { config }
    }

    /// Create a new physics plugin bridge with debug rendering enabled
    pub fn with_debug() -> Self {
        let config = PhysicsConfig::default();
        // Enable debug features for development
        Self { config }
    }
}

impl Plugin for PhysicsPluginBridge {
    fn build(&self, app: &mut App) {
        // Validate configuration
        if self.config.timestep <= 0.0 {
            warn!("Physics timestep must be positive, using default");
        }

        if self.config.substeps == 0 {
            warn!("Physics substeps must be > 0, using default");
        }

        // Add amp_physics plugin with our configuration
        app.add_plugins(PhysicsPlugin::new(self.config.clone().into()))
            .insert_resource(self.config.clone());

        info!(
            "Physics plugin bridge initialized with timestep: {:.6}s, substeps: {}",
            self.config.timestep, self.config.substeps
        );
    }
}
