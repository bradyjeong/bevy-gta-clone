/// Road system plugin for Bevy integration
use bevy::prelude::*;

use crate::road::{
    components::*,
    network::{RoadGenerationParams, RoadNetwork},
    resources::RoadEntityMap,
    systems::*,
};
use amp_render::road::{RoadMeshReady, RoadMeshRequest};

/// Plugin for the advanced road system
#[derive(Default)]
pub struct RoadPlugin {
    /// Custom road generation parameters
    pub generation_params: Option<RoadGenerationParams>,
    /// Whether to enable debug systems
    pub enable_debug: bool,
}

impl RoadPlugin {
    /// Create a new road plugin with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom road generation parameters
    pub fn with_generation_params(mut self, params: RoadGenerationParams) -> Self {
        self.generation_params = Some(params);
        self
    }

    /// Enable debug systems for development
    pub fn with_debug(mut self) -> Self {
        self.enable_debug = true;
        self
    }
}

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        // Register component types for reflection and serialization
        app.register_type::<RoadEntity>()
            .register_type::<IntersectionEntity>()
            .register_type::<RoadType>()
            .register_type::<IntersectionType>()
            .register_type::<RoadSurface>()
            .register_type::<RoadConfig>()
            .register_type::<IntersectionConfig>()
            .register_type::<TrafficControl>();

        // Initialize resources
        let road_network = if let Some(params) = &self.generation_params {
            RoadNetwork::new(params.clone())
        } else {
            RoadNetwork::default()
        };

        app.insert_resource(road_network)
            .insert_resource(RoadGenerationTimer::new(400.0))
            .insert_resource(RoadMeshCache::new(500))
            .insert_resource(RoadEntityMap::new());

        // Register events for road mesh communication
        app.add_event::<RoadMeshRequest>()
            .add_event::<RoadMeshReady>();

        // Add core road systems with Oracle's system ordering
        app.add_systems(
            Update,
            (
                road_generation_system.in_set(amp_core::system_ordering::RoadSystemSet::Generation),
                road_update_system.in_set(amp_core::system_ordering::RoadSystemSet::Update),
                intersection_detection_system
                    .in_set(amp_core::system_ordering::RoadSystemSet::Update),
                vehicle_road_alignment_system
                    .in_set(amp_core::system_ordering::RoadSystemSet::Update),
                npc_road_alignment_system.in_set(amp_core::system_ordering::RoadSystemSet::Update),
                road_entity_map_maintenance_system
                    .in_set(amp_core::system_ordering::RoadSystemSet::Update),
                // NOTE: Disabled to avoid conflict with amp_render's attach_road_meshes_with_stats
                // road_mesh_attachment_system
                //     .in_set(amp_core::system_ordering::RoadSystemSet::Update),
                road_cleanup_system.in_set(amp_core::system_ordering::RoadSystemSet::Cleanup),
            ),
        );

        // Add debug systems if enabled
        if self.enable_debug {
            app.add_systems(
                Update,
                (
                    road_debug_system.in_set(amp_core::system_ordering::RoadSystemSet::Debug),
                    road_reset_system.in_set(amp_core::system_ordering::RoadSystemSet::Debug),
                ),
            );
        }

        // Add startup systems
        app.add_systems(Startup, setup_road_materials);

        info!(
            "RoadPlugin initialized with {} debug systems",
            if self.enable_debug {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

/// Startup system to create road materials
fn setup_road_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Create standard road materials
    let asphalt_material = materials.add(StandardMaterial {
        base_color: RoadSurface::Asphalt.base_color(),
        perceptual_roughness: RoadSurface::Asphalt.roughness(),
        metallic: 0.0,
        reflectance: 0.2,
        ..default()
    });

    let concrete_material = materials.add(StandardMaterial {
        base_color: RoadSurface::Concrete.base_color(),
        perceptual_roughness: RoadSurface::Concrete.roughness(),
        metallic: 0.0,
        reflectance: 0.2,
        ..default()
    });

    let dirt_material = materials.add(StandardMaterial {
        base_color: RoadSurface::Dirt.base_color(),
        perceptual_roughness: RoadSurface::Dirt.roughness(),
        metallic: 0.0,
        reflectance: 0.1,
        ..default()
    });

    let marking_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95),
        emissive: LinearRgba::new(0.2, 0.2, 0.2, 1.0),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        reflectance: 0.5,
        ..default()
    });

    // Store materials as a resource for systems to use
    commands.insert_resource(RoadMaterials {
        asphalt: asphalt_material,
        concrete: concrete_material,
        dirt: dirt_material,
        markings: marking_material,
    });
}

