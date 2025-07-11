//! Vehicle audio systems

use crate::audio::VehicleEngineAudioEvent;
use crate::vehicle::components::*;
use bevy::prelude::*;

/// Update vehicle audio based on physics state
pub fn update_vehicle_audio(
    mut query: Query<
        (
            Entity,
            &mut VehicleAudio,
            &Engine,
            &GlobalTransform,
            &VehicleInput,
        ),
        With<Vehicle>,
    >,
    mut audio_events: EventWriter<VehicleEngineAudioEvent>,
) {
    for (entity, mut audio, engine, transform, input) in query.iter_mut() {
        if audio.engine_sound_enabled {
            // Calculate engine volume based on RPM only if engine is running
            if engine.rpm > 0.0 {
                let rpm_ratio = engine.rpm / engine.max_rpm;
                audio.engine_volume = (rpm_ratio * 0.8 + 0.2).min(1.0);

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
                audio.tire_screech_volume = velocity_ratio * 0.5;
            }
        }
    }
}
