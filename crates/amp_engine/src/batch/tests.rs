//! Tests for the batch processing system

use super::*;
use bevy::prelude::*;

#[test]
fn test_batch_type_priority() {
    assert_eq!(BatchType::Transform.priority(), 0);
    assert_eq!(BatchType::Visibility.priority(), 1);
    assert_eq!(BatchType::Physics.priority(), 2);
    assert_eq!(BatchType::LOD.priority(), 3);
    assert_eq!(BatchType::AI.priority(), 4);
}

#[test]
fn test_batch_type_priority_order() {
    let order = BatchType::priority_order();
    assert_eq!(order.len(), 5);
    assert_eq!(order[0], BatchType::Transform);
    assert_eq!(order[1], BatchType::Visibility);
    assert_eq!(order[2], BatchType::Physics);
    assert_eq!(order[3], BatchType::LOD);
    assert_eq!(order[4], BatchType::AI);
}

#[test]
fn test_batch_job_creation() {
    let entity = Entity::from_raw(1);
    let system_id = SystemId::from_entity(entity);
    let job = BatchJob::new(system_id, 0.5);

    assert_eq!(job.system_id, system_id);
    assert_eq!(job.weight_cost, 0.5);
}

#[test]
fn test_batch_job_cost_clamping() {
    let entity = Entity::from_raw(1);
    let system_id = SystemId::from_entity(entity);
    let job_high = BatchJob::new(system_id, 2.0);
    let job_low = BatchJob::new(system_id, -1.0);

    assert_eq!(job_high.weight_cost, 1.0);
    assert_eq!(job_low.weight_cost, 0.0);
}

#[test]
fn test_batch_controller_default() {
    let controller = BatchController::default();

    assert_eq!(controller.budget_ms, 2.5);
    assert_eq!(controller.queues.len(), 5);
    assert_eq!(controller.stats.jobs_processed, 0);
    assert_eq!(controller.total_queued_jobs(), 0);
}

#[test]
fn test_queue_operations() {
    let mut controller = BatchController::default();
    let entity = Entity::from_raw(1);
    let system_id = SystemId::from_entity(entity);
    let job = BatchJob::new(system_id, 0.5);

    controller.enqueue_job(BatchType::Transform, job.clone());
    assert_eq!(controller.total_queued_jobs(), 1);
    assert_eq!(controller.queue_depth(BatchType::Transform), 1);

    let (batch_type, dequeued_job) = controller.dequeue_job().unwrap();
    assert_eq!(batch_type, BatchType::Transform);
    assert_eq!(dequeued_job.system_id, system_id);
    assert_eq!(controller.total_queued_jobs(), 0);
}

#[test]
fn test_priority_ordering() {
    let mut controller = BatchController::default();
    let entity = Entity::from_raw(1);
    let system_id = SystemId::from_entity(entity);

    controller.enqueue_job(BatchType::AI, BatchJob::new(system_id, 0.1));
    controller.enqueue_job(BatchType::Transform, BatchJob::new(system_id, 0.2));
    controller.enqueue_job(BatchType::Physics, BatchJob::new(system_id, 0.3));

    let (batch_type, _) = controller.dequeue_job().unwrap();
    assert_eq!(batch_type, BatchType::Transform);

    let (batch_type, _) = controller.dequeue_job().unwrap();
    assert_eq!(batch_type, BatchType::Physics);

    let (batch_type, _) = controller.dequeue_job().unwrap();
    assert_eq!(batch_type, BatchType::AI);
}

#[test]
fn test_fifo_within_queue() {
    let mut controller = BatchController::default();
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let system_id1 = SystemId::from_entity(entity1);
    let system_id2 = SystemId::from_entity(entity2);

    controller.enqueue_job(BatchType::Transform, BatchJob::new(system_id1, 0.1));
    controller.enqueue_job(BatchType::Transform, BatchJob::new(system_id2, 0.2));

    let (_, job1) = controller.dequeue_job().unwrap();
    assert_eq!(job1.system_id, system_id1);

    let (_, job2) = controller.dequeue_job().unwrap();
    assert_eq!(job2.system_id, system_id2);
}

#[test]
fn test_budget_tracking() {
    let mut controller = BatchController::default();

    assert!(controller.has_budget());

    controller.start_frame();
    assert!(controller.has_budget());

    controller.budget_ms = 0.0001;
    std::thread::sleep(Duration::from_millis(1));
    assert!(!controller.has_budget());
}

#[test]
fn test_stats_tracking() {
    let mut controller = BatchController::default();

    controller.start_frame();
    controller.stats.jobs_processed = 5;
    controller.stats.jobs_deferred = 3;
    controller
        .stats
        .jobs_per_type
        .insert(BatchType::Transform, 3);
    controller.stats.jobs_per_type.insert(BatchType::Physics, 2);
    controller.finish_frame();

    assert_eq!(controller.stats.jobs_processed, 5);
    assert_eq!(controller.stats.jobs_deferred, 3);
    assert!(controller.stats.elapsed_time > Duration::from_nanos(0));
    assert!(controller.stats.budget_utilization >= 0.0);
}

#[test]
fn test_register_batch_system() {
    let mut controller = BatchController::default();
    let entity = Entity::from_raw(1);
    let system_id = SystemId::from_entity(entity);

    register_batch_system(&mut controller, BatchType::Transform, system_id, 0.5);

    assert_eq!(controller.total_queued_jobs(), 1);
    assert_eq!(controller.queue_depth(BatchType::Transform), 1);

    let (batch_type, job) = controller.dequeue_job().unwrap();
    assert_eq!(batch_type, BatchType::Transform);
    assert_eq!(job.system_id, system_id);
    assert_eq!(job.weight_cost, 0.5);
}
