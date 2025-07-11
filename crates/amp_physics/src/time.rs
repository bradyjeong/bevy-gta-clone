//! Physics time management for consistent simulation.
//!
//! This module provides fixed timestep physics simulation to ensure
//! consistent and deterministic physics behavior at 60 Hz.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Fixed timestep physics time resource.
///
/// Maintains a consistent 60 Hz physics simulation rate regardless of
/// frame rate variations. Uses time accumulation for sub-frame interpolation.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "inspector", derive(bevy::reflect::Reflect))]
pub struct PhysicsTime {
    /// Fixed timestep duration (1/60 seconds)
    pub fixed_timestep: f32,
    /// Accumulated time since last physics update
    pub accumulator: f32,
    /// Maximum number of physics steps per frame
    pub max_steps: u32,
    /// Current interpolation alpha for rendering (0.0 to 1.0)
    pub interpolation_alpha: f32,
    /// Total physics time elapsed
    pub total_time: f32,
    /// Physics step counter
    pub step_count: u64,
    /// Enable/disable physics simulation
    pub enabled: bool,
}

impl Default for PhysicsTime {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 60.0, // 60 Hz
            accumulator: 0.0,
            max_steps: 4,
            interpolation_alpha: 0.0,
            total_time: 0.0,
            step_count: 0,
            enabled: true,
        }
    }
}

impl PhysicsTime {
    /// Create a new PhysicsTime with custom timestep.
    pub fn new(timestep_hz: f32) -> Self {
        Self {
            fixed_timestep: 1.0 / timestep_hz,
            ..Default::default()
        }
    }

    /// Update the physics time accumulator.
    pub fn update(&mut self, delta_time: f32) {
        if self.enabled {
            self.accumulator += delta_time;

            // Calculate interpolation alpha for smooth rendering
            self.interpolation_alpha = self.accumulator / self.fixed_timestep;
            self.interpolation_alpha = self.interpolation_alpha.clamp(0.0, 1.0);
        }
    }

    /// Check if a physics step should be taken.
    pub fn should_step(&self) -> bool {
        self.enabled && self.accumulator >= self.fixed_timestep
    }

    /// Consume one physics step.
    pub fn consume_step(&mut self) {
        if self.should_step() {
            self.accumulator -= self.fixed_timestep;
            self.total_time += self.fixed_timestep;
            self.step_count += 1;
        }
    }

    /// Get the number of physics steps needed this frame.
    pub fn steps_needed(&self) -> u32 {
        if !self.enabled {
            return 0;
        }

        let steps = (self.accumulator / self.fixed_timestep) as u32;
        steps.min(self.max_steps)
    }

    /// Reset the physics time state.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
        self.total_time = 0.0;
        self.step_count = 0;
        self.interpolation_alpha = 0.0;
    }
}

/// Physics configuration resource for runtime tuning.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "inspector", derive(bevy::reflect::Reflect))]
pub struct PhysicsConfig {
    /// Enable/disable physics simulation
    pub enabled: bool,
    /// Enable/disable debug rendering
    pub debug_rendering: bool,
    /// Enable/disable performance profiling
    pub profiling: bool,
    /// Physics timestep frequency (Hz)
    pub timestep_hz: f32,
    /// Maximum physics steps per frame
    pub max_steps_per_frame: u32,
    /// Enable/disable continuous collision detection
    pub ccd_enabled: bool,
    /// Gravity vector
    pub gravity: Vec3,
    /// Performance monitoring
    pub performance_monitoring: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_rendering: false,
            profiling: false,
            timestep_hz: 60.0,
            max_steps_per_frame: 4,
            ccd_enabled: false, // Disabled for performance per Oracle's specs
            gravity: Vec3::new(0.0, -9.81, 0.0),
            performance_monitoring: false,
        }
    }
}

/// System to update physics time each frame.
pub fn update_physics_time(mut physics_time: ResMut<PhysicsTime>, time: Res<Time>) {
    physics_time.update(time.delta_secs());
}

