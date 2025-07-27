//! Animation asset definitions and loaders for character system
//!
//! This module provides the asset loading infrastructure for character animations,
//! including RON-based configuration files and automatic asset registration.

use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::character::components::{AnimationSet, Locomotion};

/// Animation set configuration loaded from RON files
#[derive(Asset, Debug, Serialize, Deserialize, Reflect)]
pub struct AnimationSetConfig {
    /// Character type identifier
    pub character_type: String,
    /// Animation clip paths mapped to locomotion states
    pub clips: HashMap<String, String>,
    /// Blend weights for each animation
    pub blend_weights: Option<HashMap<String, f32>>,
    /// Animation speeds/multipliers
    pub speeds: Option<HashMap<String, f32>>,
    /// Transition durations between states
    pub transitions: Option<HashMap<String, f32>>,
}

impl Default for AnimationSetConfig {
    fn default() -> Self {
        Self {
            character_type: "default".to_string(),
            clips: HashMap::new(),
            blend_weights: None,
            speeds: None,
            transitions: None,
        }
    }
}

impl AnimationSetConfig {
    /// Convert string keys to Locomotion enum values
    pub fn parse_locomotion(key: &str) -> Option<Locomotion> {
        match key.to_lowercase().as_str() {
            "idle" => Some(Locomotion::Idle),
            "walk" => Some(Locomotion::Walk),
            "run" => Some(Locomotion::Run),
            "sprint" => Some(Locomotion::Sprint),
            "jump" => Some(Locomotion::Jump),
            "fall" => Some(Locomotion::Fall),
            "land" => Some(Locomotion::Land),
            "turn" => Some(Locomotion::Turn),
            _ => None,
        }
    }

    /// Create a default animation set configuration for Mixamo characters
    pub fn default_mixamo() -> Self {
        let mut clips = HashMap::new();
        clips.insert(
            "idle".to_string(),
            "animations/idle.glb#Animation0".to_string(),
        );
        clips.insert(
            "walk".to_string(),
            "animations/walking.glb#Animation0".to_string(),
        );
        clips.insert(
            "run".to_string(),
            "animations/running.glb#Animation0".to_string(),
        );
        clips.insert(
            "sprint".to_string(),
            "animations/sprinting.glb#Animation0".to_string(),
        );
        clips.insert(
            "jump".to_string(),
            "animations/jumping.glb#Animation0".to_string(),
        );

        let mut blend_weights = HashMap::new();
        blend_weights.insert("idle".to_string(), 1.0);
        blend_weights.insert("walk".to_string(), 1.0);
        blend_weights.insert("run".to_string(), 1.0);
        blend_weights.insert("sprint".to_string(), 1.0);
        blend_weights.insert("jump".to_string(), 1.0);

        let mut speeds = HashMap::new();
        speeds.insert("idle".to_string(), 1.0);
        speeds.insert("walk".to_string(), 1.2);
        speeds.insert("run".to_string(), 1.5);
        speeds.insert("sprint".to_string(), 2.0);
        speeds.insert("jump".to_string(), 1.0);

        let mut transitions = HashMap::new();
        transitions.insert("idle".to_string(), 0.3);
        transitions.insert("walk".to_string(), 0.2);
        transitions.insert("run".to_string(), 0.15);
        transitions.insert("sprint".to_string(), 0.1);
        transitions.insert("jump".to_string(), 0.1);

        Self {
            character_type: "mixamo".to_string(),
            clips,
            blend_weights: Some(blend_weights),
            speeds: Some(speeds),
            transitions: Some(transitions),
        }
    }
}

/// Asset loader for animation set configuration files
#[derive(Default)]
pub struct AnimationSetConfigLoader;

impl AssetLoader for AnimationSetConfigLoader {
    type Asset = AnimationSetConfig;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl std::future::Future<Output = Result<Self::Asset, Self::Error>> + Send {
        async move {
            let mut contents = String::new();
            reader.read_to_string(&mut contents).await?;
            let config: AnimationSetConfig = ron::from_str(&contents)?;
            Ok(config)
        }
    }

    fn extensions(&self) -> &[&str] {
        &["animset.ron"]
    }
}

