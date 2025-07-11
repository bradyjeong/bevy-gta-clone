//! Audio systems for gameplay
//!
//! Provides advanced audio integration with bevy_kira_audio including:
//! - Engine sound simulation
//! - Environmental audio
//! - Music system
//! - Sound effects management

pub mod components;
pub mod config;
pub mod simple_audio;

pub use components::*;
pub use simple_audio::*;

use bevy::prelude::*;

/// Plugin for audio systems
#[derive(Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<VehicleEngineAudioEvent>()
            .init_resource::<GameplayAudioSettings>()
            .add_systems(Startup, setup_audio_systems)
            .add_systems(
                Update,
                (
                    update_engine_audio,
                    update_environmental_audio,
                    update_music_system,
                    handle_vehicle_engine_audio_events,
                ),
            )
            .register_type::<components::AudioSource>()
            .register_type::<components::AudioCategory>()
            .register_type::<components::EngineAudio>()
            .register_type::<components::EnvironmentalAudio>()
            .register_type::<components::MusicSystem>()
            .register_type::<components::SoundEffect>();
    }
}
