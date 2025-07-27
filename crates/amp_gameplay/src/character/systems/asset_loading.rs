//! Asset loading systems for Mixamo characters
//!
//! This module handles loading glTF models, extracting skeleton information,
//! and setting up animation systems for Mixamo characters.

use bevy::animation::{graph::AnimationGraphHandle, AnimationPlayer};
use bevy::gltf::{Gltf, GltfNode};
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use std::collections::HashMap;

use crate::character::components::*;

/// Resource for tracking asset loading state
#[derive(Resource, Default)]
pub struct AssetLoadingState {
    /// Characters currently being loaded
    pub loading_characters: HashMap<Entity, Handle<Gltf>>,
    /// Animation sets being loaded from files
    pub loading_animation_sets: HashMap<Handle<AnimationSet>, String>,
}

/// Component to request character asset loading
#[derive(Component)]
pub struct LoadCharacterAsset {
    /// Path to the glTF file
    pub gltf_path: String,
    /// Character type identifier
    pub character_type: String,
    /// Scale to apply to the loaded model
    pub scale: f32,
}

impl LoadCharacterAsset {
    pub fn new(gltf_path: impl Into<String>, character_type: impl Into<String>) -> Self {
        Self {
            gltf_path: gltf_path.into(),
            character_type: character_type.into(),
            scale: 1.0,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// System to handle character asset loading requests
pub fn handle_character_loading_requests(
    mut commands: Commands,
    mut loading_state: ResMut<AssetLoadingState>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &LoadCharacterAsset), Added<LoadCharacterAsset>>,
) {
    for (entity, load_request) in query.iter() {
        // Start loading the glTF asset
        let gltf_handle: Handle<Gltf> = asset_server.load(&load_request.gltf_path);

        // Track this loading operation
        loading_state
            .loading_characters
            .insert(entity, gltf_handle.clone());

        // Add loading component to track progress
        commands.entity(entity).insert(CharacterAssetLoading {
            gltf_handle,
            character_type: load_request.character_type.clone(),
            scale: load_request.scale,
            stage: LoadingStage::LoadingGltf,
            scene_instance: None,
            skeleton_entity: None,
        });

        // Remove the request component
        commands.entity(entity).remove::<LoadCharacterAsset>();
    }
}

/// Component to track character loading progress
#[derive(Component)]
pub struct CharacterAssetLoading {
    pub gltf_handle: Handle<Gltf>,
    pub character_type: String,
    pub scale: f32,
    pub stage: LoadingStage,
    pub scene_instance: Option<Entity>,
    pub skeleton_entity: Option<Entity>,
}

/// Loading stages for character assets
#[derive(Debug, PartialEq)]
pub enum LoadingStage {
    LoadingGltf,
    WaitingForScene,
    ProcessingSkeleton,
    SettingUpAnimation,
    Complete,
}

/// System to process loaded glTF assets and build character rigs
pub fn process_loaded_characters(
    mut commands: Commands,
    mut loading_query: Query<(Entity, &mut CharacterAssetLoading)>,
    character_animations_query: Query<&CharacterAnimations>,
    locomotion_query: Query<&LocomotionState>,
    velocity_query: Query<&Velocity>,
    asset_server: Res<AssetServer>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    animation_sets: Res<Assets<AnimationSet>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    scene_instance_query: Query<&SceneInstance>,
    scene_spawner: Res<SceneSpawner>,
) {
    for (entity, mut loading) in loading_query.iter_mut() {
        debug!(
            "🔧 Processing character loading stage {:?} for entity {:?}",
            loading.stage, entity
        );
        match loading.stage {
            LoadingStage::LoadingGltf => {
                // Check if glTF is loaded
                if let Some(gltf) = gltf_assets.get(&loading.gltf_handle) {
                    // Spawn the glTF scene
                    if let Some(scene) = gltf.scenes.first() {
                        let scene_entity = commands.spawn(SceneRoot(scene.clone())).id();
                        commands.entity(entity).add_child(scene_entity);
                        loading.scene_instance = Some(scene_entity);
                        loading.stage = LoadingStage::WaitingForScene;
                    }
                }
            }
            LoadingStage::WaitingForScene => {
                // Wait for scene to be fully instantiated
                if let Some(scene_entity) = loading.scene_instance {
                    if let Ok(scene_instance) = scene_instance_query.get(scene_entity) {
                        if scene_spawner.instance_is_ready(**scene_instance) {
                            loading.stage = LoadingStage::ProcessingSkeleton;
                        }
                    }
                }
            }
            LoadingStage::ProcessingSkeleton => {
                // Find the skeleton in the spawned scene
                if let Some(scene_entity) = loading.scene_instance {
                    if let Ok(children) = children_query.get(scene_entity) {
                        if let Some(skeleton_entity) = find_skeleton_entity(
                            children.to_vec().into_iter(),
                            &children_query,
                            &name_query,
                        ) {
                            // Build the humanoid rig with optimized bone mapping
                            let bone_names =
                                collect_bone_names(skeleton_entity, &children_query, &name_query);
                            let mut humanoid_rig =
                                HumanoidRig::from_skeleton_optimized(skeleton_entity, &bone_names);
                            humanoid_rig.scale = loading.scale;

                            // Store skeleton entity for animation player placement
                            loading.skeleton_entity = Some(skeleton_entity);

                            // Add the rig to the character
                            commands.entity(entity).insert(humanoid_rig);

                            loading.stage = LoadingStage::SettingUpAnimation;
                        }
                    }
                }
            }
            LoadingStage::SettingUpAnimation => {
                // Oracle's guidance: AnimationPlayer must be on the skeleton/armature entity
                // Move animation components from character entity to skeleton entity
                if let Some(skeleton_entity) = loading.skeleton_entity {
                    // Move CharacterAnimations from character to skeleton if it exists
                    let character_animations =
                        if let Ok(character_animations) = character_animations_query.get(entity) {
                            let character_animations = (*character_animations).clone();
                            commands.entity(entity).remove::<CharacterAnimations>();
                            commands
                                .entity(skeleton_entity)
                                .insert(character_animations.clone());
                            Some(character_animations)
                        } else {
                            None
                        };

                    // Move LocomotionState from character to skeleton if it exists
                    if let Ok(locomotion_state) = locomotion_query.get(entity) {
                        let locomotion_state = (*locomotion_state).clone();
                        commands.entity(entity).remove::<LocomotionState>();
                        commands.entity(skeleton_entity).insert(locomotion_state);
                    } else {
                        // Add default if not found on character
                        commands
                            .entity(skeleton_entity)
                            .insert(LocomotionState::default());
                    }

                    // Clone Velocity from character to skeleton if it exists
                    // Oracle's Option A: Keep Velocity on player entity and sync it to skeleton
                    if let Ok(velocity) = velocity_query.get(entity) {
                        // Leave component on player, just clone to skeleton
                        commands.entity(skeleton_entity).insert(velocity.clone());
                        debug!(
                            "✅ Velocity component cloned to skeleton entity {:?}",
                            skeleton_entity
                        );
                    }

                    // Remove any auto-generated AnimationPlayer from GLB loader
                    commands.entity(skeleton_entity).remove::<AnimationPlayer>();

                    // Place remaining animation components on the skeleton entity
                    // Add ControlledBy component to link skeleton back to controlling character
                    let mut components = (
                        AnimationPlayer::default(),
                        AnimationPlayback::default(),
                        ControlledBy::new(entity), // Link skeleton to character entity
                    );

                    commands.entity(skeleton_entity).insert(components);

                    // Add AnimationGraphHandle if we have the animation set
                    if let Some(character_animations) = character_animations {
                        if let Some(animation_set) =
                            animation_sets.get(&character_animations.animation_set)
                        {
                            commands
                                .entity(skeleton_entity)
                                .insert(AnimationGraphHandle(animation_set.graph.clone()));
                            debug!(
                                "✅ Added AnimationGraphHandle with graph to skeleton entity {:?}",
                                skeleton_entity
                            );
                        } else {
                            debug!(
                                "⚠️ Animation set not yet loaded for skeleton entity {:?}",
                                skeleton_entity
                            );
                        }
                    }

                    debug!(
                        "✅ Animation components moved to skeleton entity {:?} with ControlledBy link to {:?} (Oracle-guided fix)",
                        skeleton_entity, entity
                    );
                } else {
                    warn!(
                        "❌ No skeleton entity found for animation setup on character {:?}",
                        entity
                    );
                }
                loading.stage = LoadingStage::Complete;
            }
            LoadingStage::Complete => {
                // Remove loading component
                commands.entity(entity).remove::<CharacterAssetLoading>();
            }
        }
    }
}

/// Find the skeleton entity in a glTF scene hierarchy
fn find_skeleton_entity(
    entities: impl Iterator<Item = Entity>,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
) -> Option<Entity> {
    for entity in entities {
        // Check if this entity has bones
        if let Ok(name) = name_query.get(entity) {
            if name.as_str().contains("Armature") || name.as_str().contains("Skeleton") {
                return Some(entity);
            }
        }

        // Check if any bones are found in children
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(child_name) = name_query.get(child) {
                    if child_name.as_str().contains("mixamorig")
                        || child_name.as_str().contains("Hips")
                    {
                        return Some(entity);
                    }
                }
            }

            // Recursively search children
            if let Some(skeleton) =
                find_skeleton_entity(children.to_vec().into_iter(), children_query, name_query)
            {
                return Some(skeleton);
            }
        }
    }
    None
}