/// System to apply physics configuration changes.
pub fn apply_physics_config(
    mut physics_time: ResMut<PhysicsTime>,
    physics_config: Res<PhysicsConfig>,
) {
    if physics_config.is_changed() {
        physics_time.enabled = physics_config.enabled;
        physics_time.fixed_timestep = 1.0 / physics_config.timestep_hz;
        physics_time.max_steps = physics_config.max_steps_per_frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_time_default_creation() {
        let physics_time = PhysicsTime::default();
        assert_eq!(physics_time.fixed_timestep, 1.0 / 60.0);
        assert_eq!(physics_time.accumulator, 0.0);
        assert_eq!(physics_time.max_steps, 4);
        assert_eq!(physics_time.interpolation_alpha, 0.0);
        assert_eq!(physics_time.total_time, 0.0);
        assert_eq!(physics_time.step_count, 0);
        assert!(physics_time.enabled);
    }

    #[test]
    fn physics_time_custom_timestep() {
        let physics_time = PhysicsTime::new(30.0);
        assert_eq!(physics_time.fixed_timestep, 1.0 / 30.0);
    }

    #[test]
    fn physics_time_update() {
        let mut physics_time = PhysicsTime::default();
        let delta_time = 0.016; // ~60 FPS

        physics_time.update(delta_time);
        assert_eq!(physics_time.accumulator, delta_time);
        assert!(physics_time.interpolation_alpha > 0.0);
    }

    #[test]
    fn physics_time_should_step() {
        let mut physics_time = PhysicsTime::default();

        assert!(!physics_time.should_step());

        physics_time.accumulator = physics_time.fixed_timestep + 0.001;
        assert!(physics_time.should_step());
    }

    #[test]
    fn physics_time_consume_step() {
        let mut physics_time = PhysicsTime::default();
        physics_time.accumulator = physics_time.fixed_timestep + 0.001;

        let initial_accumulator = physics_time.accumulator;
        physics_time.consume_step();

        assert_eq!(
            physics_time.accumulator,
            initial_accumulator - physics_time.fixed_timestep
        );
        assert_eq!(physics_time.step_count, 1);
        assert!(physics_time.total_time > 0.0);
    }

    #[test]
    fn physics_time_steps_needed() {
        let mut physics_time = PhysicsTime::default();

        // No steps needed initially
        assert_eq!(physics_time.steps_needed(), 0);

        // One step needed
        physics_time.accumulator = physics_time.fixed_timestep + 0.001;
        assert_eq!(physics_time.steps_needed(), 1);

        // Multiple steps needed but clamped to max
        physics_time.accumulator = physics_time.fixed_timestep * 10.0;
        assert_eq!(physics_time.steps_needed(), physics_time.max_steps);
    }

    #[test]
    fn physics_time_disabled() {
        let mut physics_time = PhysicsTime {
            enabled: false,
            ..Default::default()
        };

        physics_time.update(0.016);
        assert_eq!(physics_time.accumulator, 0.0);
        assert!(!physics_time.should_step());
        assert_eq!(physics_time.steps_needed(), 0);
    }

    #[test]
    fn physics_time_reset() {
        let mut physics_time = PhysicsTime {
            accumulator: 0.5,
            total_time: 10.0,
            step_count: 100,
            interpolation_alpha: 0.7,
            ..Default::default()
        };

        physics_time.reset();

        assert_eq!(physics_time.accumulator, 0.0);
        assert_eq!(physics_time.total_time, 0.0);
        assert_eq!(physics_time.step_count, 0);
        assert_eq!(physics_time.interpolation_alpha, 0.0);
    }

    #[test]
    fn physics_config_default() {
        let config = PhysicsConfig::default();
        assert!(config.enabled);
        assert!(!config.debug_rendering);
        assert!(!config.profiling);
        assert_eq!(config.timestep_hz, 60.0);
        assert_eq!(config.max_steps_per_frame, 4);
        assert!(!config.ccd_enabled);
        assert_eq!(config.gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn physics_time_serialization() {
        let physics_time = PhysicsTime {
            fixed_timestep: 1.0 / 30.0,
            accumulator: 0.1,
            max_steps: 2,
            interpolation_alpha: 0.5,
            total_time: 5.0,
            step_count: 150,
            enabled: true,
        };

        let serialized = serde_json::to_string(&physics_time).unwrap();
        let deserialized: PhysicsTime = serde_json::from_str(&serialized).unwrap();

        assert_eq!(physics_time.fixed_timestep, deserialized.fixed_timestep);
        assert_eq!(physics_time.accumulator, deserialized.accumulator);
        assert_eq!(physics_time.max_steps, deserialized.max_steps);
        assert_eq!(
            physics_time.interpolation_alpha,
            deserialized.interpolation_alpha
        );
        assert_eq!(physics_time.total_time, deserialized.total_time);
        assert_eq!(physics_time.step_count, deserialized.step_count);
        assert_eq!(physics_time.enabled, deserialized.enabled);
    }

    #[test]
    fn physics_config_serialization() {
        let config = PhysicsConfig {
            enabled: false,
            debug_rendering: true,
            profiling: true,
            timestep_hz: 30.0,
            max_steps_per_frame: 2,
            ccd_enabled: true,
            gravity: Vec3::new(0.0, -19.62, 0.0),
            performance_monitoring: true,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: PhysicsConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.debug_rendering, deserialized.debug_rendering);
        assert_eq!(config.profiling, deserialized.profiling);
        assert_eq!(config.timestep_hz, deserialized.timestep_hz);
        assert_eq!(config.max_steps_per_frame, deserialized.max_steps_per_frame);
        assert_eq!(config.ccd_enabled, deserialized.ccd_enabled);
        assert_eq!(config.gravity, deserialized.gravity);
        assert_eq!(
            config.performance_monitoring,
            deserialized.performance_monitoring
        );
    }
}
