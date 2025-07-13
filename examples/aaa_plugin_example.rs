//! AAA Plugin Architecture Example
//!
//! This example demonstrates how to use the AAAPlugin system to create
//! a custom plugin and integrate it with the built-in subsystems.
//!
//! Run with: cargo run --bin aaa_plugin_example (from examples directory)

use amp_engine::prelude::*;
use bevy::prelude::*;

/// Example custom plugin that adds a simple system
struct CustomGameplayPlugin;

impl AAAPlugin for CustomGameplayPlugin {
    fn build(&self, app: &mut App) -> amp_core::Result<()> {
        info!("Building CustomGameplayPlugin");
        
        // Add a resource
        app.insert_resource(GameTimer { elapsed: 0.0 });
        
        // Add a system
        app.add_systems(Update, update_game_timer);
        
        Ok(())
    }
    
    fn stage(&self) -> PluginStage {
        PluginStage::PostStartup
    }
}

/// Resource for tracking game time
#[derive(Resource)]
struct GameTimer {
    elapsed: f32,
}

/// System that updates the game timer
fn update_game_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.elapsed += time.delta_secs();
    
    if timer.elapsed.floor() as i32 % 5 == 0 && timer.elapsed.fract() < time.delta_secs() {
        info!("Game has been running for {} seconds", timer.elapsed.floor());
    }
}

fn main() {
    // Create Bevy app with AAA plugins
    App::new()
        // Add core Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AAA Plugin Example".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        
        // Add the AAA plugin system with a custom plugin
        .add_plugins(
            AAAPlugins::default()
                .add_plugin(CustomGameplayPlugin)
        )
        
        // Add startup system
        .add_systems(Startup, setup)
        
        // Run the app
        .run();
}

fn setup() {
    info!("Setting up AAA Plugin Example");
    info!("Setup complete - AAA Plugin system initialized");
}
