//! Simplified batch processing orchestration system
//!
//! This module provides a basic implementation of the batch processing system
//! that compiles and demonstrates the core concepts.

#[cfg(test)]
mod tests;

#[cfg(feature = "bevy16")]
use bevy::ecs::system::SystemId;
#[cfg(feature = "bevy16")]
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Batch processing categories with priority ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatchType {
    Transform,
    Visibility,
    Physics,
    LOD,
    AI,
    Buildings,
    Vehicles,
    NPCs,
    Environment,
}

impl BatchType {
    pub fn priority(&self) -> u32 {
        match self {
            BatchType::Transform => 0,
            BatchType::Visibility => 1,
            BatchType::Physics => 2,
            BatchType::LOD => 3,
            BatchType::AI => 4,
            BatchType::Buildings => 5,
            BatchType::Vehicles => 6,
            BatchType::NPCs => 7,
            BatchType::Environment => 8,
        }
    }

    pub fn priority_order() -> Vec<BatchType> {
        vec![
            BatchType::Transform,
            BatchType::Visibility,
            BatchType::Physics,
            BatchType::LOD,
            BatchType::AI,
            BatchType::Buildings,
            BatchType::Vehicles,
            BatchType::NPCs,
            BatchType::Environment,
        ]
    }
}

/// Individual batch job with system and cost information
#[derive(Debug, Clone)]
pub struct BatchJob {
    #[cfg(feature = "bevy16")]
    pub system_id: SystemId,
    #[cfg(not(feature = "bevy16"))]
    pub system_id: u32, // Mock system ID
    pub batch_type: BatchType,
    pub entity_count: u32,
    pub weight_cost: f32,
    pub created_at: Instant,
}

impl BatchJob {
    #[cfg(feature = "bevy16")]
    pub fn new(
        system_id: SystemId,
        batch_type: BatchType,
        entity_count: u32,
        weight_cost: f32,
    ) -> Self {
        Self {
            system_id,
            batch_type,
            entity_count,
            weight_cost: weight_cost.clamp(0.0, 1.0),
            created_at: Instant::now(),
        }
    }

    #[cfg(not(feature = "bevy16"))]
    pub fn new(system_id: u32, batch_type: BatchType, entity_count: u32, weight_cost: f32) -> Self {
        Self {
            system_id,
            batch_type,
            entity_count,
            weight_cost: weight_cost.clamp(0.0, 1.0),
            created_at: Instant::now(),
        }
    }
}

/// Statistics for batch processing monitoring
#[derive(Debug, Default, Clone)]
pub struct BatchStats {
    pub jobs_processed: u32,
    pub jobs_deferred: u32,
    pub elapsed_time: Duration,
    pub budget_utilization: f32,
    pub jobs_per_type: HashMap<BatchType, u32>,
    pub avg_job_time: Duration,
    pub peak_queue_depth: u32,
}

/// Core batch processing controller
#[cfg_attr(feature = "bevy16", derive(Resource))]
pub struct BatchController {
    pub budget_ms: f32,
    pub queues: HashMap<BatchType, VecDeque<BatchJob>>,
    pub stats: BatchStats,
    frame_start: Option<Instant>,
}

impl Default for BatchController {
    fn default() -> Self {
        let mut queues = HashMap::new();
        for batch_type in BatchType::priority_order() {
            queues.insert(batch_type, VecDeque::new());
        }

        Self {
            budget_ms: 2.5,
            queues,
            stats: BatchStats::default(),
            frame_start: None,
        }
    }
}

impl BatchController {
    pub fn enqueue_job(&mut self, batch_type: BatchType, job: BatchJob) {
        if let Some(queue) = self.queues.get_mut(&batch_type) {
            queue.push_back(job);
            let current_depth = queue.len() as u32;
            if current_depth > self.stats.peak_queue_depth {
                self.stats.peak_queue_depth = current_depth;
            }
        }
    }

    pub fn submit_job(&mut self, job: BatchJob) {
        self.enqueue_job(job.batch_type, job);
    }

    pub fn dequeue_job(&mut self) -> Option<(BatchType, BatchJob)> {
        for batch_type in BatchType::priority_order() {
            if let Some(queue) = self.queues.get_mut(&batch_type) {
                if let Some(job) = queue.pop_front() {
                    return Some((batch_type, job));
                }
            }
        }
        None
    }

    pub fn has_budget(&self) -> bool {
        if let Some(start) = self.frame_start {
            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
            elapsed < self.budget_ms
        } else {
            true
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.stats = BatchStats::default();
        self.stats.jobs_per_type = HashMap::new();
    }

    pub fn finish_frame(&mut self) {
        if let Some(start) = self.frame_start {
            self.stats.elapsed_time = start.elapsed();
            self.stats.budget_utilization =
                (self.stats.elapsed_time.as_secs_f32() * 1000.0) / self.budget_ms;

            if self.stats.jobs_processed > 0 {
                self.stats.avg_job_time = self.stats.elapsed_time / self.stats.jobs_processed;
            }
        }
        self.frame_start = None;
    }

    pub fn total_queued_jobs(&self) -> u32 {
        self.queues.values().map(|queue| queue.len() as u32).sum()
    }

    pub fn queue_depth(&self, batch_type: BatchType) -> u32 {
        self.queues
            .get(&batch_type)
            .map_or(0, |queue| queue.len() as u32)
    }
}

/// Batch processing dispatcher system
pub fn batch_dispatcher_system(mut controller: ResMut<BatchController>) {
    controller.start_frame();

    while controller.has_budget() {
        if let Some((batch_type, job)) = controller.dequeue_job() {
            // Simulate job execution
            let execution_time = Duration::from_secs_f32(job.weight_cost * 0.001);
            std::thread::sleep(execution_time);

            controller.stats.jobs_processed += 1;
            *controller
                .stats
                .jobs_per_type
                .entry(batch_type)
                .or_insert(0) += 1;
        } else {
            break;
        }
    }

    controller.stats.jobs_deferred = controller.total_queued_jobs();
    controller.finish_frame();
}

/// Helper function to register batch system
pub fn register_batch_system(
    controller: &mut BatchController,
    batch_type: BatchType,
    system_id: SystemId,
    cost: f32,
) {
    let job = BatchJob::new(system_id, batch_type, 1, cost);
    controller.enqueue_job(batch_type, job);
}

/// Basic batch processing plugin
pub struct BatchProcessingPlugin;

impl Plugin for BatchProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatchController>()
            .add_systems(FixedUpdate, batch_dispatcher_system);
    }
}
