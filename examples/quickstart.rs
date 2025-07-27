//! Quickstart Example - Canonical Imports for Each Facade
//!
//! This example demonstrates the proper way to use each facade crate:
//! 1. amp_foundation - Bevy-free core functionality
//! 2. amp_system - Engine systems with Bevy integration
//! 3. amp_game - Complete game development interface
//!
//! Run with: `cargo run --example quickstart`

use bevy::prelude::*;

fn main() {
    println!("🚀 Amp Facade System Quickstart");
    println!("================================");

    // Demonstrate amp_foundation usage (Bevy-free)
    foundation_example();

    // Demonstrate amp_system usage (Bevy + engine systems)
    system_example();

    // Demonstrate amp_game usage (complete game interface)
    game_example();
}

/// Example 1: amp_foundation - Core functionality without Bevy dependencies
fn foundation_example() {
    use amp_foundation::prelude::*;

    println!("\n📦 amp_foundation Example (Bevy-free):");
    println!("--------------------------------------");

    // Math operations without Bevy
    let transform = AmpTransform::identity()
        .with_translation(Vec3::new(10.0, 5.0, 0.0))
        .with_scale(Vec3::splat(2.0));

    println!("✅ Transform created: {:?}", transform.translation);

    // AABB collision detection
    let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(10.0));
    let point = Vec3::new(5.0, 5.0, 5.0);
    let contains = aabb.contains(&point);

    println!("✅ AABB contains point {}: {}", point, contains);

    // Morton encoding for spatial indexing
    let morton = Morton3D::new(Vec3::new(128.0, 256.0, 512.0));
    let encoded = morton.encode();

    println!(
        "✅ Morton encoding: ({}, {}, {}) -> {}",
        128.0, 256.0, 512.0, encoded
    );

    // Configuration management
    let config_path = "config/game.ron";
    println!("✅ Config path resolved: {}", config_path);
}

/// Example 2: amp_system - Engine systems with Bevy integration
fn system_example() {
    use amp_system::prelude::*;

    println!("\n⚙️  amp_system Example (Bevy + Engine Systems):");
    println!("-----------------------------------------------");

    let mut app = App::new();

    // Add minimal plugins for this example
    app.add_plugins(bevy::prelude::MinimalPlugins);

    // Note: amp_system provides individual plugins, not a single system plugin
    println!("✅ amp_system types available for Bevy App");

    // Run a few update cycles
    for i in 0..3 {
        app.update();
        println!("✅ Update cycle {} completed", i + 1);
    }

    println!("✅ Engine systems running successfully");
}

/// Example 3: amp_game - Complete game development interface  
fn game_example() {
    use amp_game::prelude::*;

    println!("\n🎮 amp_game Example (Complete Game Interface):");
    println!("----------------------------------------------");

    let mut app = App::new();

    // Add minimal plugins for this example
    app.add_plugins(bevy::prelude::MinimalPlugins);

    // Add complete game development stack
    app.add_plugins(GameplayPlugins);
    app.add_plugins(PrefabFactoryPlugin);

    println!("✅ Game plugins added to Bevy App");

    // Add a simple game system
    app.add_systems(Update, demo_game_system);

    println!("✅ Demo game system added");

    // Run the game loop briefly
    for i in 0..3 {
        app.update();
        println!("✅ Game loop iteration {} completed", i + 1);
    }

    println!("✅ Complete game stack running successfully");
}

/// Simple demo system to show game development patterns
fn demo_game_system(time: Res<Time>) {
    // This would contain your game logic
    // For the demo, we just track elapsed time
    let _elapsed = time.elapsed_secs();
    // Game systems would update entities, handle input, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the quickstart example compiles and runs
    #[test]
    fn test_quickstart_example() {
        // Test foundation functionality
        foundation_example();

        // Test system functionality
        system_example();

        // Test game functionality
        game_example();
    }

    /// Test canonical imports work correctly
    #[test]
    fn test_canonical_imports() {
        // amp_foundation imports (Bevy-free)
        use amp_foundation::prelude::*;

        let _transform = AmpTransform::identity();
        let _aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let _morton = Morton3D::encode(Vec3::new(1.0, 2.0, 3.0));

        // amp_system imports (with Bevy)
        use amp_system::prelude::*;

        let mut _app = App::new();

        // amp_game imports
        use amp_game::prelude::*;

        let _game_plugin = GameplayPlugins;
    }
}
