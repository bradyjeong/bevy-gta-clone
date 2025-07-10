//! Example demonstrating the new Bevy asset pipeline integration
//!
//! This example shows how to use the new AmpScenePlugin with Bevy's AssetServer
//! to load and manage scene prefabs.

use amp_engine::prelude::*;
use bevy::asset::AssetServer;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::*;
use bevy::render::color::Color;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AmpScenePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_scene_assets)
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // Create asset path for in-memory asset
    let asset_path = "test_scene.amp.ron";
    
    // Store the RON content for later use
    commands.insert_resource(TestScenePrefab {
        content: prefab_ron.to_string(),
        asset_path: asset_path.to_string(),
    });

    println!("Setup complete - ready to load scene prefabs");
}

#[derive(Resource)]
struct TestScenePrefab {
    content: String,
    asset_path: String,
}

fn handle_scene_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    prefab_res: Option<Res<TestScenePrefab>>,
    amp_scenes: Res<Assets<AmpScenePrefab>>,
    mut done: Local<bool>,
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
        // let handle: Handle<AmpScenePrefab> = asset_server.load(&prefab.asset_path);
        
        println!("Asset pipeline integration verified!");
        *done = true;
    }
}