/// Resource containing pre-created road materials
#[derive(Resource)]
pub struct RoadMaterials {
    pub asphalt: Handle<StandardMaterial>,
    pub concrete: Handle<StandardMaterial>,
    pub dirt: Handle<StandardMaterial>,
    pub markings: Handle<StandardMaterial>,
}

impl RoadMaterials {
    /// Get material handle for a road surface type
    pub fn get_surface_material(&self, surface: RoadSurface) -> Handle<StandardMaterial> {
        match surface {
            RoadSurface::Asphalt => self.asphalt.clone(),
            RoadSurface::Concrete => self.concrete.clone(),
            RoadSurface::Dirt => self.dirt.clone(),
            RoadSurface::Cobblestone => self.concrete.clone(), // Fallback to concrete
        }
    }

    /// Get the lane marking material
    pub fn get_marking_material(&self) -> Handle<StandardMaterial> {
        self.markings.clone()
    }
}

/// Extension trait for App to easily add road system
pub trait RoadPluginExt {
    /// Add the road plugin with default settings
    fn add_road_system(&mut self) -> &mut Self;

    /// Add the road plugin with custom settings
    fn add_road_system_with_config(&mut self, plugin: RoadPlugin) -> &mut Self;
}

impl RoadPluginExt for App {
    fn add_road_system(&mut self) -> &mut Self {
        self.add_plugins(RoadPlugin::new())
    }

    fn add_road_system_with_config(&mut self, plugin: RoadPlugin) -> &mut Self {
        self.add_plugins(plugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_road_plugin_basic_setup() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new());

        // Verify resources were inserted
        assert!(app.world().contains_resource::<RoadNetwork>());
        assert!(app.world().contains_resource::<RoadGenerationTimer>());
        assert!(app.world().contains_resource::<RoadMeshCache>());
    }

    #[test]
    fn test_road_plugin_with_custom_params() {
        let params = RoadGenerationParams {
            chunk_size: 500.0,
            generation_radius: 1000.0,
            cleanup_radius: 2500.0,
            road_density: 0.9,
            curve_probability: 0.4,
        };

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new().with_generation_params(params.clone()));

        let road_network = app.world().resource::<RoadNetwork>();
        assert_eq!(road_network.generation_params.chunk_size, 500.0);
        assert_eq!(road_network.generation_params.generation_radius, 1000.0);
        assert_eq!(road_network.generation_params.road_density, 0.9);
    }

    #[test]
    fn test_road_plugin_with_debug() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_plugins(RoadPlugin::new().with_debug());

        // Verify the plugin was configured with debug enabled
        // This is a basic smoke test since we can't easily verify system registration
        assert!(app.world().contains_resource::<RoadNetwork>());
    }

    #[test]
    fn test_road_plugin_extension_trait() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins).add_road_system();

        assert!(app.world().contains_resource::<RoadNetwork>());
    }

    #[test]
    fn test_road_materials_resource() {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .add_plugins(RoadPlugin::new());

        // Run startup systems
        app.update();

        // Verify materials resource was created
        assert!(app.world().contains_resource::<RoadMaterials>());

        let materials = app.world().resource::<RoadMaterials>();
        let asphalt_material = materials.get_surface_material(RoadSurface::Asphalt);
        let marking_material = materials.get_marking_material();

        // Basic verification that handles are valid (not default)
        assert!(!asphalt_material.is_weak());
        assert!(!marking_material.is_weak());
    }

    #[test]
    fn test_road_surface_material_mapping() {
        use bevy::asset::AssetServer;

        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .add_plugins(RoadPlugin::new());

        // Run startup systems
        app.update();

        let materials = app.world().resource::<RoadMaterials>();

        // Test that different surfaces map to different materials
        let asphalt = materials.get_surface_material(RoadSurface::Asphalt);
        let concrete = materials.get_surface_material(RoadSurface::Concrete);
        let dirt = materials.get_surface_material(RoadSurface::Dirt);

        // They should be different handles
        assert_ne!(asphalt.id(), concrete.id());
        assert_ne!(concrete.id(), dirt.id());
        assert_ne!(asphalt.id(), dirt.id());
    }
}
