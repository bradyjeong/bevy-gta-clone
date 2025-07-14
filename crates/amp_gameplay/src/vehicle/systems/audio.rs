//! Vehicle audio systems

use crate::audio::VehicleEngineAudioEvent;
use crate::vehicle::components::*;
use bevy::prelude::*;
use config_core::{AudioConfig, ConfigLoader};

/// Update vehicle audio based on physics state
#[allow(clippy::type_complexity)]
pub fn update_vehicle_audio(
    mut query: Query<
        (
            Entity,
            &mut VehicleAudio,
            &Engine,
            &GlobalTransform,
            &VehicleInput,
        ),
        (With<Vehicle>, With<crate::audio::components::EngineAudio>),
    >,
    mut audio_events: EventWriter<VehicleEngineAudioEvent>,
) {
    // Load audio config for parameter lookup
    let audio_config = ConfigLoader::new()
        .load_with_merge::<AudioConfig>()
        .unwrap_or_default();
    for (entity, mut audio, engine, transform, input) in query.iter_mut() {
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
            // Calculate tire screech volume based on velocity
            let speed = 0.0_f32; // TODO: Get speed from physics system
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
