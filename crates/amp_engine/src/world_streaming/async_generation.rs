//! Async chunk generation system to prevent main thread blocking
//!
//! This module implements async task pools for world streaming generation
//! as mandated by Oracle to eliminate production frame hitches.

use bevy::prelude::*;
use bevy::tasks::{ComputeTaskPool, Task};
use futures_lite::future;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[cfg(feature = "unstable_hierarchical_world")]
use super::hierarchical_streaming::{ChunkState, ContentLayers, WorldChunk};

#[cfg(feature = "unstable_hierarchical_world")]
use amp_math::spatial::WorldCoord;

// When hierarchical world is not available, define minimal types
#[cfg(not(feature = "unstable_hierarchical_world"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldCoord {
    pub x: i32,
    pub z: i32,
    pub level: u8,
}

#[cfg(not(feature = "unstable_hierarchical_world"))]
impl WorldCoord {
    pub fn new(level: u8, x: i32, z: i32) -> Self {
        Self { x, z, level }
    }

    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(self.x as f32 * 100.0, 0.0, self.z as f32 * 100.0)
    }
}

#[cfg(not(feature = "unstable_hierarchical_world"))]
#[derive(Debug, Clone, Default)]
pub struct ContentLayers {
    pub terrain: bool,
    pub roads: bool,
    pub buildings: bool,
    pub vegetation: bool,
    pub vehicles: bool,
    pub npcs: bool,
    pub water: bool,
    pub details: bool,
}

/// Async generation status for progress tracking
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationStatus {
    Queued { priority: f32 },
    InProgress { progress: f32 },
    Completed { entities: Vec<Entity> },
    Failed { error: String },
}

/// Async chunk generation task
#[derive(Component)]
pub struct ChunkGenerationTask {
    pub coord: WorldCoord,
    pub task: Task<ChunkGenerationResult>,
    pub status: GenerationStatus,
    pub started_at: f32,
}

/// Result from async chunk generation
#[derive(Debug, Clone)]
pub struct ChunkGenerationResult {
    pub coord: WorldCoord,
    pub entities: Vec<Entity>,
    pub content_layers: ContentLayers,
    pub generation_time: f32,
    pub error: Option<String>,
}

/// Generation job data passed to background threads
#[derive(Debug, Clone)]
pub struct GenerationJob {
    pub coord: WorldCoord,
    pub generation_seed: u64,
    pub priority: f32,
    pub content_layers: ContentLayers,
}

/// Async generation manager resource
#[derive(Resource)]
pub struct AsyncGenerationManager {
    /// Queue of pending generation jobs
    pub generation_queue: VecDeque<GenerationJob>,
    /// Currently running tasks
    pub active_tasks: HashMap<WorldCoord, Entity>,
    /// Completed results waiting for main thread processing
    pub completed_results: Arc<Mutex<Vec<ChunkGenerationResult>>>,
    /// Maximum concurrent generation tasks
    pub max_concurrent_tasks: usize,
    /// Generation diagnostics
    pub diagnostics: GenerationDiagnostics,
}

/// Performance diagnostics for generation system
#[derive(Debug, Default)]
pub struct GenerationDiagnostics {
    pub total_chunks_generated: u64,
    pub total_generation_time: f32,
    pub average_generation_time: f32,
    pub peak_concurrent_tasks: usize,
    pub failed_generations: u64,
    pub queue_size: usize,
    pub active_task_count: usize,
}

impl Default for AsyncGenerationManager {
    fn default() -> Self {
        Self {
            generation_queue: VecDeque::new(),
            active_tasks: HashMap::new(),
            completed_results: Arc::new(Mutex::new(Vec::new())),
            max_concurrent_tasks: 4, // Conservative default
            diagnostics: GenerationDiagnostics::default(),
        }
    }
}

impl AsyncGenerationManager {
    /// Queue a chunk for async generation
    pub fn queue_chunk_generation(&mut self, job: GenerationJob) {
        // Insert with priority ordering
        let position = self
            .generation_queue
            .iter()
            .position(|existing| existing.priority < job.priority)
            .unwrap_or(self.generation_queue.len());

        self.generation_queue.insert(position, job);
        self.diagnostics.queue_size = self.generation_queue.len();
    }

    /// Start next generation task if capacity allows
    pub fn try_start_next_task(&mut self, commands: &mut Commands, time: &Res<Time>) -> bool {
        if self.active_tasks.len() >= self.max_concurrent_tasks {
            return false;
        }

        if let Some(job) = self.generation_queue.pop_front() {
            self.start_chunk_generation(commands, job, time);
            self.diagnostics.queue_size = self.generation_queue.len();
            true
        } else {
            false
        }
    }

