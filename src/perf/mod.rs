use bevy::prelude::*;

pub mod diagnostics;
pub mod logging;
pub mod resources;
pub mod sampling;

pub use diagnostics::*;
pub use logging::*;
pub use resources::*;
pub use sampling::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfStatus {
    Good,
    Warn,
    Crit,
}

impl PerfStatus {
    pub fn from_frame_time(frame_time_ms: f64, thresholds: &PerfThresholds) -> Self {
        if frame_time_ms >= thresholds.frame_time_crit_ms {
            PerfStatus::Crit
        } else if frame_time_ms >= thresholds.frame_time_warn_ms {
            PerfStatus::Warn
        } else {
            PerfStatus::Good
        }
    }

    pub fn from_physics_time(physics_time_ms: f64, thresholds: &PerfThresholds) -> Self {
        if physics_time_ms >= thresholds.physics_time_crit_ms {
            PerfStatus::Crit
        } else if physics_time_ms >= thresholds.physics_time_warn_ms {
            PerfStatus::Warn
        } else {
            PerfStatus::Good
        }
    }

    pub fn from_render_time(render_time_ms: f64, thresholds: &PerfThresholds) -> Self {
        if render_time_ms >= thresholds.render_time_crit_ms {
            PerfStatus::Crit
        } else if render_time_ms >= thresholds.render_time_warn_ms {
            PerfStatus::Warn
        } else {
            PerfStatus::Good
        }
    }

    pub fn worst_of(&self, other: PerfStatus) -> PerfStatus {
        match (self, other) {
            (PerfStatus::Crit, _) | (_, PerfStatus::Crit) => PerfStatus::Crit,
            (PerfStatus::Warn, _) | (_, PerfStatus::Warn) => PerfStatus::Warn,
            (PerfStatus::Good, PerfStatus::Good) => PerfStatus::Good,
        }
    }
}

pub struct PerfMonitoringPlugin;

impl Plugin for PerfMonitoringPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "perf_monitoring")]
        {
            app.insert_resource(PerfThresholds::default())
                .insert_resource(GlobalPerformance::default())
                .add_systems(
                    Update,
                    (
                        sample_frame_diagnostics,
                        sample_physics_diagnostics,
                        sample_render_diagnostics,
                        rate_limited_perf_logger,
                    ),
                );
        }

        #[cfg(not(feature = "perf_monitoring"))]
        {
            // Performance monitoring disabled
            info!("Performance monitoring disabled (feature 'perf_monitoring' not enabled)");
        }
    }
}
