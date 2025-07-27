//! Async road mesh generation to prevent main thread blocking
//!
//! This module implements async task pools for road mesh generation
//! as mandated by Oracle to eliminate production frame hitches during
//! complex road network generation.

use amp_math::spline::Spline;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::tasks::{ComputeTaskPool, Task};
use futures_lite::future;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::mesh_generation::{IntersectionType, MarkingParams, RoadMeshParams};

/// Async road mesh generation status
#[derive(Debug, Clone, PartialEq)]
pub enum MeshGenerationStatus {
    Queued { priority: f32 },
    InProgress { progress: f32 },
    Completed { mesh_data: MeshData },
    Failed { error: String },
}

/// Serializable mesh data for async generation
#[derive(Debug, Clone, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub primitive_topology: PrimitiveTopology,
}

impl MeshData {
    /// Convert MeshData to Bevy Mesh (must be called on main thread)
    pub fn to_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(self.primitive_topology, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Async road mesh generation task
#[derive(Component)]
pub struct RoadMeshGenerationTask {
    pub road_id: String,
    pub task: Task<RoadMeshGenerationResult>,
    pub status: MeshGenerationStatus,
    pub started_at: f32,
}

/// Result from async road mesh generation
#[derive(Debug, Clone)]
pub struct RoadMeshGenerationResult {
    pub road_id: String,
    pub road_entity: Entity,
    pub main_mesh: MeshData,
    pub marking_meshes: Vec<MeshData>,
    pub generation_time: f32,
    pub error: Option<String>,
}

/// Road generation job data
#[derive(Debug, Clone)]
pub struct RoadGenerationJob {
    pub road_id: String,
    pub road_entity: Entity,
    pub spline: Spline,
    pub road_params: RoadMeshParams,
    pub marking_params: MarkingParams,
    pub lane_count: u32,
    pub priority: f32,
    pub generation_type: RoadGenerationType,
}

/// Types of road generation
#[derive(Debug, Clone)]
pub enum RoadGenerationType {
    Standard,
    Intersection {
        intersection_type: IntersectionType,
        radius: f32,
    },
    Highway {
        lanes_per_direction: u32,
    },
    Bridge {
        supports: Vec<Vec3>,
    },
}

/// Async road mesh generation manager
#[derive(Resource)]
pub struct AsyncRoadMeshManager {
    /// Queue of pending mesh generation jobs
    pub generation_queue: VecDeque<RoadGenerationJob>,
    /// Currently running tasks
    pub active_tasks: HashMap<String, Entity>,
    /// Completed results waiting for main thread processing
    pub completed_results: Arc<Mutex<Vec<RoadMeshGenerationResult>>>,
    /// Maximum concurrent mesh generation tasks
    pub max_concurrent_tasks: usize,
    /// Generation diagnostics
    pub diagnostics: RoadMeshDiagnostics,
}

/// Performance diagnostics for road mesh generation
#[derive(Debug, Default)]
pub struct RoadMeshDiagnostics {
    pub total_roads_generated: u64,
    pub total_generation_time: f32,
    pub average_generation_time: f32,
    pub peak_concurrent_tasks: usize,
    pub failed_generations: u64,
    pub queue_size: usize,
    pub active_task_count: usize,
    pub vertices_generated: u64,
    pub triangles_generated: u64,
}

impl Default for AsyncRoadMeshManager {
    fn default() -> Self {
        Self {
            generation_queue: VecDeque::new(),
            active_tasks: HashMap::new(),
            completed_results: Arc::new(Mutex::new(Vec::new())),
            max_concurrent_tasks: 3, // Conservative for mesh generation
            diagnostics: RoadMeshDiagnostics::default(),
        }
    }
}

impl AsyncRoadMeshManager {
    /// Queue a road for async mesh generation
    pub fn queue_road_generation(&mut self, job: RoadGenerationJob) {
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
            self.start_road_mesh_generation(commands, job, time);
            self.diagnostics.queue_size = self.generation_queue.len();
            true
        } else {
            false
        }
    }

    /// Start async road mesh generation task
    fn start_road_mesh_generation(
        &mut self,
        commands: &mut Commands,
        job: RoadGenerationJob,
        time: &Res<Time>,
    ) {
        let task_pool = ComputeTaskPool::get();
        let results = Arc::clone(&self.completed_results);
        let current_time = time.elapsed_secs();
        let road_id = job.road_id.clone();

        let task = task_pool.spawn(async move {
            let start_time = std::time::Instant::now();

            // Generate road mesh on background thread
            let (main_mesh, marking_meshes) = match generate_road_mesh_async(&job).await {
                Ok((main, markings)) => (main, markings),
                Err(error) => {
                    let error_result = RoadMeshGenerationResult {
                        road_id: job.road_id,
                        road_entity: job.road_entity,
                        main_mesh: MeshData::empty(),
                        marking_meshes: Vec::new(),
                        generation_time: start_time.elapsed().as_secs_f32(),
                        error: Some(error),
                    };

                    if let Ok(mut results) = results.lock() {
                        results.push(error_result.clone());
                    }

                    return error_result;
                }
            };

            let generation_time = start_time.elapsed().as_secs_f32();

            let result = RoadMeshGenerationResult {
                road_id: job.road_id,
                road_entity: job.road_entity,
                main_mesh,
                marking_meshes,
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
            .spawn(RoadMeshGenerationTask {
                road_id: road_id.clone(),
                task,
                status: MeshGenerationStatus::InProgress { progress: 0.0 },
                started_at: current_time,
            })
            .id();

        self.active_tasks.insert(road_id, task_entity);
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
    ) -> Vec<RoadMeshGenerationResult> {
        let mut completed = Vec::new();

        if let Ok(mut results) = self.completed_results.lock() {
            completed = results.drain(..).collect();
        }

        // Update diagnostics
        for result in &completed {
            self.diagnostics.total_roads_generated += 1;
            self.diagnostics.total_generation_time += result.generation_time;
            self.diagnostics.average_generation_time = self.diagnostics.total_generation_time
                / self.diagnostics.total_roads_generated as f32;

            if result.error.is_some() {
                self.diagnostics.failed_generations += 1;
            } else {
                // Count mesh statistics
                self.diagnostics.vertices_generated += result.main_mesh.vertices.len() as u64;
                self.diagnostics.triangles_generated += (result.main_mesh.indices.len() / 3) as u64;

                for marking in &result.marking_meshes {
                    self.diagnostics.vertices_generated += marking.vertices.len() as u64;
                    self.diagnostics.triangles_generated += (marking.indices.len() / 3) as u64;
                }
            }

            // Remove from active tasks
            if let Some(task_entity) = self.active_tasks.remove(&result.road_id) {
                commands.entity(task_entity).despawn();
            }
        }

        self.diagnostics.active_task_count = self.active_tasks.len();
        completed
    }

    /// Check for timed out or failed tasks
    pub fn cleanup_stalled_tasks(&mut self, commands: &mut Commands, timeout_secs: f32) {
        // In a real implementation, we'd track task start times and timeout
        // For now, rely on the task completion mechanism
        self.diagnostics.active_task_count = self.active_tasks.len();
    }
}

impl MeshData {
    /// Create empty mesh data for error cases
    fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            primitive_topology: PrimitiveTopology::TriangleList,
        }
    }
}

/// Async road mesh generation (runs on background thread)
async fn generate_road_mesh_async(
    job: &RoadGenerationJob,
) -> Result<(MeshData, Vec<MeshData>), String> {
    // Yield to allow other tasks to run
    future::yield_now().await;

    // Generate main road surface mesh
    let main_mesh = generate_road_surface_mesh_data(&job.spline, &job.road_params).await?;

    // Yield again for responsiveness
    future::yield_now().await;

    // Generate lane marking meshes
    let marking_meshes = generate_road_markings_mesh_data(
        &job.spline,
        job.road_params.width,
        job.lane_count,
        &job.marking_params,
    )
    .await?;

    // Handle special generation types
    match &job.generation_type {
        RoadGenerationType::Standard => {}
        RoadGenerationType::Intersection {
            intersection_type,
            radius,
        } => {
            // Future: Generate intersection-specific geometry
        }
        RoadGenerationType::Highway {
            lanes_per_direction,
        } => {
            // Future: Generate highway-specific features (barriers, wider lanes, etc.)
        }
        RoadGenerationType::Bridge { supports } => {
            // Future: Generate bridge supports and structure
        }
    }

    Ok((main_mesh, marking_meshes))
}

/// Generate road surface mesh data asynchronously
async fn generate_road_surface_mesh_data(
    spline: &Spline,
    params: &RoadMeshParams,
) -> Result<MeshData, String> {
    let segments = calculate_optimal_segments_async(spline, params.segments).await;

    // Pre-allocate with exact capacity for performance
    let vertex_count = (segments + 1) * 2;
    let index_count = segments * 6;

    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);

    let half_width = params.width * 0.5;

    // Generate vertices along the spline
    for i in 0..=segments {
        // Yield periodically for responsiveness
        if i % 10 == 0 {
            future::yield_now().await;
        }

        let t = i as f32 / segments as f32;
        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);

        // Calculate right vector (perpendicular to tangent)
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        // Left and right edge vertices
        let left_pos = position + right * half_width;
        let right_pos = position - right * half_width;

        // Add vertices
        vertices.push([left_pos.x, left_pos.y, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y, right_pos.z]);

        // Calculate normals
        let normal = if params.smooth_normals {
            calculate_smooth_normal_async(spline, t, &right).await
        } else {
            [0.0, 1.0, 0.0]
        };

        normals.push(normal);
        normals.push(normal);

        // UV coordinates with proper scaling
        let v = t * params.uv_scale;
        uvs.push([0.0, v]);
        uvs.push([1.0, v]);

        // Generate triangles (except for the last segment)
        if i < segments {
            let base = (i * 2) as u32;

            // First triangle (counter-clockwise winding)
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            // Second triangle
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }
    }

