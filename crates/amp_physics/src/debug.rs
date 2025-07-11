//! Debug rendering for physics visualization.
//!
//! This module provides visual debugging tools for physics simulation including
//! suspension rays, force vectors, contact points, and performance metrics.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time::PhysicsConfig;
use crate::{Engine, SuspensionRay, Transmission, WheelState};

/// Debug rendering configuration.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "inspector", derive(bevy::reflect::Reflect))]
pub struct DebugConfig {
    /// Enable/disable all debug rendering
    pub enabled: bool,
    /// Show suspension rays
    pub show_suspension_rays: bool,
    /// Show force vectors
    pub show_force_vectors: bool,
    /// Show wheel contact points
    pub show_contact_points: bool,
    /// Show engine/transmission state
    pub show_engine_state: bool,
    /// Debug ray color
    pub ray_color: Color,
    /// Force vector color
    pub force_color: Color,
    /// Contact point color
    pub contact_color: Color,
    /// Force vector scale multiplier
    pub force_scale: f32,
    /// Ray width
    pub ray_width: f32,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_suspension_rays: true,
            show_force_vectors: true,
            show_contact_points: true,
            show_engine_state: true,
            ray_color: Color::srgb(0.0, 1.0, 0.0),     // Green
            force_color: Color::srgb(1.0, 0.0, 0.0),   // Red
            contact_color: Color::srgb(0.0, 0.0, 1.0), // Blue
            force_scale: 0.001,                        // Scale forces for visualization
            ray_width: 2.0,
        }
    }
}

/// Component for debug line rendering.
#[derive(Component, Debug)]
pub struct DebugLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Color,
    pub width: f32,
}

/// Component for debug text rendering.
#[derive(Component, Debug)]
pub struct DebugText {
    pub text: String,
    pub position: Vec3,
    pub color: Color,
    pub size: f32,
}

/// Performance metrics for physics debug display.
#[derive(Resource, Debug, Default)]
pub struct PhysicsPerformanceMetrics {
    /// Last frame's physics step count
    pub last_frame_steps: u32,
    /// Physics CPU time (ms)
    pub physics_cpu_time: f32,
    /// Suspension system time (ms)
    pub suspension_time: f32,
    /// Drivetrain system time (ms)
    pub drivetrain_time: f32,
    /// Total physics entities
    pub total_physics_entities: u32,
    /// Active vehicles
    pub active_vehicles: u32,
    /// Average frame time (ms)
    pub average_frame_time: f32,
    /// FPS counter
    pub fps: f32,
}

/// System to render debug lines for suspension rays.
pub fn render_suspension_debug(
    mut commands: Commands,
    debug_config: Res<DebugConfig>,
    physics_config: Res<PhysicsConfig>,
    suspension_query: Query<(&SuspensionRay, &WheelState, &GlobalTransform)>,
    existing_debug_lines: Query<Entity, With<DebugLine>>,
) {
    // Clear existing debug lines
    for entity in existing_debug_lines.iter() {
        commands.entity(entity).despawn();
    }

    if !debug_config.enabled
        || !physics_config.debug_rendering
        || !debug_config.show_suspension_rays
    {
        return;
    }

    for (suspension_ray, wheel_state, global_transform) in suspension_query.iter() {
        let ray_start = global_transform.translation()
            + global_transform.rotation() * suspension_ray.ray_origin;
        let ray_direction = global_transform.rotation() * suspension_ray.ray_direction;

        // Draw suspension ray
        let ray_end = if let Some(hit_distance) = suspension_ray.hit_distance {
            ray_start + ray_direction * hit_distance
        } else {
            ray_start + ray_direction * suspension_ray.cast_distance
        };

        let ray_color = if wheel_state.in_contact {
            debug_config.contact_color
        } else {
            debug_config.ray_color
        };

        commands.spawn(DebugLine {
            start: ray_start,
            end: ray_end,
            color: ray_color,
            width: debug_config.ray_width,
        });

        // Draw contact point
        if debug_config.show_contact_points {
            if let Some(contact_point) = suspension_ray.hit_point {
                commands.spawn(DebugLine {
                    start: contact_point - Vec3::X * 0.1,
                    end: contact_point + Vec3::X * 0.1,
                    color: debug_config.contact_color,
                    width: debug_config.ray_width * 2.0,
                });
                commands.spawn(DebugLine {
                    start: contact_point - Vec3::Z * 0.1,
                    end: contact_point + Vec3::Z * 0.1,
                    color: debug_config.contact_color,
                    width: debug_config.ray_width * 2.0,
                });
            }
        }

        // Draw force vectors
        if debug_config.show_force_vectors && wheel_state.in_contact {
            let force_magnitude = wheel_state.contact_force * debug_config.force_scale;
            let force_vector = Vec3::Y * force_magnitude;

            if let Some(contact_point) = suspension_ray.hit_point {
                commands.spawn(DebugLine {
                    start: contact_point,
                    end: contact_point + force_vector,
                    color: debug_config.force_color,
                    width: debug_config.ray_width * 1.5,
                });
            }
        }
    }
}

