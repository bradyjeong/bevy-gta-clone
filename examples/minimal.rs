//! Minimal example demonstrating the Amp game engine
//!
//! This example opens a window, renders a purple screen using the amp_engine::gpu module,
//! and demonstrates the Factory pattern for entity spawning.

use amp_engine::gpu::{GpuContext, SurfaceManager};
use config_core::{ConfigLoader, GameConfig};
use gameplay_factory::{Factory, PrefabId};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Register default components
    gameplay_factory::register_default_components();

    // Load configuration
    let config = ConfigLoader::new().load_with_merge::<GameConfig>()?;
    println!("Factory config: {:?}", config.factory());

    // Create factory and demonstrate entity spawning
    let mut factory = Factory::new();

    // Create a mock world to demonstrate spawning
    let mut world = bevy_ecs::world::World::new();
    let mut queue = bevy_ecs::system::CommandQueue::default();
    let mut commands = bevy_ecs::system::Commands::new(&mut queue, &world);

    // Demonstrate factory pattern by creating a simple prefab
    println!("Demonstrating Factory pattern for entity spawning...");
    let player_prefab_id = PrefabId::new(1);

    // Create a simple prefab with a mock component
    #[derive(Debug, Clone)]
    struct MockComponent {
        name: String,
    }

    impl gameplay_factory::ComponentInit for MockComponent {
        fn init(
            &self,
            _cmd: &mut bevy_ecs::system::Commands,
            _entity: bevy_ecs::entity::Entity,
        ) -> amp_core::Result<()> {
            println!("Initializing component: {}", self.name);
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Register a prefab with the factory
    let mut prefab = gameplay_factory::Prefab::new();
    prefab.add_component(Box::new(MockComponent {
        name: "Player".to_string(),
    }));
    let _ = factory.register(player_prefab_id, prefab);

    // Now spawn the entity - this should succeed
    match factory.spawn(&mut commands, player_prefab_id) {
        Ok(entity) => println!("✅ Factory successfully spawned entity {entity:?} with prefab ID: {player_prefab_id:?}"),
        Err(e) => println!("❌ Factory spawn failed: {e}"),
    }

    // Demonstrate failure case with unregistered prefab
    let unknown_prefab_id = PrefabId::new(999);
    match factory.spawn(&mut commands, unknown_prefab_id) {
        Ok(_) => println!("❌ Unexpected success for unknown prefab"),
        Err(e) => println!("✅ Expected failure for unknown prefab: {e}"),
    }

    // Demonstrate directory loading functionality
    println!("\nDemonstrating Factory directory loading...");
    #[cfg(feature = "ron")]
    {
        // Try to load prefabs from the configured directory
        match factory.load_directory(config.factory()) {
            Ok(loaded_count) => {
                println!(
                    "✅ Successfully loaded {} prefabs from directory",
                    loaded_count
                );
                println!("   Total prefabs in factory: {}", factory.len());

                // Log the expanded path for debugging
                match config.factory().expanded_prefab_path() {
                    Ok(expanded) => println!("   Searched path: {}", expanded),
                    Err(e) => println!("   Path expansion failed: {}", e),
                }
            }
            Err(e) => {
                println!("ℹ️  Directory loading failed (this is expected if no prefab directory exists): {}", e);
                println!(
                    "   You can create prefab files at: {}",
                    config.factory().prefab_path
                );
            }
        }
    }

    #[cfg(not(feature = "ron"))]
    {
        println!("ℹ️  Directory loading requires the 'ron' feature to be enabled");
    }

    // Apply commands to the world
    queue.apply(&mut world);

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Amp Game Engine - Minimal Example")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .build(&event_loop)?,
    );

    // Create GPU context
    let gpu_context = pollster::block_on(GpuContext::new(&window))?;

    // Create surface manager
    let mut surface_manager = SurfaceManager::new(&gpu_context, &window, window.inner_size())?;

    println!("Adapter: {:?}", gpu_context.adapter_info());
    println!("Features: {:?}", gpu_context.device_features());
    println!("Surface format: {:?}", surface_manager.format());

    let window_id = window.id();
    let window_for_loop = Arc::clone(&window);

    // Main event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: event_window_id,
            } if event_window_id == window_id => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        surface_manager.resize(&gpu_context, *physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // Render purple frame
                        match surface_manager.render_purple_frame(&gpu_context) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Render error: {e}"),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request a redraw
                window_for_loop.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