    Ok(MeshData {
        vertices,
        normals,
        uvs,
        indices,
        primitive_topology: PrimitiveTopology::TriangleList,
    })
}

/// Generate road markings mesh data asynchronously
async fn generate_road_markings_mesh_data(
    spline: &Spline,
    road_width: f32,
    lane_count: u32,
    params: &MarkingParams,
) -> Result<Vec<MeshData>, String> {
    let mut markings = Vec::new();

    // Generate center line
    if lane_count > 1 {
        let center_line = generate_center_line_mesh_async(spline, params, true).await?;
        markings.push(center_line);
    }

    // Yield for responsiveness
    future::yield_now().await;

    // Generate lane divider lines for multi-lane roads
    if lane_count > 2 {
        let lane_width = road_width / lane_count as f32;

        for lane in 1..lane_count {
            let offset = (lane as f32 - lane_count as f32 * 0.5) * lane_width;
            let divider_line = generate_lane_divider_mesh_async(spline, offset, params).await?;
            markings.push(divider_line);

            // Yield periodically
            if lane % 2 == 0 {
                future::yield_now().await;
            }
        }
    }

    // Generate edge lines
    let left_edge = generate_edge_line_mesh_async(spline, road_width * 0.5, params).await?;
    let right_edge = generate_edge_line_mesh_async(spline, -road_width * 0.5, params).await?;

    markings.push(left_edge);
    markings.push(right_edge);

    Ok(markings)
}

