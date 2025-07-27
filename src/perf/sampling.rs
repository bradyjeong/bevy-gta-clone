use bevy::diagnostic::{Diagnostic, Diagnostics};
use bevy::prelude::*;

use crate::perf::{
    diagnostics::*,
    resources::{GlobalPerformance, PerfThresholds},
    PerfStatus,
};

/// Sample frame time diagnostics and update global performance
pub fn sample_frame_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    thresholds: Res<PerfThresholds>,
    time: Res<Time>,
) {
    // Add frame time measurement to Bevy diagnostics
    let frame_time_ms = time.delta_secs_f64() * 1000.0;
    diagnostics.add_measurement(&FRAME_TIME, || frame_time_ms);

    // Update global performance tracking
    global_perf.frame_time_ms = frame_time_ms;
    global_perf.frame_status = PerfStatus::from_frame_time(frame_time_ms, &thresholds);
    global_perf.update_overall_status();
}

/// Sample physics timestep diagnostics
pub fn sample_physics_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    thresholds: Res<PerfThresholds>,
    time: Res<Time<Fixed>>,
) {
    // Add physics timestep measurement to Bevy diagnostics
    let physics_time_ms = time.delta_secs_f64() * 1000.0;
    diagnostics.add_measurement(&PHYSICS_TIMESTEP, || physics_time_ms);

    // Update global performance tracking
    global_perf.physics_time_ms = physics_time_ms;
    global_perf.physics_status = PerfStatus::from_physics_time(physics_time_ms, &thresholds);
    global_perf.update_overall_status();
}

/// Sample render time diagnostics
pub fn sample_render_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    thresholds: Res<PerfThresholds>,
) {
    // Note: This is a simplified render time sampling
    // In a real implementation, you'd measure actual render pipeline timing
    // For now, we'll estimate based on frame time - physics time
    let estimated_render_time = global_perf.frame_time_ms - global_perf.physics_time_ms;
    let render_time_ms = estimated_render_time.max(0.0);

    diagnostics.add_measurement(&RENDER_TIME, || render_time_ms);

    // Update global performance tracking
    global_perf.render_time_ms = render_time_ms;
    global_perf.render_status = PerfStatus::from_render_time(render_time_ms, &thresholds);
    global_perf.update_overall_status();
}

/// Sample entity count diagnostics
pub fn sample_entity_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    query: Query<Entity>,
) {
    let entity_count = query.iter().count() as u32;

    diagnostics.add_measurement(&ENTITY_COUNT, || entity_count as f64);
    global_perf.entity_count = entity_count;
}

/// Sample NPC count diagnostics
pub fn sample_npc_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    // This would use actual NPC component query in real implementation
    // For now, using a placeholder
) {
    // Placeholder - in real implementation this would query NPC components
    let npc_count = 0u32;

    diagnostics.add_measurement(&NPC_COUNT, || npc_count as f64);
    global_perf.npc_count = npc_count;
}

/// Sample vehicle count diagnostics
pub fn sample_vehicle_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    // This would use actual NPC component query in real implementation
    // For now, using a placeholder
) {
    // Placeholder - in real implementation this would query Vehicle components
    let vehicle_count = 0u32;

    diagnostics.add_measurement(&VEHICLE_COUNT, || vehicle_count as f64);
    global_perf.vehicle_count = vehicle_count;
}

/// Sample building count diagnostics
pub fn sample_building_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    // This would use actual Building component query in real implementation
    // For now, using a placeholder
) {
    // Placeholder - in real implementation this would query Building components
    let building_count = 0u32;

    diagnostics.add_measurement(&BUILDING_COUNT, || building_count as f64);
    global_perf.building_count = building_count;
}

/// Sample GPU culling diagnostics (feature-gated)
#[cfg(feature = "gpu_culling")]
pub fn sample_gpu_culling_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
    // This would access actual GPU culling metrics in real implementation
) {
    // Placeholder - in real implementation this would measure GPU culling performance
    let gpu_culling_time_ms = 0.0;
    let culled_objects = 0u32;
    let visible_objects = 0u32;

    diagnostics.add_measurement(&GPU_CULLING_TIME, || gpu_culling_time_ms);
    diagnostics.add_measurement(&CULLED_OBJECTS, || culled_objects as f64);
    diagnostics.add_measurement(&VISIBLE_OBJECTS, || visible_objects as f64);

    global_perf.gpu_culling_time_ms = gpu_culling_time_ms;
}

/// Sample memory usage diagnostics
pub fn sample_memory_diagnostics(
    mut diagnostics: Diagnostics,
    mut global_perf: ResMut<GlobalPerformance>,
) {
    // Placeholder - in real implementation this would measure actual memory usage
    // You'd use platform-specific APIs or memory profiling crates
    let heap_usage_mb = 0.0;
    let gpu_memory_mb = 0.0;

    diagnostics.add_measurement(&HEAP_USAGE, || heap_usage_mb);
    diagnostics.add_measurement(&GPU_MEMORY_USAGE, || gpu_memory_mb);

    global_perf.heap_usage_mb = heap_usage_mb;
    global_perf.gpu_memory_mb = gpu_memory_mb;
}
