use bevy::prelude::*;

use amp_engine::batch::BatchProcessingPlugin;
use amp_engine::world_streaming::{WorldStreamer, WorldStreamingPlugin};
use amp_gameplay::prelude::*;
use amp_render::culling::CullingConfig;
use amp_render::prelude::*;

mod perf_ui;
use perf_ui::PerfUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayPlugins)
        .add_plugins(BatchingPlugin)
        .add_plugins(BatchProcessingPlugin)
        .add_plugins(WorldStreamingPlugin)
        .add_plugins(PerfUiPlugin)
        .insert_resource(CullingConfig {
            max_distance: 800.0,
            enable_frustum_culling: true,
            enable_distance_culling: true,
        })
        .insert_resource(WorldStreamer::default())
        .add_systems(Startup, (setup_scene, setup_interaction_ui))
        .add_systems(Update, (display_info, camera_controller))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn player character
    commands.spawn((
        CharacterBundle {
            player: Player,
            speed: Speed::default(),
            grounded: Grounded::default(),
            controller: CharacterController::default(),
            camera_target: CameraTarget::new(),
            input: CharacterInput::default(),
            capsule: CapsuleCollider::default(),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        },
        // Player marker already included in Player component from amp_gameplay
        Mesh3d(meshes.add(Mesh::from(Capsule3d::new(0.5, 1.8)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.8),
            ..default()
        })),
        Name::new("Player"),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Camera"),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, 0.7, 0.0)),
        Name::new("Sun"),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(100.0, 100.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
        Name::new("Ground"),
    ));
}

fn setup_interaction_ui(mut commands: Commands) {
    // Interaction prompt UI
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        },
        InteractionPromptUI,
    ));

    // Controls UI
    commands.spawn((
        Text::new("CONTROLS - Walking:\nArrow Keys: Move\nF: Enter Vehicle\nESC: Toggle Cursor"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ControlsUI,
    ));

    // FPS counter
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        FpsCounter,
    ));
}

fn display_info(time: Res<Time>, mut fps_query: Query<&mut Text, With<FpsCounter>>) {
    let fps = 1.0 / time.delta_secs();

    for mut text in fps_query.iter_mut() {
        **text = format!("FPS: {:.0}", fps);
    }
}

fn camera_controller(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut mouse_events: EventReader<bevy::input::mouse::MouseMotion>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            let player_pos = player_transform.translation;

            // Handle mouse input for camera orbit
            let mut mouse_delta = Vec2::ZERO;
            for event in mouse_events.read() {
                mouse_delta += event.delta;
            }

            // Simple camera follow with offset
            let camera_offset = Vec3::new(0.0, 5.0, 10.0);
            let target_pos = player_pos + camera_offset;

            // Smooth camera movement
            let smoothing = 5.0;
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, time.delta_secs() * smoothing);

            // Look at player
            camera_transform.look_at(player_pos + Vec3::Y, Vec3::Y);
        }
    }
}

#[derive(Component)]
struct InteractionPromptUI;

#[derive(Component)]
struct ControlsUI;

#[derive(Component)]
struct FpsCounter;