/// Generate center line mesh asynchronously
async fn generate_center_line_mesh_async(
    spline: &Spline,
    params: &MarkingParams,
    dashed: bool,
) -> Result<MeshData, String> {
    let segments = calculate_optimal_segments_async(spline, 40).await;
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let dash_cycle = params.dash_length + params.gap_length;
    let total_length = spline.length();

    for i in 0..=segments {
        if i % 20 == 0 {
            future::yield_now().await;
        }

        let t = i as f32 / segments as f32;

        // Skip segments for dashed line
        if dashed {
            let distance = t * total_length;
            let cycle_position = distance % dash_cycle;
            if cycle_position > params.dash_length {
                continue;
            }
        }

        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        // Line vertices
        let left_pos = position + right * half_width;
        let right_pos = position - right * half_width;

        let base_idx = vertices.len() as u32;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]); // Slightly above road surface
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        // Generate triangles for line segments
        if vertices.len() >= 4 && base_idx >= 2 {
            indices.push(base_idx - 2);
            indices.push(base_idx);
            indices.push(base_idx - 1);

            indices.push(base_idx - 1);
            indices.push(base_idx);
            indices.push(base_idx + 1);
        }
    }

    Ok(MeshData {
        vertices,
        normals,
        uvs,
        indices,
        primitive_topology: PrimitiveTopology::TriangleList,
    })
}

