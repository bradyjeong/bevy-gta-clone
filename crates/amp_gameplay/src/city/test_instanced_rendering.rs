//! Tests for instanced rendering implementation
//!
//! This test verifies that the city generation system properly
//! creates entities with BatchKey and Cullable components.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::components::*;
    use crate::city::render_assets::*;
    use crate::city::resources::*;
    use crate::city::systems::*;
    use amp_render::prelude::*;
    use bevy::prelude::*;

    #[test]
    fn test_city_entities_have_batch_key_and_cullable() {
        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()));

        // Add resources
        app.init_resource::<CityConfig>()
            .init_resource::<CityLayout>()
            .init_resource::<CityPrefabs>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>();

        // Run the systems in order
        app.add_systems(
            Startup,
            (
                city_setup,
                load_city_layout,
                register_city_prefabs_system,
                load_city_render_assets,
                generate_city_grid,
            )
                .chain(),
        );

        app.update();

        // Check that buildings have BatchKey and Cullable components
        let mut building_query = app
            .world_mut()
            .query::<(Entity, &Building, &BatchKey, &Cullable)>();
        let building_count = building_query.iter(&app.world()).count();
        assert!(
            building_count > 0,
            "No buildings with BatchKey and Cullable found"
        );

        // Check that streets have BatchKey and Cullable components
        let mut street_query = app
            .world_mut()
            .query::<(Entity, &Street, &BatchKey, &Cullable)>();
        let street_count = street_query.iter(&app.world()).count();
        assert!(
            street_count > 0,
            "No streets with BatchKey and Cullable found"
        );

        // Check that intersections have BatchKey and Cullable components
        let mut intersection_query = app
            .world_mut()
            .query::<(Entity, &Intersection, &BatchKey, &Cullable)>();
        let intersection_count = intersection_query.iter(&app.world()).count();
        assert!(
            intersection_count > 0,
            "No intersections with BatchKey and Cullable found"
        );

        println!("✅ City instanced rendering test passed!");
        println!("  - Buildings with BatchKey/Cullable: {}", building_count);
        println!("  - Streets with BatchKey/Cullable: {}", street_count);
        println!(
            "  - Intersections with BatchKey/Cullable: {}",
            intersection_count
        );
    }

    #[test]
    fn test_city_render_assets_created() {
        let mut app = App::new();

        // Add minimal plugins
        app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()));

        // Add resources
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>();

        // Run the system
        app.add_systems(Startup, load_city_render_assets);
        app.update();

        // Check that CityRenderAssets resource was created
        let city_render_assets = app.world().resource::<CityRenderAssets>();

        // Verify the handles are valid
        assert!(city_render_assets.cube_mesh.id() != AssetId::default());
        assert!(city_render_assets.plane_mesh.id() != AssetId::default());
        assert!(city_render_assets.building_material.id() != AssetId::default());
        assert!(city_render_assets.street_material.id() != AssetId::default());
        assert!(city_render_assets.intersection_material.id() != AssetId::default());
        assert!(city_render_assets.building_emissive_material.id() != AssetId::default());

        println!("✅ City render assets test passed!");
    }
}
