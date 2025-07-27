use crate::water::components::*;
use amp_render::prelude::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Material factory functions for water rendering
pub struct WaterMaterialFactory;

impl WaterMaterialFactory {
    /// Create water bottom material (mud/sand with high roughness)
    pub fn create_water_bottom_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        })
    }

    /// Create water surface material (reflective with alpha blending)
    pub fn create_water_surface_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.8,
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..default()
        })
    }

    /// Create metallic material with custom properties
    pub fn create_metallic_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
        metallic: f32,
        roughness: f32,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic,
            perceptual_roughness: roughness,
            ..default()
        })
    }
}

/// System to setup lake with water surface and bottom
pub fn setup_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lake_size = 200.0;
    let lake_depth = 5.0;
    let lake_position = Vec3::new(300.0, -2.0, 300.0); // Positioned away from spawn and below ground

    // Create lake basin (carved out ground)
    let basin_mesh = meshes.add(Cylinder::new(lake_size / 2.0, lake_depth));
    let basin_material = WaterMaterialFactory::create_water_bottom_material(
        &mut materials,
        Color::srgb(0.3, 0.25, 0.2),
    );

    let _basin_entity = commands
        .spawn((
            Mesh3d(basin_mesh.clone()),
            MeshMaterial3d(basin_material.clone()),
            Transform::from_xyz(
                lake_position.x,
                lake_position.y - lake_depth / 2.0,
                lake_position.z,
            ),
            GlobalTransform::default(),
            Visibility::default(),
            RigidBody::Fixed,
            Collider::cylinder(lake_depth / 2.0, lake_size / 2.0),
            Name::new("Lake Basin"),
            BatchKey::new(&basin_mesh, &basin_material),
            Cullable::new(lake_size * 0.7),
        ))
        .id();

    // Create lake water surface
    let water_mesh = meshes.add(Plane3d::default().mesh().size(lake_size, lake_size));
    let water_material = WaterMaterialFactory::create_water_surface_material(
        &mut materials,
        Color::srgba(0.1, 0.4, 0.8, 0.7),
    );

    let _water_entity = commands
        .spawn((
            Mesh3d(water_mesh.clone()),
            MeshMaterial3d(water_material.clone()),
            Transform::from_xyz(lake_position.x, lake_position.y, lake_position.z),
            GlobalTransform::default(),
            Visibility::default(),
            Lake {
                size: lake_size,
                depth: lake_depth,
                wave_height: 0.5,
                wave_speed: 1.0,
                position: lake_position,
            },
            WaterBody,
            RigidBody::Fixed,
            Collider::cuboid(lake_size / 2.0, 0.1, lake_size / 2.0),
            Sensor,
            Name::new("Lake"),
            BatchKey::new(&water_mesh, &water_material),
            Cullable::new(lake_size * 0.7),
        ))
        .id();

    // Create lake bottom
    let bottom_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(lake_size * 0.9, lake_size * 0.9),
    );
    let bottom_material = WaterMaterialFactory::create_water_bottom_material(
        &mut materials,
        Color::srgb(0.2, 0.15, 0.1),
    );

    commands.spawn((
        Mesh3d(bottom_mesh.clone()),
        MeshMaterial3d(bottom_material.clone()),
        Transform::from_xyz(
            lake_position.x,
            lake_position.y - lake_depth,
            lake_position.z,
        ),
        GlobalTransform::default(),
        Visibility::default(),
        Name::new("Lake Bottom"),
        BatchKey::new(&bottom_mesh, &bottom_material),
        Cullable::new(lake_size * 0.7),
    ));

    info!(
        "🌊 Lake system setup complete - positioned at ({}, {}, {})",
        lake_position.x, lake_position.y, lake_position.z
    );
}

/// System to setup yacht with full movement controls
pub fn setup_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let yacht_position = Vec3::new(300.0, -1.0, 300.0); // On the lake surface

    // Yacht hull
    let yacht_mesh = meshes.add(Cuboid::new(8.0, 2.0, 20.0)); // Boat-like proportions
    let yacht_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..default()
    });

    let yacht_id = commands
        .spawn((
            Mesh3d(yacht_mesh.clone()),
            MeshMaterial3d(yacht_material.clone()),
            Transform::from_xyz(yacht_position.x, yacht_position.y, yacht_position.z),
            GlobalTransform::default(),
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::cuboid(4.0, 1.0, 10.0),
            Yacht {
                speed: 0.0,
                max_speed: 25.0,
                turning_speed: 2.0,
                buoyancy: 15.0,
                wake_enabled: true,
            },
            Boat,
            Name::new("Yacht"),
            BatchKey::new(&yacht_mesh, &yacht_material),
            Cullable::new(300.0),
        ))
        .id();

    // Yacht cabin
    let cabin_mesh = meshes.add(Cuboid::new(6.0, 3.0, 8.0));
    let cabin_material = WaterMaterialFactory::create_metallic_material(
        &mut materials,
        Color::srgb(0.8, 0.8, 0.9),
        0.3,
        0.4,
    );

    commands
        .spawn((
            Mesh3d(cabin_mesh.clone()),
            MeshMaterial3d(cabin_material.clone()),
            Transform::from_xyz(0.0, 3.5, -2.0),
            GlobalTransform::default(),
            Visibility::default(),
            Name::new("Yacht Cabin"),
            BatchKey::new(&cabin_mesh, &cabin_material),
            Cullable::new(300.0),
        ))
        .insert(ChildOf(yacht_id));

    // Yacht mast
    let mast_mesh = meshes.add(Cylinder::new(0.2, 15.0));
    let mast_material = WaterMaterialFactory::create_metallic_material(
        &mut materials,
        Color::srgb(0.6, 0.4, 0.2),
        0.1,
        0.8,
    );

    commands
        .spawn((
            Mesh3d(mast_mesh.clone()),
            MeshMaterial3d(mast_material.clone()),
            Transform::from_xyz(0.0, 9.5, 2.0),
            GlobalTransform::default(),
            Visibility::default(),
            Name::new("Yacht Mast"),
            BatchKey::new(&mast_mesh, &mast_material),
            Cullable::new(300.0),
        ))
        .insert(ChildOf(yacht_id));

    info!("⛵ Yacht system setup complete - controls: I/K (forward/back), J/L (turn)");
}

