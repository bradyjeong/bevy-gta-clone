use amp_render::diagnostics::{
    PerformanceBudgets, PerformanceDiagnosticPaths, PerformanceDiagnostics, PerformanceStatus,
};
use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

/// Unified performance statistics resource
#[derive(Resource, Default)]
pub struct PerfStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub entity_count: u32,
    pub streaming_stats: Option<StreamingStats>,
    pub gpu_culling_stats: Option<GpuCullingStats>,
}

/// Placeholder for streaming stats (will be replaced with actual import)
#[derive(Default, Clone)]
pub struct StreamingStats {
    pub chunks_loaded: u32,
    pub chunks_unloaded: u32,
    pub entities_spawned: u32,
    pub entities_despawned: u32,
    pub average_update_time_ms: f32,
    pub peak_update_time_ms: f32,
    pub sectors_loaded: u32,
    pub sectors_unloaded: u32,
    pub total_entities_in_world: u32,
    pub memory_usage_mb: f32,
    pub lod_level_counts: [u32; 3], // [LOD0, LOD1, Impostor]
}

/// Placeholder for GPU culling stats (will be replaced with actual import)
#[derive(Default, Clone)]
pub struct GpuCullingStats {
    pub instances_processed: u32,
    pub instances_visible: u32,
    pub gpu_time_ms: f32,
    pub upload_time_ms: f32,
    pub readback_time_ms: f32,
}

impl GpuCullingStats {
    pub fn culling_efficiency(&self) -> f32 {
        if self.instances_processed == 0 {
            0.0
        } else {
            1.0 - (self.instances_visible as f32 / self.instances_processed as f32)
        }
    }

    pub fn total_time_ms(&self) -> f32 {
        self.gpu_time_ms + self.upload_time_ms + self.readback_time_ms
    }
}

/// Performance UI plugin
pub struct PerfUiPlugin;

impl Plugin for PerfUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(EntityCountDiagnosticsPlugin::default())
            .add_plugins(LogDiagnosticsPlugin::default())
            .init_resource::<PerfStats>()
            .add_systems(Update, update_perf_stats)
            .add_systems(EguiPrimaryContextPass, display_perf_ui);
    }
}

/// Update performance statistics from diagnostics
fn update_perf_stats(
    mut perf_stats: ResMut<PerfStats>,
    diagnostics: Res<DiagnosticsStore>,
    entities: Query<Entity>,
) {
    // Update FPS from diagnostics
    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diag.smoothed() {
            perf_stats.fps = fps as f32;
        }
    }

    // Update frame time from diagnostics
    if let Some(frame_time_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time) = frame_time_diag.smoothed() {
            perf_stats.frame_time_ms = (frame_time * 1000.0) as f32;
        }
    }

    // Count entities
    perf_stats.entity_count = entities.iter().count() as u32;
}

