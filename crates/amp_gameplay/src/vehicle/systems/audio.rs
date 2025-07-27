//! Vehicle audio systems

use crate::audio::VehicleEngineAudioEvent;
use crate::vehicle::components::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use config_core::{AudioConfig, ConfigLoader};

/// Cached physics data for audio systems
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CachedVehiclePhysics {
    pub velocity: Vec3,
    pub speed: f32,
    pub last_updated: f64,
}

/// System to ensure vehicles have cached physics component
pub fn ensure_vehicle_cached_physics(
    mut commands: Commands,
    vehicles_without_cache: Query<Entity, (With<Vehicle>, Without<CachedVehiclePhysics>)>,
) {
    for entity in vehicles_without_cache.iter() {
        commands
            .entity(entity)
            .insert(CachedVehiclePhysics::default());
    }
}

/// System to cache physics data from FixedUpdate for use in Update systems
pub fn cache_vehicle_physics_for_audio(
    mut query: Query<(&mut CachedVehiclePhysics, &Velocity), With<Vehicle>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();
    for (mut cached, velocity) in query.iter_mut() {
        cached.velocity = velocity.linvel;
        cached.speed = velocity.linvel.length();
        cached.last_updated = current_time;
    }
}

/// Update vehicle audio based on cached physics state
#[allow(clippy::type_complexity)]
pub fn update_vehicle_audio(
    mut query: Query<
        (
            Entity,
            &mut VehicleAudio,
            &Engine,
            &GlobalTransform,
            &VehicleInput,
            Option<&CachedVehiclePhysics>,
        ),
        (With<Vehicle>, With<crate::audio::components::EngineAudio>),
    >,
    mut audio_events: EventWriter<VehicleEngineAudioEvent>,
) {
    // Load audio config for parameter lookup
    let audio_config = ConfigLoader::new()
        .load_with_merge::<AudioConfig>()
        .unwrap_or_default();
    for (entity, mut audio, engine, transform, input, cached_physics) in query.iter_mut() {
        if audio.engine_sound_enabled {
            // Calculate engine volume based on RPM only if engine is running
            if engine.rpm > 0.0 {
                let rpm_ratio = engine.rpm / engine.max_rpm;
                // Use config values instead of hard-coded constants
                audio.engine_volume = (rpm_ratio * audio_config.engine.rpm_scaling
                    + audio_config.engine.min_volume)
                    .min(audio_config.engine.max_volume);

                // Emit engine audio event
                audio_events.write(VehicleEngineAudioEvent {
                    vehicle_entity: entity,
                    rpm: engine.rpm,
                    throttle: input.throttle,
                    load: 0.5_f32, // Placeholder - engine.load not available in current Engine struct
                    gear: 1, // Placeholder - engine.gear not available in current Engine struct
                    position: transform.translation(),
                });
            }
        }

        if audio.tire_screech_enabled {
            // Calculate tire screech volume based on cached velocity from FixedUpdate physics
            let speed = cached_physics.map(|p| p.speed).unwrap_or(0.0);
            let velocity_ratio = (speed / 50.0).min(1.0); // Assume 50 m/s max for screech

            // Only update tire screech volume if there's actual velocity
            if speed > 0.0 {
                // Use config value instead of hard-coded constant
                audio.tire_screech_volume =
                    velocity_ratio * audio_config.vehicle.tire_screech_scaling;
            }
        }
    }
}
