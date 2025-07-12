//! Audio resources for managing channels and assets

use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioSource as KiraAudioSource;
use std::collections::HashMap;

/// Audio assets resource for managing pre-loaded sounds
#[derive(Resource, Debug, Default)]
pub struct AudioAssets {
    /// Engine sound assets
    pub engine_sounds: HashMap<String, Handle<KiraAudioSource>>,
    /// Environmental sound assets
    pub environmental_sounds: HashMap<String, Handle<KiraAudioSource>>,
    /// Music tracks
    pub music_tracks: HashMap<String, Handle<KiraAudioSource>>,
    /// Sound effects
    pub sound_effects: HashMap<String, Handle<KiraAudioSource>>,
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
        self.engine_sounds
            .insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_environmental_sound(
        &mut self,
        name: &str,
        path: &str,
        asset_server: &Res<AssetServer>,
    ) {
        self.environmental_sounds
            .insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_music_track(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.music_tracks
            .insert(name.to_string(), asset_server.load(path));
    }

    pub fn load_sound_effect(&mut self, name: &str, path: &str, asset_server: &Res<AssetServer>) {
        self.sound_effects
            .insert(name.to_string(), asset_server.load(path));
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
pub struct AudioSettings {
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

/// Alias for backwards compatibility
pub type GameplayAudioSettings = AudioSettings;

impl Default for GameplayAudioSettings {
    fn default() -> Self {
        // Load from config if available, otherwise use fallback values
        if let Ok(loader) =
            config_core::ConfigLoader::new().load_with_merge::<config_core::AudioConfig>()
        {
            Self {
                master_volume: loader.master_volume,
                engine_volume: loader.engine_volume,
                music_volume: loader.music_volume,
                sfx_volume: loader.sfx_volume,
                environment_volume: loader.environment_volume,
                ui_volume: loader.ui_volume,
            }
        } else {
            // Fallback to hardcoded values if config loading fails
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
}
