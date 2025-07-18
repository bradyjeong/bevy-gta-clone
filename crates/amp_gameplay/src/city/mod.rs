//! City generation systems for massive urban environments
//!
//! This module provides static city generation with buildings, streets, and infrastructure
//! matching the massive urban environment visible in f430bc6.

pub mod components;
pub mod layout;
pub mod render_assets;
pub mod resources;
pub mod systems;

#[cfg(test)]
mod test_instanced_rendering;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::*;

use self::render_assets::*;
use self::resources::*;
use self::systems::*;

/// City plugin providing static city generation with buildings, streets, and infrastructure
#[derive(Default)]
pub struct CityPlugin;

impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register component types
            .register_type::<components::Building>()
            .register_type::<components::Street>()
            .register_type::<components::Intersection>()
            .register_type::<components::BuildingType>()
            .register_type::<components::CityTile>()
            .register_type::<components::ColliderMarker>()
            .register_type::<components::DeferredLight>()
            .register_type::<components::LightType>()
            // Resources
            .init_resource::<CityConfig>()
            .init_resource::<CityLayout>()
            .init_resource::<CityPrefabs>()
            // City generation systems
            .add_systems(
                Startup,
                (
                    city_setup,
                    load_city_layout,
                    register_city_prefabs_system,
                    load_city_render_assets,
                    generate_city_grid,
                    spawn_city_infrastructure,
                    add_city_colliders,
                )
                    .chain(),
            );
    }
}

/// City streaming plugin providing radius-based city streaming
#[derive(Default)]
pub struct CityStreamingPlugin;

impl Plugin for CityStreamingPlugin {
    fn build(&self, app: &mut App) {
        app
            // City streaming systems
            .add_systems(Update, (spawn_city_radius, update_light_activity));
    }
}
