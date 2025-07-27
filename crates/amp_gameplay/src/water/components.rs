use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct Lake {
    pub size: f32,
    pub depth: f32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub position: Vec3,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Yacht {
    pub speed: f32,
    pub max_speed: f32,
    pub turning_speed: f32,
    pub buoyancy: f32,
    pub wake_enabled: bool,
}

#[derive(Component, Debug, Clone)]
pub struct WaterBody;

#[derive(Component, Debug, Clone)]
pub struct WaterWave {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Boat;
