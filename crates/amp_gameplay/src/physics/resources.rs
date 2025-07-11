//! Physics resources for gameplay systems

use amp_physics::PhysicsConfig as PhysicsEngineConfig;
use bevy::prelude::*;

/// Resource for gameplay physics configuration
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    /// World gravity
    pub gravity: Vec3,
    /// Physics timestep
    pub timestep: f32,
    /// Maximum physics delta time
    pub max_dt: f32,
    /// Physics substeps
    pub substeps: usize,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            max_dt: 1.0 / 60.0,
            substeps: 1,
        }
    }
}

impl From<PhysicsConfig> for PhysicsEngineConfig {
    fn from(config: PhysicsConfig) -> Self {
        Self {
            enabled: true,
            debug_rendering: false,
            profiling: false,
            timestep_hz: 1.0 / config.timestep,
            max_steps_per_frame: config.substeps as u32,
            ccd_enabled: false,
            gravity: config.gravity,
            performance_monitoring: false,
        }
    }
}