/// System to handle yacht movement with IJKL keys
pub fn yacht_movement_system(
    time: Res<Time<Fixed>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut yacht_query: Query<(&mut Transform, &mut Yacht, &mut Velocity), With<Boat>>,
) {
    for (mut transform, yacht, mut velocity) in yacht_query.iter_mut() {
        let mut acceleration = Vec3::ZERO;
        let mut angular_velocity = 0.0;

        // Forward/backward movement
        if keys.pressed(KeyCode::KeyI) {
            acceleration += transform.forward() * yacht.max_speed;
        }
        if keys.pressed(KeyCode::KeyK) {
            acceleration -= transform.forward() * yacht.max_speed * 0.5;
        }

        // Turning
        if keys.pressed(KeyCode::KeyJ) {
            angular_velocity = yacht.turning_speed;
        }
        if keys.pressed(KeyCode::KeyL) {
            angular_velocity = -yacht.turning_speed;
        }

        // Apply rotation using fixed timestep for consistent physics
        transform.rotate_y(angular_velocity * time.delta_secs());

        // Apply movement with water resistance using fixed timestep
        let drag = 0.95;
        velocity.linvel = velocity.linvel * drag + acceleration * time.delta_secs() * 0.1;

        // Keep yacht on water surface (simple buoyancy) using fixed timestep
        if transform.translation.y < 0.5 {
            velocity.linvel.y += yacht.buoyancy * time.delta_secs();
        }
    }
}

/// System to animate water waves
pub fn water_wave_system(
    time: Res<Time>,
    mut lake_query: Query<(&mut Transform, &Lake), With<WaterBody>>,
) {
    for (mut transform, lake) in lake_query.iter_mut() {
        let wave_offset = (time.elapsed_secs() * lake.wave_speed).sin() * lake.wave_height * 0.1;
        transform.translation.y = lake.position.y + wave_offset;
    }
}

/// System to handle yacht buoyancy physics in water
pub fn yacht_buoyancy_system(
    mut yacht_query: Query<(&mut Transform, &mut Velocity, &Yacht), With<Boat>>,
    lake_query: Query<(&Transform, &Lake), (With<WaterBody>, Without<Boat>)>,
) {
    if let Ok((water_transform, lake)) = lake_query.single() {
        for (yacht_transform, mut velocity, yacht) in yacht_query.iter_mut() {
            let water_level = water_transform.translation.y;
            let yacht_bottom = yacht_transform.translation.y - 1.0;

            // Check if yacht is within lake boundaries
            let distance_from_center = Vec2::new(
                yacht_transform.translation.x - lake.position.x,
                yacht_transform.translation.z - lake.position.z,
            )
            .length();

            let max_distance = lake.size / 2.0 - 10.0; // 10m buffer from edge

            if distance_from_center > max_distance {
                // Push yacht back toward lake center
                let direction_to_center = Vec2::new(
                    lake.position.x - yacht_transform.translation.x,
                    lake.position.z - yacht_transform.translation.z,
                )
                .normalize();

                velocity.linvel.x += direction_to_center.x * 5.0;
                velocity.linvel.z += direction_to_center.y * 5.0;
            }

            if yacht_bottom < water_level {
                let submersion = water_level - yacht_bottom;
                let buoyancy_force = submersion * yacht.buoyancy;
                velocity.linvel.y += buoyancy_force * 0.1;

                // Damping in water
                velocity.linvel *= 0.98;
            }
        }
    }
}

/// System to keep yacht within reasonable water constraints
pub fn yacht_water_constraint_system(
    mut yacht_query: Query<(&mut Transform, &mut Velocity), With<Yacht>>,
    lake_query: Query<&Lake, With<WaterBody>>,
) {
    if let Ok(lake) = lake_query.single() {
        for (mut transform, mut velocity) in yacht_query.iter_mut() {
            // Ensure yacht stays above ground level
            if transform.translation.y < -4.0 {
                transform.translation.y = -1.0;
                velocity.linvel.y = 0.0;
            }

            // Keep yacht within a reasonable distance of lake
            let distance_from_lake = Vec2::new(
                transform.translation.x - lake.position.x,
                transform.translation.z - lake.position.z,
            )
            .length();

            if distance_from_lake > lake.size {
                // Teleport yacht back to lake if it gets too far
                transform.translation.x = lake.position.x;
                transform.translation.z = lake.position.z;
                transform.translation.y = lake.position.y + 1.0;
                velocity.linvel = Vec3::ZERO;
            }
        }
    }
}
