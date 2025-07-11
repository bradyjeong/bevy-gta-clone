//! Audio resources for managing channels and assets

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::collections::HashMap;

/// Audio channels resource for managing different audio types
#[derive(Resource, Debug, Clone)]
pub struct AudioChannels {
    /// Engine audio channel
    pub engine: Handle<AudioChannel<MainTrack>>,
    /// Sound effects channel
    pub sfx: Handle<AudioChannel<MainTrack>>,
    /// Music channel
    pub music: Handle<AudioChannel<MainTrack>>,
    /// Environmental audio channel
    pub environment: Handle<AudioChannel<MainTrack>>,
    /// UI sound channel
    pub ui: Handle<AudioChannel<MainTrack>>,
}

impl AudioChannels {
    pub fn new(audio: &Res<Audio>) -> Self {
        Self {
            engine: audio.create_channel(),
            sfx: audio.create_channel(),
            music: audio.create_channel(),
            environment: audio.create_channel(),
            ui: audio.create_channel(),
        }
    }
}

/// Audio assets resource for managing pre-loaded sounds
#[derive(Resource, Debug, Default)]
pub struct AudioAssets {
    /// Engine sound assets
    pub engine_sounds: HashMap<String, Handle<bevy_kira_audio::AudioSource>>,
    /// Environmental sound assets
    pub environmental_sounds: HashMap<String, Handle<bevy_kira_audio::AudioSource>>,
    /// Music tracks
    pub music_tracks: HashMap<String, Handle<bevy_kira_audio::AudioSource>>,
    /// Sound effects
    pub sound_effects: HashMap<String, Handle<bevy_kira_audio::AudioSource>>,
}

impl AudioAssets {
    pub fn new() -> Self {
        Self {
            engine_sounds: HashMap::new(),
            environmental_sounds: HashMap::new(),
            music_tracks: HashMap::new(),
            sound_effects: HashMap::new(),
        }
    }

    pub fn load_engine_sound(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.engine_sounds.insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_environmental_sound(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.environmental_sounds.insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_music_track(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.music_tracks.insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_sound_effect(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.sound_effects.insert(name.to_string(), asset_server.load(path));
    }
}

/// Vehicle engine audio event
#[derive(Event, Debug, Clone)]
pub struct VehicleEngineAudioEvent {
    /// Vehicle entity
    pub vehicle_entity: Entity,
    /// Current RPM
    pub rpm: f32,
    /// Throttle input (0.0 to 1.0)
    pub throttle: f32,
    /// Engine load (0.0 to 1.0)
    pub load: f32,
    /// Gear number
    pub gear: i32,
    /// Vehicle position for 3D audio
    pub position: Vec3,
}

/// Audio settings resource
#[derive(Resource, Debug, Clone)]
pub struct GameplayAudioSettings {
    /// Master volume
    pub master_volume: f32,
    /// Engine volume
    pub engine_volume: f32,
    /// Music volume
    pub music_volume: f32,
    /// SFX volume
    pub sfx_volume: f32,
    /// Environmental volume
    pub environment_volume: f32,
    /// UI volume
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