/// System to render debug text for engine/transmission state.
pub fn render_engine_debug(
    mut commands: Commands,
    debug_config: Res<DebugConfig>,
    physics_config: Res<PhysicsConfig>,
    engine_query: Query<&Engine>,
    transmission_query: Query<&Transmission>,
    existing_debug_text: Query<Entity, With<DebugText>>,
) {
    // Clear existing debug text
    for entity in existing_debug_text.iter() {
        commands.entity(entity).despawn();
    }

    if !debug_config.enabled || !physics_config.debug_rendering || !debug_config.show_engine_state {
        return;
    }

    let mut debug_y = 50.0;
    for engine in engine_query.iter() {
        let engine_info = format!(
            "Engine: RPM={:.0}, Throttle={:.2}, Torque={:.1}Nm",
            engine.rpm, engine.throttle, engine.torque
        );

        commands.spawn(DebugText {
            text: engine_info,
            position: Vec3::new(10.0, debug_y, 0.0),
            color: Color::WHITE,
            size: 16.0,
        });

        debug_y += 20.0;
    }

    for transmission in transmission_query.iter() {
        let current_ratio = if transmission.current_gear > 0
            && transmission.current_gear <= transmission.gear_ratios.len() as i32
        {
            transmission.gear_ratios[transmission.current_gear as usize - 1]
        } else {
            0.0
        };

        let transmission_info = format!(
            "Transmission: Gear={}, Ratio={:.2}",
            transmission.current_gear, current_ratio
        );

        commands.spawn(DebugText {
            text: transmission_info,
            position: Vec3::new(10.0, debug_y, 0.0),
            color: Color::WHITE,
            size: 16.0,
        });

        debug_y += 20.0;
    }
}

/// System to update performance metrics.
pub fn update_performance_metrics(
    mut metrics: ResMut<PhysicsPerformanceMetrics>,
    physics_time: Res<crate::time::PhysicsTime>,
    suspension_query: Query<&SuspensionRay>,
    engine_query: Query<&Engine>,
    time: Res<Time>,
) {
    metrics.last_frame_steps = physics_time.steps_needed();
    metrics.total_physics_entities = suspension_query.iter().count() as u32;
    metrics.active_vehicles = engine_query.iter().count() as u32;

    // Calculate FPS
    let frame_time = time.delta_secs();
    if frame_time > 0.0 {
        metrics.fps = 1.0 / frame_time;
    }

    // Update average frame time (simple moving average)
    metrics.average_frame_time = metrics.average_frame_time * 0.9 + frame_time * 1000.0 * 0.1;
}

