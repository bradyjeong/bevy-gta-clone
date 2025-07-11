//! Simple audio implementation for Sprint 3 completion

use crate::VehicleEngine;
use crate::audio::components::*;
use bevy::prelude::*;

/// Simple audio event for vehicle engine
#[derive(Event, Debug, Clone)]
pub struct VehicleEngineAudioEvent {
    pub vehicle_entity: Entity,
    pub rpm: f32,
    pub throttle: f32,
    pub load: f32,
    pub gear: i32,
    pub position: Vec3,
}

/// Simple audio settings
#[derive(Resource, Debug, Clone)]
pub struct GameplayAudioSettings {
    pub master_volume: f32,
    pub engine_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub environment_volume: f32,
    pub ui_volume: f32,
}

impl Default for GameplayAudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            engine_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.9,
            environment_volume: 0.6,
            ui_volume: 0.8,
        }
    }
}

/// Setup simple audio systems
pub fn setup_audio_systems(mut commands: Commands) {
    // Create global audio entities
    commands.spawn((MusicSystem::default(), Name::new("Music System")));

    commands.spawn((
        EnvironmentalAudio::default(),
        Name::new("Environmental Audio"),
    ));
}

/// Update engine audio based on RPM and throttle
pub fn update_engine_audio(
    mut engine_query: Query<(&mut EngineAudio, &VehicleEngine, &GlobalTransform)>,
    audio_settings: Res<GameplayAudioSettings>,
    time: Res<Time>,
) {
    for (mut engine_audio, vehicle_engine, _transform) in engine_query.iter_mut() {
        // Calculate target pitch based on RPM
        let target_pitch =
            engine_audio.base_pitch + (vehicle_engine.rpm * engine_audio.rpm_pitch_factor);

        // Calculate target volume based on throttle
        let _target_volume = engine_audio.throttle_volume_factor
            * (0.3 + vehicle_engine.throttle * 0.7)
            * audio_settings.engine_volume
            * audio_settings.master_volume;

        // Smoothly interpolate pitch and volume
        let delta = time.delta_secs();
        engine_audio.pitch_multiplier =
            lerp(engine_audio.pitch_multiplier, target_pitch, delta * 2.0);

        // Update engine state
        engine_audio.is_playing = vehicle_engine.throttle > 0.0;
    }
}

/// Update environmental audio
pub fn update_environmental_audio(mut env_query: Query<&mut EnvironmentalAudio>, time: Res<Time>) {
    for mut env_audio in env_query.iter_mut() {
        // Update environmental audio based on game state
        env_audio.wind_intensity = 0.2 + 0.1 * (time.elapsed_secs() * 0.5).sin();
        env_audio.traffic_density = 0.3;
    }
}

/// Update music system
pub fn update_music_system(mut music_query: Query<&mut MusicSystem>) {
    for _music_system in music_query.iter_mut() {
        // Handle music playback logic
        // Placeholder for now
    }
}

/// Handle vehicle engine audio events
pub fn handle_vehicle_engine_audio_events(
    mut engine_events: EventReader<VehicleEngineAudioEvent>,
    mut engine_query: Query<&mut EngineAudio>,
    audio_settings: Res<GameplayAudioSettings>,
) {
    for event in engine_events.read() {
        if let Ok(mut engine_audio) = engine_query.get_mut(event.vehicle_entity) {
            // Update engine audio based on event data
            let target_pitch =
                engine_audio.base_pitch + (event.rpm * engine_audio.rpm_pitch_factor);
            let _target_volume = engine_audio.throttle_volume_factor
                * (0.3 + event.throttle * 0.7)
                * audio_settings.engine_volume
                * audio_settings.master_volume;

            engine_audio.pitch_multiplier = target_pitch;
        }
    }
}

/// Linear interpolation helper
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}
