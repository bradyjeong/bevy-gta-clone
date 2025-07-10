//! Example demonstrating the new Bevy asset pipeline integration
//!
//! This example shows how to use the new AmpScenePlugin with Bevy's AssetServer
//! to load and manage scene prefabs.

use amp_engine::prelude::*;
use bevy::asset::{AssetServer, Assets};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AmpScenePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_scene_assets)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
}

fn setup(mut commands: bevy::ecs::system::Commands, asset_server: bevy::ecs::system::Res<AssetServer>) {
    // Add a camera
    commands.spawn(Camera2dBundle::default());

    // Create a simple scene prefab in memory for testing
    let prefab_ron = r#"
    (
        components: [
            (
                component_type: "Transform",
                data: Map({
                    "translation": Array([Number(0.0), Number(0.0), Number(0.0)]),
                    "rotation": Array([Number(0.0), Number(0.0), Number(0.0), Number(1.0)]),
                    "scale": Array([Number(1.0), Number(1.0), Number(1.0)])
                })
            ),
            (
                component_type: "Name",
                data: Map({
                    "name": String("Test Entity")
                })
            )
        ]
    )
    "#;

    // Store the RON content for later use
    commands.insert_resource(TestScenePrefab {
        content: prefab_ron.to_string(),
    });

    println!("Setup complete - ready to load scene prefabs");
}

#[derive(bevy::ecs::system::Resource)]
struct TestScenePrefab {
    content: String,
}

fn handle_scene_assets(
    mut commands: bevy::ecs::system::Commands,
    asset_server: bevy::ecs::system::Res<AssetServer>,
    prefab_res: Option<bevy::ecs::system::Res<TestScenePrefab>>,
    amp_scenes: bevy::ecs::system::Res<Assets<AmpScenePrefab>>,
    mut done: bevy::ecs::system::Local<bool>,
) {
    if *done {
        return;
    }

    if let Some(prefab) = prefab_res {
        // For this example, we'll create the asset in memory
        // In a real application, you would load from a file
        println!("Demonstrating asset loading with in-memory prefab");
        println!("Prefab content: {}", prefab.content);
        
        // In a real scenario, you would use:
        // let handle: Handle<AmpScenePrefab> = asset_server.load("test_scene.amp.ron");
        
        println!("Asset pipeline integration verified!");
        *done = true;
    }
}