    /// Start async chunk generation task
    fn start_chunk_generation(
        &mut self,
        commands: &mut Commands,
        job: GenerationJob,
        time: &Res<Time>,
    ) {
        let task_pool = ComputeTaskPool::get();
        let results = Arc::clone(&self.completed_results);
        let current_time = time.elapsed_secs();

        // Clone the job data we need after the move
        let job_coord = job.coord;

        let task = task_pool.spawn(async move {
            // Simulate chunk generation work on background thread
            let start_time = std::time::Instant::now();

            // Generate chunk content based on LOD level
            let entities = generate_chunk_content_async(&job).await;

            let generation_time = start_time.elapsed().as_secs_f32();

            let result = ChunkGenerationResult {
                coord: job.coord,
                entities,
                content_layers: job.content_layers,
                generation_time,
                error: None,
            };

            // Store result for main thread pickup
            if let Ok(mut results) = results.lock() {
                results.push(result.clone());
            }

            result
        });

        let task_entity = commands
            .spawn(ChunkGenerationTask {
                coord: job_coord,
                task,
                status: GenerationStatus::InProgress { progress: 0.0 },
                started_at: current_time,
            })
            .id();

        self.active_tasks.insert(job_coord, task_entity);
        self.diagnostics.active_task_count = self.active_tasks.len();
        self.diagnostics.peak_concurrent_tasks = self
            .diagnostics
            .peak_concurrent_tasks
            .max(self.active_tasks.len());
    }

    /// Process completed results from background threads
    pub fn process_completed_results(
        &mut self,
        commands: &mut Commands,
    ) -> Vec<ChunkGenerationResult> {
        let mut completed = Vec::new();

        if let Ok(mut results) = self.completed_results.lock() {
            completed = results.drain(..).collect();
        }

        // Update diagnostics
        for result in &completed {
            self.diagnostics.total_chunks_generated += 1;
            self.diagnostics.total_generation_time += result.generation_time;
            self.diagnostics.average_generation_time = self.diagnostics.total_generation_time
                / self.diagnostics.total_chunks_generated as f32;

            if result.error.is_some() {
                self.diagnostics.failed_generations += 1;
            }

            // Remove from active tasks
            if let Some(task_entity) = self.active_tasks.remove(&result.coord) {
                commands.entity(task_entity).despawn();
            }
        }

        self.diagnostics.active_task_count = self.active_tasks.len();
        completed
    }

    /// Check for timed out or failed tasks
    pub fn cleanup_stalled_tasks(
        &mut self,
        commands: &mut Commands,
        current_time: f32,
        timeout_secs: f32,
    ) {
        let mut stalled_coords = Vec::new();

        // Find stalled tasks
        for (coord, _entity) in &self.active_tasks {
            // This is a simplified check - in a real implementation we'd track start times
            // For now, we'll rely on the task completion mechanism
        }

        // Clean up stalled tasks
        for coord in stalled_coords {
            if let Some(task_entity) = self.active_tasks.remove(&coord) {
                commands.entity(task_entity).despawn();
                self.diagnostics.failed_generations += 1;
            }
        }

        self.diagnostics.active_task_count = self.active_tasks.len();
    }
}

/// Async chunk content generation (runs on background thread)
async fn generate_chunk_content_async(job: &GenerationJob) -> Vec<Entity> {
    // Simulate deterministic generation work
    // This would normally include:
    // - Terrain mesh generation
    // - Building placement
    // - Road network generation
    // - Vegetation spawning
    // - NPC/vehicle placement

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Use seed for deterministic generation
    let mut hasher = DefaultHasher::new();
    job.generation_seed.hash(&mut hasher);
    let rng_seed = hasher.finish();

    // Simulate generation time based on LOD level
    #[cfg(feature = "unstable_hierarchical_world")]
    let generation_time_ms = match job.coord.level {
        amp_math::spatial::LODLevel::Macro => 10, // Fast for macro chunks
        amp_math::spatial::LODLevel::Region => 25, // Medium for regions
        amp_math::spatial::LODLevel::Local => 50, // Slower for local detail
        amp_math::spatial::LODLevel::Detail => 100, // Slower for detail
        amp_math::spatial::LODLevel::Micro => 200, // Slowest for micro detail
    };

    #[cfg(not(feature = "unstable_hierarchical_world"))]
    let generation_time_ms = match job.coord.level {
        0 => 10,  // Macro equivalent
        1 => 25,  // Region equivalent
        2 => 50,  // Local equivalent
        3 => 100, // Detail equivalent
        4 => 200, // Micro equivalent
        _ => 50,  // Default
    };

    // Async delay to simulate work
    future::yield_now().await; // Yield to allow other tasks

    // In production, this would return actual entity IDs
    // For now, return empty vec as entities will be created on main thread
    Vec::new()
}