/// Generate lane divider mesh asynchronously
async fn generate_lane_divider_mesh_async(
    spline: &Spline,
    offset: f32,
    params: &MarkingParams,
) -> Result<MeshData, String> {
    let segments = calculate_optimal_segments_async(spline, 40).await;
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Dashed line for lane dividers
    let dash_cycle = params.dash_length + params.gap_length;
    let total_length = spline.length();

    for i in 0..=segments {
        if i % 15 == 0 {
            future::yield_now().await;
        }

        let t = i as f32 / segments as f32;

        // Dashed pattern
        let distance = t * total_length;
        let cycle_position = distance % dash_cycle;
        if cycle_position > params.dash_length {
            continue;
        }

        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        let line_center = position + right * offset;
        let left_pos = line_center + right * half_width;
        let right_pos = line_center - right * half_width;

        let base_idx = vertices.len() as u32;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        if vertices.len() >= 4 && base_idx >= 2 {
            indices.push(base_idx - 2);
            indices.push(base_idx);
            indices.push(base_idx - 1);

            indices.push(base_idx - 1);
            indices.push(base_idx);
            indices.push(base_idx + 1);
        }
    }

    Ok(MeshData {
        vertices,
        normals,
        uvs,
        indices,
        primitive_topology: PrimitiveTopology::TriangleList,
    })
}

/// Generate edge line mesh asynchronously
async fn generate_edge_line_mesh_async(
    spline: &Spline,
    offset: f32,
    params: &MarkingParams,
) -> Result<MeshData, String> {
    let segments = calculate_optimal_segments_async(spline, 30).await;
    let half_width = params.line_width * 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        if i % 10 == 0 {
            future::yield_now().await;
        }

        let t = i as f32 / segments as f32;
        let position = spline.evaluate(t);
        let tangent = spline.evaluate_tangent(t);
        let right = Vec3::new(tangent.z, 0.0, -tangent.x).normalize_or_zero();

        let line_center = position + right * offset;
        let left_pos = line_center + right * half_width;
        let right_pos = line_center - right * half_width;

        vertices.push([left_pos.x, left_pos.y + 0.001, left_pos.z]);
        vertices.push([right_pos.x, right_pos.y + 0.001, right_pos.z]);

        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);

        uvs.push([0.0, t * params.uv_scale]);
        uvs.push([1.0, t * params.uv_scale]);

        if i > 0 {
            let base = ((i - 1) * 2) as u32;

            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 1);

            indices.push(base + 1);
            indices.push(base + 2);
            indices.push(base + 3);
        }
    }

    Ok(MeshData {
        vertices,
        normals,
        uvs,
        indices,
        primitive_topology: PrimitiveTopology::TriangleList,
    })
}

/// Calculate optimal segments asynchronously
async fn calculate_optimal_segments_async(spline: &Spline, base_segments: usize) -> usize {
    future::yield_now().await; // Yield for responsiveness

    let length = spline.length();
    let segments_by_length = (length / 10.0) as usize; // Segment every 10 units

    // Adjust based on curvature (simplified heuristic)
    let control_points = spline.control_points.len();
    let curvature_multiplier = if control_points > 2 { 1.5 } else { 1.0 };

    let optimal = ((segments_by_length as f32 * curvature_multiplier) as usize)
        .max(base_segments.min(4))
        .min(100); // Cap at 100 segments for performance

    optimal
}

/// Calculate smooth normal asynchronously
async fn calculate_smooth_normal_async(_spline: &Spline, _t: f32, _right: &Vec3) -> [f32; 3] {
    // For now, return upward normal
    // Future enhancement: consider road banking based on curvature
    [0.0, 1.0, 0.0]
}

/// System to enqueue road mesh generation jobs from RoadMeshRequest events
pub fn enqueue_road_mesh_jobs(
    mut mesh_manager: ResMut<AsyncRoadMeshManager>,
    mut mesh_requests: EventReader<super::events::RoadMeshRequest>,
) {
    for request in mesh_requests.read() {
        let job = RoadGenerationJob {
            road_id: request.road_id.to_string(),
            road_entity: request.road_entity,
            spline: request.spline.clone(),
            road_params: request.road_params.clone(),
            marking_params: request.marking_params.clone(),
            lane_count: request.lane_count,
            priority: request.priority,
            generation_type: request.generation_type.clone(),
        };

        mesh_manager.queue_road_generation(job);
        trace!(
            "Queued road mesh generation for road_id: {}",
            request.road_id
        );
    }
}

