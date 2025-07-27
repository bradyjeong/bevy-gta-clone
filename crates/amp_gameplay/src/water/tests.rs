#[cfg(test)]
mod tests {
    use crate::water::*;
    use amp_render::BatchingPlugin;
    use bevy::app::App;
    use bevy::DefaultPlugins;
    use bevy_rapier3d::prelude::*;

    #[test]
    fn water_plugin_can_be_added() {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(BatchingPlugin)
            .add_plugins(WaterPlugin);

        // If we get here without panicking, the plugin was added successfully
        assert!(true);
    }

    #[test]
    fn water_components_exist() {
        // Test that all water components can be instantiated
        let _lake = Lake::default();
        let _yacht = Yacht::default();
        let _water_body = WaterBody;
        let _boat = Boat;
        let _water_wave = WaterWave {
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
        };

        assert!(true);
    }
}