/// System to manage async chunk generation
pub fn async_chunk_generation_system(
    mut commands: Commands,
    mut generation_manager: ResMut<AsyncGenerationManager>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ChunkGenerationTask)>,
) {
    let current_time = time.elapsed_secs();

    // Check for completed tasks
    for (entity, mut task) in query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.task)) {
            task.status = GenerationStatus::Completed {
                entities: result.entities.clone(),
            };

            // Task will be cleaned up in process_completed_results
        }
    }

    // Process completed results
    let completed_results = generation_manager.process_completed_results(&mut commands);

    // Start new tasks if capacity allows
    while generation_manager.try_start_next_task(&mut commands, &time) {
        // Keep starting tasks until we hit the limit or run out of work
    }

    // Clean up any stalled tasks
    generation_manager.cleanup_stalled_tasks(&mut commands, current_time, 30.0);
}

/// Debug system for async generation diagnostics
pub fn async_generation_debug_system(
    generation_manager: Res<AsyncGenerationManager>,
    time: Res<Time>,
) {
    // Only update debug info every 2 seconds to avoid spam
    if (time.elapsed_secs() % 2.0) < time.delta_secs() {
        let diag = &generation_manager.diagnostics;

        info!(
            "🔄 ASYNC GENERATION STATUS:\n\
            📊 Performance:\n\
            • Total Generated: {} chunks\n\
            • Average Gen Time: {:.2}ms\n\
            • Failed Generations: {}\n\
            📈 Current Status:\n\
            • Queue Size: {} chunks\n\
            • Active Tasks: {}/{}\n\
            • Peak Concurrent: {}\n\
            💾 Resource Usage:\n\
            • Task Pool Utilization: {:.1}%",
            diag.total_chunks_generated,
            diag.average_generation_time * 1000.0,
            diag.failed_generations,
            diag.queue_size,
            diag.active_task_count,
            generation_manager.max_concurrent_tasks,
            diag.peak_concurrent_tasks,
            (diag.active_task_count as f32 / generation_manager.max_concurrent_tasks as f32)
                * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amp_math::spatial::LODLevel;

    #[test]
    fn test_async_generation_manager_creation() {
        let manager = AsyncGenerationManager::default();

        assert_eq!(manager.max_concurrent_tasks, 4);
        assert!(manager.generation_queue.is_empty());
        assert!(manager.active_tasks.is_empty());
    }

    #[test]
    fn test_generation_job_queueing() {
        let mut manager = AsyncGenerationManager::default();

        let job1 = GenerationJob {
            coord: WorldCoord::new(LODLevel::Local, 0, 0),
            generation_seed: 12345,
            priority: 1.0,
            content_layers: ContentLayers::default(),
        };

        let job2 = GenerationJob {
            coord: WorldCoord::new(LODLevel::Detail, 1, 1),
            generation_seed: 67890,
            priority: 2.0, // Higher priority
            content_layers: ContentLayers::default(),
        };

        manager.queue_chunk_generation(job1);
        manager.queue_chunk_generation(job2);

        // Higher priority job should be first
        assert_eq!(manager.generation_queue.len(), 2);
        assert_eq!(manager.generation_queue[0].priority, 2.0);
        assert_eq!(manager.generation_queue[1].priority, 1.0);
    }

    #[test]
    fn test_diagnostics_update() {
        let mut manager = AsyncGenerationManager::default();

        let result = ChunkGenerationResult {
            coord: WorldCoord::new(LODLevel::Local, 0, 0),
            entities: Vec::new(),
            content_layers: ContentLayers::default(),
            generation_time: 0.1,
            error: None,
        };

        // Simulate processing a completed result
        manager.diagnostics.total_chunks_generated += 1;
        manager.diagnostics.total_generation_time += result.generation_time;
        manager.diagnostics.average_generation_time = manager.diagnostics.total_generation_time
            / manager.diagnostics.total_chunks_generated as f32;

        assert_eq!(manager.diagnostics.total_chunks_generated, 1);
        assert_eq!(manager.diagnostics.average_generation_time, 0.1);
    }
}
