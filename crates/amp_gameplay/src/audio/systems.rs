//! Audio systems for managing sounds

use crate::audio::{components::*, resources::*};
use crate::vehicle::components::Engine;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Setup audio systems
pub fn setup_audio_systems(mut commands: Commands) {
    // Create global audio entities
    commands.spawn((MusicSystem::default(), Name::new("Music System")));

    commands.spawn((
        EnvironmentalAudio::default(),
        Name::new("Environmental Audio"),
    ));
}

/// Load audio assets
pub fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut audio_assets = AudioAssets::new();

    // Load engine sounds (placeholder assets)
    audio_assets.load_engine_sound("default", "audio/engine_default.ogg", &asset_server);
    audio_assets.load_engine_sound("idle", "audio/engine_idle.ogg", &asset_server);
    audio_assets.load_engine_sound("rev", "audio/engine_rev.ogg", &asset_server);

    // Load environmental sounds
    audio_assets.load_environmental_sound("wind", "audio/wind.ogg", &asset_server);
    audio_assets.load_environmental_sound("rain", "audio/rain.ogg", &asset_server);
    audio_assets.load_environmental_sound("traffic", "audio/traffic.ogg", &asset_server);

    // Load music tracks
    audio_assets.load_music_track("menu", "audio/music/menu.ogg", &asset_server);
    audio_assets.load_music_track("gameplay", "audio/music/gameplay.ogg", &asset_server);

    // Load sound effects
    audio_assets.load_sound_effect("brake", "audio/sfx/brake.ogg", &asset_server);
    audio_assets.load_sound_effect("gear_shift", "audio/sfx/gear_shift.ogg", &asset_server);
    audio_assets.load_sound_effect("collision", "audio/sfx/collision.ogg", &asset_server);

    commands.insert_resource(audio_assets);
}

/// Update engine audio based on RPM and throttle
pub fn update_engine_audio(
    mut engine_query: Query<(&mut EngineAudio, &Engine, &GlobalTransform)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    audio_settings: Res<GameplayAudioSettings>,
    time: Res<Time>,
) {
    for (mut engine_audio, vehicle_engine, _transform) in engine_query.iter_mut() {
        // Calculate target pitch based on RPM
        let target_pitch =
            engine_audio.base_pitch + (vehicle_engine.rpm * engine_audio.rpm_pitch_factor);

        // Calculate target volume based on throttle
        let target_volume = engine_audio.throttle_volume_factor
            * (0.3 + vehicle_engine.throttle * 0.7)
            * audio_settings.engine_volume
            * audio_settings.master_volume;

        // Smoothly interpolate pitch and volume
        let delta = time.delta_secs();
        engine_audio.pitch_multiplier =
            lerp(engine_audio.pitch_multiplier, target_pitch, delta * 2.0);

        // Play engine sound with proper volume and pitch
        if vehicle_engine.throttle > 0.0 && !engine_audio.is_playing {
            if let Some(sound_handle) = audio_assets.engine_sounds.get("default") {
                audio
                    .play(sound_handle.clone())
                    .looped()
                    .with_volume(target_volume as f64)
                    .with_playback_rate(engine_audio.pitch_multiplier as f64);
                engine_audio.is_playing = true;
            }
        }

        // Update engine state
        engine_audio.is_playing = vehicle_engine.throttle > 0.0;
    }
}

/// Update environmental audio
pub fn update_environmental_audio(
    mut env_query: Query<&mut EnvironmentalAudio>,
    time: Res<Time>,
    _audio: Res<Audio>,
) {
    for mut env_audio in env_query.iter_mut() {
        // Update environmental audio based on game state
        // This would typically be driven by weather system, traffic system, etc.

        // For now, just simulate some basic ambient sound
        env_audio.wind_intensity = 0.2 + 0.1 * (time.elapsed_secs() * 0.5).sin();
        env_audio.traffic_density = 0.3;
    }
}

/// Update music system
pub fn update_music_system(
    mut music_query: Query<&mut MusicSystem>,
    _audio: Res<Audio>,
    _time: Res<Time>,
) {
    for _music_system in music_query.iter_mut() {
        // Handle music playback logic
        // Placeholder for now
    }
}

/// Handle vehicle engine audio events
pub fn handle_vehicle_engine_audio_events(
    mut engine_events: EventReader<VehicleEngineAudioEvent>,
    mut engine_query: Query<&mut EngineAudio>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    audio_settings: Res<GameplayAudioSettings>,
) {
    for event in engine_events.read() {
        if let Ok(mut engine_audio) = engine_query.get_mut(event.vehicle_entity) {
            // Update engine audio based on event data
            let target_pitch =
                engine_audio.base_pitch + (event.rpm * engine_audio.rpm_pitch_factor);
            let target_volume = engine_audio.throttle_volume_factor
                * (0.3 + event.throttle * 0.7)
                * audio_settings.engine_volume
                * audio_settings.master_volume;

            engine_audio.pitch_multiplier = target_pitch;

            // Play appropriate engine sound based on load and gear
            let sound_key = match event.load {
                load if load > 0.8 => "rev",
                load if load < 0.2 => "idle",
                _ => "default",
            };

            if let Some(sound_handle) = audio_assets.engine_sounds.get(sound_key) {
                audio
                    .play(sound_handle.clone())
                    .with_volume(target_volume as f64)
                    .with_playback_rate(engine_audio.pitch_multiplier as f64);
            }
        }
    }
}

/// Linear interpolation helper
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}