/// System to process loaded animation set configs and create animation sets
pub fn process_animation_set_configs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_sets: ResMut<Assets<AnimationSet>>,
    mut animation_graphs: ResMut<Assets<bevy::animation::graph::AnimationGraph>>,
    config_assets: Res<Assets<AnimationSetConfig>>,
    mut config_events: EventReader<AssetEvent<AnimationSetConfig>>,
) {
    for event in config_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(config) = config_assets.get(*id) {
                    // Create animation graph and set following Oracle's pattern
                    let mut graph = bevy::animation::graph::AnimationGraph::new();
                    let mut animation_set = AnimationSet::new(&config.character_type);

                    // Load animation clips and create graph nodes following Oracle's pattern
                    for (state_str, clip_path) in &config.clips {
                        if let Some(locomotion) = AnimationSetConfig::parse_locomotion(state_str) {
                            let clip_handle: Handle<AnimationClip> = asset_server.load(clip_path);
                            let weight = config
                                .blend_weights
                                .as_ref()
                                .and_then(|w| w.get(state_str))
                                .copied()
                                .unwrap_or(1.0);

                            // Add clip to graph and store node index (Oracle's pattern)
                            let node_index =
                                graph.add_clip(clip_handle.clone(), weight, graph.root);
                            animation_set.set_clip(locomotion, clip_handle);
                            animation_set.set_node_index(locomotion, node_index);
                        }
                    }

                    // Store the animation graph and add handle to animation set
                    let graph_handle = animation_graphs.add(graph);
                    animation_set.graph = graph_handle;

                    // Store the animation set
                    let animation_set_handle = animation_sets.add(animation_set);

                    // Emit event that animation set is ready
                    commands.trigger(AnimationSetReady {
                        handle: animation_set_handle,
                        character_type: config.character_type.clone(),
                    });
                }
            }
            _ => {}
        }
    }
}

/// Event triggered when an animation set is ready for use
#[derive(Event)]
pub struct AnimationSetReady {
    pub handle: Handle<AnimationSet>,
    pub character_type: String,
}

/// Resource for managing character asset references
#[derive(Resource, Default)]
pub struct CharacterAssetRegistry {
    /// Animation sets by character type
    pub animation_sets: HashMap<String, Handle<AnimationSet>>,
    /// Model handles by character type
    pub models: HashMap<String, Handle<Scene>>,
    /// Animation configs by character type
    pub configs: HashMap<String, Handle<AnimationSetConfig>>,
}

impl CharacterAssetRegistry {
    /// Register an animation set for a character type
    pub fn register_animation_set(&mut self, character_type: String, handle: Handle<AnimationSet>) {
        self.animation_sets.insert(character_type, handle);
    }

    /// Get animation set for character type
    pub fn get_animation_set(&self, character_type: &str) -> Option<&Handle<AnimationSet>> {
        self.animation_sets.get(character_type)
    }

    /// Register a model for a character type
    pub fn register_model(&mut self, character_type: String, handle: Handle<Scene>) {
        self.models.insert(character_type, handle);
    }

    /// Get model for character type
    pub fn get_model(&self, character_type: &str) -> Option<&Handle<Scene>> {
        self.models.get(character_type)
    }

    /// Load default Mixamo assets
    pub fn load_mixamo_defaults(&mut self, asset_server: &AssetServer) {
        // Load default animation set config
        let config_handle: Handle<AnimationSetConfig> =
            asset_server.load("animations/mixamo_default.animset.ron");
        self.configs.insert("mixamo".to_string(), config_handle);
    }
}

/// System to handle animation set ready events
pub fn handle_animation_set_ready(
    mut registry: ResMut<CharacterAssetRegistry>,
    mut events: EventReader<AnimationSetReady>,
) {
    for event in events.read() {
        registry.register_animation_set(event.character_type.clone(), event.handle.clone());
    }
}

/// Plugin for character asset loading
pub struct CharacterAssetPlugin;

impl Plugin for CharacterAssetPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register asset types
            .init_asset::<AnimationSetConfig>()
            .register_asset_loader(AnimationSetConfigLoader)
            .register_type::<AnimationSetConfig>()
            // Register resources
            .init_resource::<CharacterAssetRegistry>()
            // Register events
            .add_event::<AnimationSetReady>()
            // Add systems
            .add_systems(
                Update,
                (process_animation_set_configs, handle_animation_set_ready),
            );
    }
}
