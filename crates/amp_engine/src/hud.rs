//! HUD system for displaying streaming stats and performance metrics
//! Oracle's M4 requirements: Match f430bc6 UI style with FPS counter

use crate::world_streaming::{StreamingStats, WorldStreamer};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::ui::Node;

/// HUD component marker
#[derive(Component)]
pub struct HudRoot;

/// FPS counter component
#[derive(Component)]
pub struct FpsCounter;

/// Streaming stats display component
#[derive(Component)]
pub struct StreamingStatsDisplay;

/// Memory usage display component
#[derive(Component)]
pub struct MemoryUsageDisplay;

/// Performance metrics display component
#[derive(Component)]
pub struct PerformanceMetricsDisplay;

/// HUD plugin for streaming stats and performance display
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    update_fps_counter,
                    update_streaming_stats_display,
                    update_memory_usage_display,
                    update_performance_metrics_display,
                ),
            );
    }
}

/// HUD configuration matching f430bc6 style
#[derive(Resource)]
pub struct HudConfig {
    pub font_size: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub position_offset: Vec2,
    pub panel_spacing: f32,
    pub update_interval: f32,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            text_color: Color::WHITE,
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            position_offset: Vec2::new(10.0, 10.0),
            panel_spacing: 20.0,
            update_interval: 0.1,
        }
    }
}

/// Setup HUD system
fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let hud_config = HudConfig::default();
    commands.insert_resource(hud_config);

    // HUD root container
    let hud_root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            HudRoot,
        ))
        .id();

    // FPS counter panel
    let fps_panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("FPS: --"),
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Regular.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                FpsCounter,
            ));
        })
        .id();

    // Streaming stats panel
    let streaming_panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(50.0),
                width: Val::Px(250.0),
                height: Val::Px(120.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Streaming Stats\nSectors: --\nEntities: --\nLOD0/1/I: --/--/--"),
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StreamingStatsDisplay,
            ));
        })
        .id();

    // Memory usage panel
    let memory_panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(180.0),
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Memory Usage\n-- MB"),
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MemoryUsageDisplay,
            ));
        })
        .id();

    // Performance metrics panel
    let performance_panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(250.0),
                width: Val::Px(250.0),
                height: Val::Px(80.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Performance\nUpdate: -- ms\nPeak: -- ms"),
                TextFont {
                    font: asset_server.load("fonts/FiraMono-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PerformanceMetricsDisplay,
            ));
        })
        .id();

    // Add all panels to HUD root
    commands.entity(hud_root).add_children(&[
        fps_panel,
        streaming_panel,
        memory_panel,
        performance_panel,
    ]);
}

/// Update FPS counter display
fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_query: Query<(&mut Text, &mut TextColor), With<FpsCounter>>,
) {
    for (mut text, mut text_color) in fps_query.iter_mut() {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
            .unwrap_or(0.0);

        text.0 = format!("FPS: {:.1}", fps);

        // Color coding based on FPS
        text_color.0 = if fps >= 60.0 {
            Color::srgb(0.0, 1.0, 0.0) // Green for good FPS
        } else if fps >= 30.0 {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for medium FPS
        } else {
            Color::srgb(1.0, 0.0, 0.0) // Red for poor FPS
        };
    }
}

/// Update streaming stats display
fn update_streaming_stats_display(
    streamer: Res<WorldStreamer>,
    mut stats_query: Query<&mut Text, With<StreamingStatsDisplay>>,
) {
    for mut text in stats_query.iter_mut() {
        let stats = &streamer.stats;

        text.0 = format!(
            "Streaming Stats\nSectors: {}\nEntities: {}\nLOD0/1/I: {}/{}/{}",
            streamer.sectors.len(),
            stats.total_entities_in_world,
            stats.lod_level_counts[0],
            stats.lod_level_counts[1],
            stats.lod_level_counts[2]
        );
    }
}

/// Update memory usage display
fn update_memory_usage_display(
    streamer: Res<WorldStreamer>,
    mut memory_query: Query<(&mut Text, &mut TextColor), With<MemoryUsageDisplay>>,
) {
    for (mut text, mut text_color) in memory_query.iter_mut() {
        let memory_mb = streamer.stats.memory_usage_mb;

        text.0 = format!("Memory Usage\n{:.2} MB", memory_mb);

        // Color coding based on memory usage
        text_color.0 = if memory_mb <= 50.0 {
            Color::srgb(0.0, 1.0, 0.0) // Green for low memory
        } else if memory_mb <= 100.0 {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for medium memory
        } else {
            Color::srgb(1.0, 0.0, 0.0) // Red for high memory
        };
    }
}

/// Update performance metrics display
fn update_performance_metrics_display(
    streamer: Res<WorldStreamer>,
    mut perf_query: Query<(&mut Text, &mut TextColor), With<PerformanceMetricsDisplay>>,
) {
    for (mut text, mut text_color) in perf_query.iter_mut() {
        let stats = &streamer.stats;

        text.0 = format!(
            "Performance\nUpdate: {:.2} ms\nPeak: {:.2} ms",
            stats.average_update_time_ms, stats.peak_update_time_ms
        );

        // Color coding based on update time
        text_color.0 = if stats.average_update_time_ms <= 1.0 {
            Color::srgb(0.0, 1.0, 0.0) // Green for good performance
        } else if stats.average_update_time_ms <= 5.0 {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for medium performance
        } else {
            Color::srgb(1.0, 0.0, 0.0) // Red for poor performance
        };
    }
}
