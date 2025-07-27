use bevy::diagnostic::DiagnosticPath;

// Core performance diagnostic paths
pub const FRAME_TIME: DiagnosticPath = DiagnosticPath::const_new("frame_time");
pub const PHYSICS_TIMESTEP: DiagnosticPath = DiagnosticPath::const_new("physics_timestep");
pub const RENDER_TIME: DiagnosticPath = DiagnosticPath::const_new("render_time");
pub const GPU_MEMORY_USAGE: DiagnosticPath = DiagnosticPath::const_new("gpu_memory_usage");
pub const ENTITY_COUNT: DiagnosticPath = DiagnosticPath::const_new("entity_count");
pub const SYSTEM_COUNT: DiagnosticPath = DiagnosticPath::const_new("system_count");

// Gameplay-specific diagnostics
pub const VEHICLE_COUNT: DiagnosticPath = DiagnosticPath::const_new("vehicle_count");
pub const NPC_COUNT: DiagnosticPath = DiagnosticPath::const_new("npc_count");
pub const BUILDING_COUNT: DiagnosticPath = DiagnosticPath::const_new("building_count");
pub const CHUNK_LOAD_TIME: DiagnosticPath = DiagnosticPath::const_new("chunk_load_time");

// GPU culling diagnostics (when feature enabled)
pub const GPU_CULLING_TIME: DiagnosticPath = DiagnosticPath::const_new("gpu_culling_time");
pub const CULLED_OBJECTS: DiagnosticPath = DiagnosticPath::const_new("culled_objects");
pub const VISIBLE_OBJECTS: DiagnosticPath = DiagnosticPath::const_new("visible_objects");

// Memory diagnostics
pub const HEAP_USAGE: DiagnosticPath = DiagnosticPath::const_new("heap_usage");
pub const POOL_ALLOCATIONS: DiagnosticPath = DiagnosticPath::const_new("pool_allocations");
pub const FRAME_ALLOCATIONS: DiagnosticPath = DiagnosticPath::const_new("frame_allocations");

// Performance thresholds (in milliseconds)
pub const TARGET_FRAME_TIME_MS: f64 = 16.67; // 60 FPS
pub const WARN_FRAME_TIME_MS: f64 = 20.0; // 50 FPS
pub const CRIT_FRAME_TIME_MS: f64 = 33.33; // 30 FPS

pub const TARGET_PHYSICS_TIME_MS: f64 = 2.0;
pub const WARN_PHYSICS_TIME_MS: f64 = 4.0;
pub const CRIT_PHYSICS_TIME_MS: f64 = 8.0;

pub const TARGET_RENDER_TIME_MS: f64 = 10.0;
pub const WARN_RENDER_TIME_MS: f64 = 15.0;
pub const CRIT_RENDER_TIME_MS: f64 = 25.0;

pub const TARGET_GPU_CULLING_TIME_MS: f64 = 0.25;
pub const WARN_GPU_CULLING_TIME_MS: f64 = 0.5;
pub const CRIT_GPU_CULLING_TIME_MS: f64 = 1.0;

// Memory thresholds (in MB)
pub const WARN_HEAP_USAGE_MB: f64 = 1024.0;
pub const CRIT_HEAP_USAGE_MB: f64 = 2048.0;

pub const WARN_GPU_MEMORY_MB: f64 = 512.0;
pub const CRIT_GPU_MEMORY_MB: f64 = 1024.0;

// Entity count thresholds
pub const WARN_ENTITY_COUNT: u32 = 100_000;
pub const CRIT_ENTITY_COUNT: u32 = 500_000;

pub const WARN_NPC_COUNT: u32 = 1_000;
pub const CRIT_NPC_COUNT: u32 = 5_000;

pub const WARN_VEHICLE_COUNT: u32 = 500;
pub const CRIT_VEHICLE_COUNT: u32 = 2_000;