/// System to render performance metrics.
pub fn render_performance_debug(
    mut commands: Commands,
    debug_config: Res<DebugConfig>,
    physics_config: Res<PhysicsConfig>,
    metrics: Res<PhysicsPerformanceMetrics>,
    existing_debug_text: Query<Entity, With<DebugText>>,
) {
    if !debug_config.enabled || !physics_config.performance_monitoring {
        return;
    }

    // Clear existing performance debug text
    for entity in existing_debug_text.iter() {
        commands.entity(entity).despawn();
    }

    let performance_info = [
        format!("FPS: {:.1}", metrics.fps),
        format!("Frame Time: {:.2}ms", metrics.average_frame_time),
        format!("Physics Steps: {}", metrics.last_frame_steps),
        format!("Physics Entities: {}", metrics.total_physics_entities),
        format!("Active Vehicles: {}", metrics.active_vehicles),
        format!("Physics CPU: {:.2}ms", metrics.physics_cpu_time),
        format!("Suspension: {:.2}ms", metrics.suspension_time),
        format!("Drivetrain: {:.2}ms", metrics.drivetrain_time),
    ];

    for (i, info) in performance_info.iter().enumerate() {
        commands.spawn(DebugText {
            text: info.clone(),
            position: Vec3::new(10.0, 300.0 + i as f32 * 20.0, 0.0),
            color: Color::srgb(1.0, 1.0, 0.0),
            size: 14.0,
        });
    }
}

/// Debug plugin for physics visualization.
pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConfig>()
            .init_resource::<PhysicsPerformanceMetrics>()
            .add_systems(
                PostUpdate,
                (
                    update_performance_metrics,
                    render_suspension_debug,
                    render_engine_debug,
                    render_performance_debug,
                )
                    .chain()
                    .after(crate::suspension::vehicle_suspension_system),
            );

        #[cfg(feature = "inspector")]
        {
            app.register_type::<DebugConfig>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_config_default() {
        let config = DebugConfig::default();
        assert!(!config.enabled);
        assert!(config.show_suspension_rays);
        assert!(config.show_force_vectors);
        assert!(config.show_contact_points);
        assert!(config.show_engine_state);
        assert_eq!(config.force_scale, 0.001);
        assert_eq!(config.ray_width, 2.0);
    }

    #[test]
    fn debug_line_creation() {
        let debug_line = DebugLine {
            start: Vec3::ZERO,
            end: Vec3::Y,
            color: Color::srgb(1.0, 0.0, 0.0),
            width: 1.0,
        };

        assert_eq!(debug_line.start, Vec3::ZERO);
        assert_eq!(debug_line.end, Vec3::Y);
        assert_eq!(debug_line.color, Color::srgb(1.0, 0.0, 0.0));
        assert_eq!(debug_line.width, 1.0);
    }

    #[test]
    fn debug_text_creation() {
        let debug_text = DebugText {
            text: "Test".to_string(),
            position: Vec3::new(10.0, 20.0, 0.0),
            color: Color::WHITE,
            size: 12.0,
        };

        assert_eq!(debug_text.text, "Test");
        assert_eq!(debug_text.position, Vec3::new(10.0, 20.0, 0.0));
        assert_eq!(debug_text.color, Color::WHITE);
        assert_eq!(debug_text.size, 12.0);
    }

    #[test]
    fn performance_metrics_default() {
        let metrics = PhysicsPerformanceMetrics::default();
        assert_eq!(metrics.last_frame_steps, 0);
        assert_eq!(metrics.physics_cpu_time, 0.0);
        assert_eq!(metrics.suspension_time, 0.0);
        assert_eq!(metrics.drivetrain_time, 0.0);
        assert_eq!(metrics.total_physics_entities, 0);
        assert_eq!(metrics.active_vehicles, 0);
        assert_eq!(metrics.average_frame_time, 0.0);
        assert_eq!(metrics.fps, 0.0);
    }

    #[test]
    fn debug_config_serialization() {
        let config = DebugConfig {
            enabled: true,
            show_suspension_rays: false,
            show_force_vectors: true,
            show_contact_points: false,
            show_engine_state: true,
            ray_color: Color::srgb(1.0, 0.0, 0.0),
            force_color: Color::srgb(0.0, 1.0, 0.0),
            contact_color: Color::srgb(0.0, 0.0, 1.0),
            force_scale: 0.002,
            ray_width: 3.0,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: DebugConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(
            config.show_suspension_rays,
            deserialized.show_suspension_rays
        );
        assert_eq!(config.show_force_vectors, deserialized.show_force_vectors);
        assert_eq!(config.show_contact_points, deserialized.show_contact_points);
        assert_eq!(config.show_engine_state, deserialized.show_engine_state);
        assert_eq!(config.force_scale, deserialized.force_scale);
        assert_eq!(config.ray_width, deserialized.ray_width);
    }
}
