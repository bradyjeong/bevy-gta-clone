//! Example demonstrating Mixamo character asset integration
//!
//! This example shows how to:
//! - Load a Mixamo character model
//! - Set up animation assets from RON files
//! - Spawn a complete character with physics and animations
//! - Demonstrate the asset loading workflow

use amp_gameplay::character::{
    assets::{AnimationSetConfig, CharacterAssetRegistry},
    bundles::CharacterBundle,
    components::AnimationSet,
    CharacterPlugin,
};
use bevy::prelude::*;

#[cfg(feature = "rapier3d_030")]
use bevy_rapier3d::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins).add_plugins(CharacterPlugin);

    #[cfg(feature = "rapier3d_030")]
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default());

    app.add_systems(Startup, setup_scene)
        .add_systems(Update, (handle_asset_loading, character_spawning_example))
        .run();
}

/// Setup the basic scene with camera and lighting
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<CharacterAssetRegistry>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    #[cfg(feature = "rapier3d_030")]
    commands.spawn((
        Collider::cuboid(10.0, 0.1, 10.0),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // Load default Mixamo assets
    registry.load_mixamo_defaults(&asset_server);

    info!("Scene setup complete. Ready to load Mixamo characters!");
}

/// Resource to track when we've spawned our example character
#[derive(Resource, Default)]
struct ExampleState {
    character_spawned: bool,
}

/// Handle asset loading and demonstrate the workflow
fn handle_asset_loading(
    mut commands: Commands,
    registry: Res<CharacterAssetRegistry>,
    mut example_state: Local<ExampleState>,
    asset_server: Res<AssetServer>,
) {
    // Check if we have loaded animation sets
    if !example_state.character_spawned && registry.animation_sets.contains_key("mixamo") {
        info!("Animation set loaded! Ready to spawn character.");

        // Create a default animation set handle for demonstration
        // In a real application, this would come from the loaded assets
        let animation_set_handle = asset_server.load("animations/mixamo_default.animset.ron");

        // Demonstrate different spawning methods
        info!("Spawning character with asset loading...");

        // Method 1: Spawn with automatic asset loading
        let character_entity = CharacterBundle::spawn_player(
            &mut commands,
            "models/mixamo_character.glb",
            animation_set_handle.clone(),
        );

        info!("Character spawned with entity: {:?}", character_entity);

        // Method 2: Spawn with custom scale (for demonstration)
        let scaled_character = CharacterBundle::spawn_player_with_scale(
            &mut commands,
            "models/mixamo_character_scaled.glb",
            animation_set_handle,
            0.01, // Mixamo models are often in centimeters
        );

        info!(
            "Scaled character spawned with entity: {:?}",
            scaled_character
        );

        example_state.character_spawned = true;
    }
}

/// Example system showing different character spawning approaches
fn character_spawning_example(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<CharacterAssetRegistry>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Press 'C' to spawn a new character
    if keyboard.just_pressed(KeyCode::KeyC) {
        info!("Spawning new character...");

        if let Some(animation_set) = registry.get_animation_set("mixamo") {
            let character = CharacterBundle::spawn_player(
                &mut commands,
                "models/another_character.glb",
                animation_set.clone(),
            );

            info!("New character spawned: {:?}", character);
        } else {
            warn!("No animation set available for 'mixamo' character type");
        }
    }

    // Press 'P' to spawn a physics character
    #[cfg(feature = "rapier3d_030")]
    if keyboard.just_pressed(KeyCode::KeyP) {
        info!("Spawning physics character...");

        if let Some(animation_set) = registry.get_animation_set("mixamo") {
            use amp_gameplay::character::bundles::PhysicsCharacterBundle;

            let physics_character = PhysicsCharacterBundle::spawn_physics_player(
                &mut commands,
                "models/physics_character.glb",
                animation_set.clone(),
            );

            info!("Physics character spawned: {:?}", physics_character);
        }
    }
}

/// Helper function to create a sample animation set config
fn create_sample_animation_config() -> AnimationSetConfig {
    AnimationSetConfig::default_mixamo()
}

/// System to demonstrate dynamic animation set creation
fn dynamic_animation_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_sets: ResMut<Assets<AnimationSet>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyA) {
        info!("Creating dynamic animation set...");

        let mut animation_set = AnimationSet::new("dynamic_character");

        // Add some animation clips
        animation_set.add_clip(
            amp_gameplay::character::components::Locomotion::Idle,
            asset_server.load("animations/custom_idle.glb#Animation0"),
            1.0,
        );

        animation_set.add_clip(
            amp_gameplay::character::components::Locomotion::Walk,
            asset_server.load("animations/custom_walk.glb#Animation0"),
            1.0,
        );

        let handle = animation_sets.add(animation_set);

        // Spawn character with this animation set
        let character =
            CharacterBundle::spawn_player(&mut commands, "models/custom_character.glb", handle);

        info!("Dynamic character spawned: {:?}", character);
    }
}
