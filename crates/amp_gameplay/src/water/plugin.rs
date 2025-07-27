use crate::water::systems::*;
use bevy::prelude::*;

/// Water plugin that provides lake and yacht systems for realistic water simulation
#[derive(Default)]
pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_lake, setup_yacht))
            .add_systems(
                FixedUpdate,
                (
                    // Physics systems run at consistent 60Hz for smooth movement
                    yacht_movement_system,
                    yacht_buoyancy_system,
                    yacht_water_constraint_system,
                ),
            )
            .add_systems(
                Update,
                (
                    // Visual systems run in Update for smooth rendering
                    water_wave_system,
                ),
            );

        info!("🌊 Water Plugin initialized - lake simulation with yacht controls (IJKL)");
    }
}