/// Display performance UI overlay
fn display_perf_ui(
    mut contexts: EguiContexts,
    perf_stats: Res<PerfStats>,
    performance_diagnostics: Option<Res<PerformanceDiagnostics>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Performance Monitor")
        .default_pos(egui::pos2(10.0, 10.0))
        .default_size(egui::vec2(350.0, 500.0))
        .fixed_size(egui::vec2(350.0, 500.0))
        .collapsible(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(460.0)
                .show(ui, |ui| {
                    ui.heading("Core Metrics");

                    // Core performance metrics with color coding
                    ui.label(format!("FPS: {:.1}", perf_stats.fps));
                    ui.label(format!("Frame Time: {:.2}ms", perf_stats.frame_time_ms));
                    ui.label(format!("Entity Count: {}", perf_stats.entity_count));

                    ui.separator();

                    // Performance budgets section
                    if let Some(perf_diag) = performance_diagnostics.as_ref() {
                        ui.heading("Performance Budgets");

                        let status = perf_diag.get_status();
                        let status_color = match status {
                            PerformanceStatus::Good => egui::Color32::GREEN,
                            PerformanceStatus::Warning => egui::Color32::YELLOW,
                            PerformanceStatus::Critical => egui::Color32::RED,
                        };

                        ui.colored_label(status_color, format!("Status: {:?}", status));

                        // Draw calls
                        if let Some(draw_calls_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::DRAW_CALLS)
                        {
                            if let Some(draw_calls) = draw_calls_diag.smoothed() {
                                let color = if draw_calls <= PerformanceBudgets::MAX_DRAW_CALLS {
                                    egui::Color32::GREEN
                                } else if draw_calls <= PerformanceBudgets::ALARM_DRAW_CALLS {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "Draw Calls: {:.0} / {:.0}",
                                        draw_calls,
                                        PerformanceBudgets::MAX_DRAW_CALLS
                                    ),
                                );
                            }
                        }

                        // Instance count
                        if let Some(instance_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::INSTANCE_COUNT)
                        {
                            if let Some(instances) = instance_diag.smoothed() {
                                let color = if instances <= PerformanceBudgets::MAX_INSTANCE_COUNT {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "Instances: {:.0} / {:.0}",
                                        instances,
                                        PerformanceBudgets::MAX_INSTANCE_COUNT
                                    ),
                                );
                            }
                        }

                        // Active point lights
                        if let Some(lights_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::ACTIVE_POINT_LIGHTS)
                        {
                            if let Some(lights) = lights_diag.smoothed() {
                                let color = if lights <= PerformanceBudgets::MAX_ACTIVE_LIGHTS {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "Point Lights: {:.0} / {:.0}",
                                        lights,
                                        PerformanceBudgets::MAX_ACTIVE_LIGHTS
                                    ),
                                );
                            }
                        }

                        // Update time
                        if let Some(update_time_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::AVERAGE_UPDATE_TIME)
                        {
                            if let Some(update_time) = update_time_diag.smoothed() {
                                let color = if update_time
                                    <= PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS
                                {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "Update Time: {:.2}ms / {:.2}ms",
                                        update_time,
                                        PerformanceBudgets::MAX_AVERAGE_UPDATE_TIME_MS
                                    ),
                                );
                            }
                        }

                        // GPU culling time
                        if let Some(gpu_time_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::GPU_CULLING_TIME)
                        {
                            if let Some(gpu_time) = gpu_time_diag.smoothed() {
                                let color =
                                    if gpu_time <= PerformanceBudgets::MAX_GPU_CULLING_TIME_MS {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::RED
                                    };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "GPU Culling: {:.3}ms / {:.3}ms",
                                        gpu_time,
                                        PerformanceBudgets::MAX_GPU_CULLING_TIME_MS
                                    ),
                                );
                            }
                        }

                        // Sectors loaded
                        if let Some(sectors_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::SECTORS_LOADED)
                        {
                            if let Some(sectors) = sectors_diag.smoothed() {
                                let color = if sectors <= PerformanceBudgets::MAX_SECTORS_LOADED {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(
                                    color,
                                    format!(
                                        "Sectors Loaded: {:.0} / {:.0}",
                                        sectors,
                                        PerformanceBudgets::MAX_SECTORS_LOADED
                                    ),
                                );
                            }
                        }

                        // Queue lengths
                        if let Some(spawn_queue_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::SPAWN_QUEUE_LENGTH)
                        {
                            if let Some(spawn_queue) = spawn_queue_diag.smoothed() {
                                let color =
                                    if spawn_queue <= PerformanceBudgets::MAX_SPAWN_QUEUE_LENGTH {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::YELLOW
                                    };
                                ui.colored_label(color, format!("Spawn Queue: {:.0}", spawn_queue));
                            }
                        }

                        if let Some(despawn_queue_diag) =
                            diagnostics.get(&PerformanceDiagnosticPaths::DESPAWN_QUEUE_LENGTH)
                        {
                            if let Some(despawn_queue) = despawn_queue_diag.smoothed() {
                                let color = if despawn_queue
                                    <= PerformanceBudgets::MAX_DESPAWN_QUEUE_LENGTH
                                {
                                    egui::Color32::GREEN
                                } else {
                                    egui::Color32::YELLOW
                                };
                                ui.colored_label(
                                    color,
                                    format!("Despawn Queue: {:.0}", despawn_queue),
                                );
                            }
                        }

                        // Performance warnings
                        if !perf_diag.warnings.is_empty() {
                            ui.separator();
                            ui.heading("Performance Warnings");
                            ui.small(format!("{} warnings", perf_diag.warnings.len()));

                            for warning in perf_diag.warnings.iter().take(5) {
                                ui.colored_label(egui::Color32::YELLOW, format!("• {:?}", warning));
                            }

                            if perf_diag.warnings.len() > 5 {
                                ui.small(format!("... and {} more", perf_diag.warnings.len() - 5));
                            }
                        }

                        ui.separator();
                    }

                    // Streaming stats (if available)
                    if let Some(streaming) = &perf_stats.streaming_stats {
                        ui.heading("World Streaming");
                        ui.label(format!("Chunks Loaded: {}", streaming.chunks_loaded));
                        ui.label(format!("Chunks Unloaded: {}", streaming.chunks_unloaded));
                        ui.label(format!("Entities Spawned: {}", streaming.entities_spawned));
                        ui.label(format!(
                            "Entities Despawned: {}",
                            streaming.entities_despawned
                        ));
                        ui.label(format!(
                            "Update Time: {:.2}ms (avg)",
                            streaming.average_update_time_ms
                        ));
                        ui.label(format!(
                            "Peak Update Time: {:.2}ms",
                            streaming.peak_update_time_ms
                        ));
                        ui.label(format!("Sectors Loaded: {}", streaming.sectors_loaded));
                        ui.label(format!(
                            "Total Entities: {}",
                            streaming.total_entities_in_world
                        ));
                        ui.label(format!("Memory Usage: {:.1}MB", streaming.memory_usage_mb));
                        ui.label(format!(
                            "LOD Levels: [{}|{}|{}]",
                            streaming.lod_level_counts[0],
                            streaming.lod_level_counts[1],
                            streaming.lod_level_counts[2]
                        ));
                        ui.separator();
                    }

                    // GPU culling stats (if available)
                    if let Some(gpu_culling) = &perf_stats.gpu_culling_stats {
                        ui.heading("GPU Culling");
                        ui.label(format!(
                            "Instances Processed: {}",
                            gpu_culling.instances_processed
                        ));
                        ui.label(format!(
                            "Instances Visible: {}",
                            gpu_culling.instances_visible
                        ));
                        ui.label(format!(
                            "Culling Efficiency: {:.1}%",
                            gpu_culling.culling_efficiency() * 100.0
                        ));
                        ui.label(format!("GPU Time: {:.2}ms", gpu_culling.gpu_time_ms));
                        ui.label(format!("Upload Time: {:.2}ms", gpu_culling.upload_time_ms));
                        ui.label(format!(
                            "Readback Time: {:.2}ms",
                            gpu_culling.readback_time_ms
                        ));
                        ui.label(format!("Total Time: {:.2}ms", gpu_culling.total_time_ms()));
                        ui.separator();
                    }

                    ui.small("Phase 0: Baseline Performance Monitoring");
                });
        });
}