/// Collect bone names from skeleton hierarchy
fn collect_bone_names(
    skeleton_entity: Entity,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
) -> Vec<String> {
    let mut bone_names = Vec::new();
    collect_bone_names_recursive(skeleton_entity, children_query, name_query, &mut bone_names);
    bone_names
}

/// Recursively collect bone names from hierarchy
fn collect_bone_names_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
    bone_names: &mut Vec<String>,
) {
    if let Ok(name) = name_query.get(entity) {
        bone_names.push(name.to_string());
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            collect_bone_names_recursive(child, children_query, name_query, bone_names);
        }
    }
}

/// System to apply scale corrections for imported models
/// Applies scale to the visual model hierarchy instead of just the root entity
pub fn apply_model_scale_corrections(
    rig_query: Query<(&HumanoidRig, &Children), Added<HumanoidRig>>,
    mut transform_query: Query<&mut Transform>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
) {
    for (rig, children) in rig_query.iter() {
        if rig.scale != 1.0 {
            // Find the visual model entities (non-skeleton entities) to apply scale
            for child in children.iter() {
                if let Ok(name) = name_query.get(child) {
                    // Skip skeleton/armature entities, apply scale to visual meshes
                    if !name.as_str().contains("Armature")
                        && !name.as_str().contains("Skeleton")
                        && !name.as_str().contains("mixamorig")
                    {
                        if let Ok(mut transform) = transform_query.get_mut(child) {
                            transform.scale = Vec3::splat(rig.scale);
                        }

                        // Also apply to child entities recursively for complex models
                        apply_scale_recursive(
                            child,
                            rig.scale,
                            &children_query,
                            &mut transform_query,
                            &name_query,
                        );
                    }
                }
            }
        }
    }
}

/// Recursively apply scale to child entities (helper function)
fn apply_scale_recursive(
    entity: Entity,
    scale: f32,
    children_query: &Query<&Children>,
    transform_query: &mut Query<&mut Transform>,
    name_query: &Query<&Name>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if let Ok(name) = name_query.get(child) {
                // Don't scale skeleton bones, only visual meshes
                if !name.as_str().contains("mixamorig") {
                    if let Ok(mut transform) = transform_query.get_mut(child) {
                        transform.scale = Vec3::splat(scale);
                    }
                    apply_scale_recursive(
                        child,
                        scale,
                        children_query,
                        transform_query,
                        name_query,
                    );
                }
            }
        }
    }
}
