//! City rendering assets for instanced rendering
//!
//! This module provides shared meshes and materials for city generation
//! to enable efficient instanced rendering through BatchKey components.

use bevy::prelude::*;

/// Resource containing shared meshes and materials for city entities
#[derive(Resource)]
pub struct CityRenderAssets {
    /// Shared cube mesh for buildings
    pub cube_mesh: Handle<Mesh>,
    /// Shared plane mesh for streets and intersections
    pub plane_mesh: Handle<Mesh>,
    /// Material for buildings
    pub building_material: Handle<StandardMaterial>,
    /// Material for streets
    pub street_material: Handle<StandardMaterial>,
    /// Material for intersections (using same as streets for now)
    pub intersection_material: Handle<StandardMaterial>,
    /// Emissive material for buildings with lit windows (replaces distant point lights)
    pub building_emissive_material: Handle<StandardMaterial>,
}

impl CityRenderAssets {
    /// Create new city render assets
    pub fn new(
        cube_mesh: Handle<Mesh>,
        plane_mesh: Handle<Mesh>,
        building_material: Handle<StandardMaterial>,
        street_material: Handle<StandardMaterial>,
        intersection_material: Handle<StandardMaterial>,
        building_emissive_material: Handle<StandardMaterial>,
    ) -> Self {
        Self {
            cube_mesh,
            plane_mesh,
            building_material,
            street_material,
            intersection_material,
            building_emissive_material,
        }
    }
}

/// System to load city render assets before city generation
pub fn load_city_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Loading city render assets for instanced rendering");

    // Create shared cube mesh for buildings
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));

    // Create shared plane mesh for streets and intersections
    let plane_mesh = meshes.add(Mesh::from(Plane3d::default().mesh().size(1.0, 1.0)));

    // Create materials for different city elements
    let building_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.7), // Light gray for buildings
        ..default()
    });

    let street_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3), // Dark gray for streets
        ..default()
    });

    let intersection_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4), // Medium gray for intersections
        ..default()
    });

    // Create emissive material for buildings with lit windows (replaces distant point lights)
    let building_emissive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.7), // Same as regular buildings
        emissive: Color::srgb(1.0, 0.9, 0.7).into(), // Warm window glow
        ..default()
    });

    // Insert the resource
    commands.insert_resource(CityRenderAssets::new(
        cube_mesh,
        plane_mesh,
        building_material,
        street_material,
        intersection_material,
        building_emissive_material,
    ));

    info!("City render assets loaded successfully");
}
