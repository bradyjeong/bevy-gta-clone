use crate::perf::{resources::GlobalPerformance, PerfStatus};
use bevy::prelude::*;

/// Rate-limited performance logging system
/// Only logs when performance status changes and respects cooldown period
pub fn rate_limited_perf_logger(mut global_perf: ResMut<GlobalPerformance>) {
    // Only log if status should be logged (handles rate limiting internally)
    if global_perf.should_log_status() {
        log_performance_status(&global_perf);
        global_perf.mark_status_logged();
    }
}

/// Log current performance status with appropriate severity level
fn log_performance_status(global_perf: &GlobalPerformance) {
    let status_str = match global_perf.overall_status {
        PerfStatus::Good => "GOOD",
        PerfStatus::Warn => "WARN",
        PerfStatus::Crit => "CRIT",
    };

    let frame_fps = if global_perf.frame_time_ms > 0.0 {
        1000.0 / global_perf.frame_time_ms
    } else {
        0.0
    };

    match global_perf.overall_status {
        PerfStatus::Good => {
            info!(
                "Performance: {} | Frame: {:.1}ms ({:.1}fps) | Physics: {:.2}ms | Render: {:.2}ms | Entities: {}",
                status_str,
                global_perf.frame_time_ms,
                frame_fps,
                global_perf.physics_time_ms,
                global_perf.render_time_ms,
                global_perf.entity_count
            );
        }
        PerfStatus::Warn => {
            warn!(
                "Performance: {} | Frame: {:.1}ms ({:.1}fps) | Physics: {:.2}ms | Render: {:.2}ms | Entities: {} | NPCs: {} | Vehicles: {}",
                status_str,
                global_perf.frame_time_ms,
                frame_fps,
                global_perf.physics_time_ms,
                global_perf.render_time_ms,
                global_perf.entity_count,
                global_perf.npc_count,
                global_perf.vehicle_count
            );
        }
        PerfStatus::Crit => {
            error!(
                "Performance: {} | Frame: {:.1}ms ({:.1}fps) | Physics: {:.2}ms | Render: {:.2}ms | Entities: {} | NPCs: {} | Vehicles: {} | Memory: {:.1}MB heap, {:.1}MB GPU",
                status_str,
                global_perf.frame_time_ms,
                frame_fps,
                global_perf.physics_time_ms,
                global_perf.render_time_ms,
                global_perf.entity_count,
                global_perf.npc_count,
                global_perf.vehicle_count,
                global_perf.heap_usage_mb,
                global_perf.gpu_memory_mb
            );
        }
    }

    // Log GPU culling metrics if available
    #[cfg(feature = "gpu_culling")]
    if global_perf.gpu_culling_time_ms > 0.0 {
        match global_perf.overall_status {
            PerfStatus::Good => {
                debug!("GPU Culling: {:.3}ms", global_perf.gpu_culling_time_ms);
            }
            _ => {
                info!("GPU Culling: {:.3}ms", global_perf.gpu_culling_time_ms);
            }
        }
    }
}

/// Debug logging system for detailed performance metrics
/// Only active in debug builds or when explicitly enabled
pub fn debug_perf_logger(global_perf: Res<GlobalPerformance>) {
    // Only log debug info every few seconds to avoid spam
    if cfg!(debug_assertions) || cfg!(feature = "perf_debug_logging") {
        // This would implement periodic detailed logging
        // For now, it's a placeholder for future detailed metrics
        trace!(
            "Debug Perf: Overall={:?}, Frame={:?}, Physics={:?}, Render={:?}",
            global_perf.overall_status,
            global_perf.frame_status,
            global_perf.physics_status,
            global_perf.render_status
        );
    }
}

/// Performance alert system for critical issues
/// Sends structured alerts that could be integrated with external monitoring
pub fn perf_alert_system(global_perf: Res<GlobalPerformance>) {
    // Only trigger alerts for critical performance issues
    if global_perf.overall_status == PerfStatus::Crit {
        // This could be extended to send alerts to external systems
        // For now, it's a structured error log
        error!(
            target: "perf_alert",
            "CRITICAL PERFORMANCE ALERT: frame_time={:.2}ms physics_time={:.2}ms render_time={:.2}ms entities={}",
            global_perf.frame_time_ms,
            global_perf.physics_time_ms,
            global_perf.render_time_ms,
            global_perf.entity_count
        );
    }
}

/// Export performance metrics in structured format
/// Useful for CI/CD performance gates and automated testing
pub fn export_perf_metrics(global_perf: Res<GlobalPerformance>) -> serde_json::Value {
    serde_json::json!({
        "overall_status": format!("{:?}", global_perf.overall_status),
        "frame_time_ms": global_perf.frame_time_ms,
        "physics_time_ms": global_perf.physics_time_ms,
        "render_time_ms": global_perf.render_time_ms,
        "gpu_culling_time_ms": global_perf.gpu_culling_time_ms,
        "entity_count": global_perf.entity_count,
        "npc_count": global_perf.npc_count,
        "vehicle_count": global_perf.vehicle_count,
        "building_count": global_perf.building_count,
        "heap_usage_mb": global_perf.heap_usage_mb,
        "gpu_memory_mb": global_perf.gpu_memory_mb,
        "frame_status": format!("{:?}", global_perf.frame_status),
        "physics_status": format!("{:?}", global_perf.physics_status),
        "render_status": format!("{:?}", global_perf.render_status)
    })
}