/// System to manage async road mesh generation
pub fn async_road_mesh_generation_system(
    mut commands: Commands,
    mut mesh_manager: ResMut<AsyncRoadMeshManager>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RoadMeshGenerationTask)>,
) {
    // Check for completed tasks
    for (entity, mut task) in query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.task)) {
            task.status = MeshGenerationStatus::Completed {
                mesh_data: result.main_mesh.clone(),
            };

            // Task will be cleaned up in process_completed_results
        }
    }

    // NOTE: Don't process completed results here - that's done in process_completed_road_meshes
    // to avoid double-draining the completed_results vector

    // Start new tasks if capacity allows
    while mesh_manager.try_start_next_task(&mut commands, &time) {
        // Keep starting tasks until we hit the limit or run out of work
    }

    // Clean up any stalled tasks
    mesh_manager.cleanup_stalled_tasks(&mut commands, 60.0);
}

/// System to process completed road mesh generation and emit RoadMeshReady events
pub fn process_completed_road_meshes(
    mut mesh_manager: ResMut<AsyncRoadMeshManager>,
    mut commands: Commands,
    mut mesh_ready_events: EventWriter<super::events::RoadMeshReady>,
) {
    let completed_results = mesh_manager.process_completed_results(&mut commands);

    for result in completed_results {
        // Parse road_id back to u32 (it was converted to String for storage)
        let road_id = match result.road_id.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                error!("Failed to parse road_id '{}' as u32", result.road_id);
                continue;
            }
        };

        if let Some(error) = result.error {
            // Emit failure event
            let failure_event = super::events::RoadMeshReady::failure(
                road_id,
                result.road_entity,
                error.clone(),
                result.generation_time,
            );
            mesh_ready_events.write(failure_event);
            warn!(
                "Road mesh generation failed for road_id {}: {}",
                road_id, error
            );
        } else {
            // Emit success event
            let success_event = super::events::RoadMeshReady::success(
                road_id,
                result.road_entity,
                result.main_mesh,
                result.marking_meshes,
                result.generation_time,
            );
            mesh_ready_events.write(success_event);
            debug!(
                "Road mesh generation completed for road_id {} in {:.2}ms",
                road_id,
                result.generation_time * 1000.0
            );
        }
    }
}

/// Debug system for async road mesh generation diagnostics
pub fn async_road_mesh_debug_system(mesh_manager: Res<AsyncRoadMeshManager>, time: Res<Time>) {
    // Only update debug info every 3 seconds to avoid spam
    if (time.elapsed_secs() % 3.0) < time.delta_secs() {
        let diag = &mesh_manager.diagnostics;

        info!(
            "🛣️ ASYNC ROAD MESH STATUS:\n\
            📊 Performance:\n\
            • Total Roads Generated: {} roads\n\
            • Average Gen Time: {:.2}ms\n\
            • Vertices Generated: {} vertices\n\
            • Triangles Generated: {} triangles\n\
            • Failed Generations: {}\n\
            📈 Current Status:\n\
            • Queue Size: {} roads\n\
            • Active Tasks: {}/{}\n\
            • Peak Concurrent: {}\n\
            💾 Resource Usage:\n\
            • Task Pool Utilization: {:.1}%",
            diag.total_roads_generated,
            diag.average_generation_time * 1000.0,
            diag.vertices_generated,
            diag.triangles_generated,
            diag.failed_generations,
            diag.queue_size,
            diag.active_task_count,
            mesh_manager.max_concurrent_tasks,
            diag.peak_concurrent_tasks,
            (diag.active_task_count as f32 / mesh_manager.max_concurrent_tasks as f32) * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_road_mesh_manager_creation() {
        let manager = AsyncRoadMeshManager::default();

        assert_eq!(manager.max_concurrent_tasks, 3);
        assert!(manager.generation_queue.is_empty());
        assert!(manager.active_tasks.is_empty());
    }

    #[test]
    fn test_mesh_data_creation() {
        let mesh_data = MeshData::empty();

        assert!(mesh_data.vertices.is_empty());
        assert!(mesh_data.indices.is_empty());
        assert_eq!(
            mesh_data.primitive_topology,
            PrimitiveTopology::TriangleList
        );
    }

    #[test]
    fn test_road_generation_job_creation() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0));

        let job = RoadGenerationJob {
            road_id: "test_road_001".to_string(),
            road_entity: Entity::from_raw(42),
            spline,
            road_params: RoadMeshParams::default(),
            marking_params: MarkingParams::default(),
            lane_count: 2,
            priority: 1.0,
            generation_type: RoadGenerationType::Standard,
        };

        assert_eq!(job.road_id, "test_road_001");
        assert_eq!(job.lane_count, 2);
    }
}
