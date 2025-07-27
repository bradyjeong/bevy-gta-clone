//! Hierarchical world streaming system with multi-scale LOD management
//!
//! This module implements the hierarchical world generation system ported from f430bc6-reference,
//! providing infinite streaming around player position with biome-based generation.

#[cfg(feature = "unstable_hierarchical_world")]
use amp_math::spatial::{HierarchicalQuadtree, LODLevel, WorldCoord};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Maximum entities per frame for smooth streaming
pub const MAX_CHUNKS_LOADED_PER_FRAME: usize = 2;
pub const MAX_CHUNKS_UNLOADED_PER_FRAME: usize = 4;
pub const MAX_CONTENT_GENERATED_PER_FRAME: usize = 5;

/// Chunk state with generation priorities
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkState {
    Unloaded,
    PendingLoad { priority: f32 },
    Loading { progress: f32 },
    Loaded { entities: Vec<Entity> },
    PendingUnload,
    Unloading,
}

/// Content layer flags for different types
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

/// Chunk data with hierarchical management
#[derive(Debug, Clone, Component)]
pub struct WorldChunk {
    pub coord: WorldCoord,
    pub state: ChunkState,
    pub distance_to_active: f32,
    pub last_accessed: f32,
    pub parent: Option<WorldCoord>,
    pub children: Vec<WorldCoord>,
    pub content_layers: ContentLayers,
    pub generation_seed: u64,
}

impl WorldChunk {
    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn new(coord: WorldCoord, current_time: f32) -> Self {
        // Generate deterministic seed from coordinates
        let generation_seed = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            (coord.level as u8, coord.x, coord.z).hash(&mut hasher);
            hasher.finish()
        };

        Self {
            coord,
            state: ChunkState::Unloaded,
            distance_to_active: f32::INFINITY,
            last_accessed: current_time,
            parent: coord.get_parent(),
            children: coord.get_children(),
            content_layers: ContentLayers::default(),
            generation_seed,
        }
    }

    pub fn get_generation_priority(&self) -> f32 {
        // Higher priority = loaded first
        let distance_factor = 1.0 / (self.distance_to_active + 1.0);
        let lod_factor = match self.coord.level {
            LODLevel::Micro => 5.0,
            LODLevel::Detail => 4.0,
            LODLevel::Local => 3.0,
            LODLevel::Region => 2.0,
            LODLevel::Macro => 1.0,
        };

        distance_factor * lod_factor
    }
}

/// Advanced world streaming manager with hierarchical LOD
#[derive(Resource)]
pub struct WorldLODManager {
    pub chunks: HashMap<WorldCoord, WorldChunk>,
    pub active_position: Vec3,
    pub active_coords: BTreeMap<LODLevel, WorldCoord>,

    // Streaming queues with priority
    pub load_queue: VecDeque<WorldCoord>,
    pub unload_queue: VecDeque<WorldCoord>,
    pub generation_queue: VecDeque<(WorldCoord, f32)>, // (coord, priority)

    // Performance tracking
    pub chunks_loaded_this_frame: usize,
    pub chunks_unloaded_this_frame: usize,
    pub content_generated_this_frame: usize,
    pub last_update_time: f32,

    // Memory management
    pub max_loaded_chunks: HashMap<LODLevel, usize>,
    pub loaded_chunk_count: HashMap<LODLevel, usize>,

    // Spatial indexing
    #[cfg(feature = "unstable_hierarchical_world")]
    pub spatial_index: HierarchicalQuadtree,
}

/// Memory usage tracking for diagnostics
#[derive(Debug, Default)]
pub struct WorldMemoryUsage {
    pub macro_chunks: usize,
    pub macro_max: usize,
    pub region_chunks: usize,
    pub region_max: usize,
    pub local_chunks: usize,
    pub local_max: usize,
    pub detail_chunks: usize,
    pub detail_max: usize,
    pub micro_chunks: usize,
    pub micro_max: usize,
    pub total_chunks: usize,
    pub load_queue_size: usize,
    pub unload_queue_size: usize,
    pub generation_queue_size: usize,
}

impl Default for WorldLODManager {
    fn default() -> Self {
        let mut max_loaded_chunks = HashMap::new();
        max_loaded_chunks.insert(LODLevel::Macro, 100); // 100 macro regions (100km²)
        max_loaded_chunks.insert(LODLevel::Region, 500); // 500 regions (2000km²)
        max_loaded_chunks.insert(LODLevel::Local, 1000); // 1000 local chunks (160km²)
        max_loaded_chunks.insert(LODLevel::Detail, 2000); // 2000 detail chunks (20km²)
        max_loaded_chunks.insert(LODLevel::Micro, 3000); // 3000 micro chunks (1.9km²)

        let mut loaded_chunk_count = HashMap::new();
        for level in [
            LODLevel::Macro,
            LODLevel::Region,
            LODLevel::Local,
            LODLevel::Detail,
            LODLevel::Micro,
        ] {
            loaded_chunk_count.insert(level, 0);
        }

        Self {
            chunks: HashMap::new(),
            active_position: Vec3::ZERO,
            active_coords: BTreeMap::new(),
            load_queue: VecDeque::new(),
            unload_queue: VecDeque::new(),
            generation_queue: VecDeque::new(),
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            content_generated_this_frame: 0,
            last_update_time: 0.0,
            max_loaded_chunks,
            loaded_chunk_count,
            #[cfg(feature = "unstable_hierarchical_world")]
            spatial_index: HierarchicalQuadtree::new(),
        }
    }
}

impl WorldLODManager {
    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn update_active_position(&mut self, position: Vec3, current_time: f32) {
        self.active_position = position;
        self.last_update_time = current_time;

        // Update active coordinates for each LOD level
        for level in [
            LODLevel::Macro,
            LODLevel::Region,
            LODLevel::Local,
            LODLevel::Detail,
            LODLevel::Micro,
        ] {
            let coord = WorldCoord::from_world_pos(position, level);
            self.active_coords.insert(level, coord);
        }

        // Update distances for all chunks
        for chunk in self.chunks.values_mut() {
            chunk.distance_to_active = position.distance(chunk.coord.to_world_pos());
            chunk.last_accessed = current_time;
        }
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn should_load_chunk(&self, coord: WorldCoord) -> bool {
        if self.chunks.contains_key(&coord) {
            return false; // Already exists
        }

        let distance = self.active_position.distance(coord.to_world_pos());
        distance <= coord.get_streaming_radius()
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn should_unload_chunk(&self, coord: WorldCoord, current_time: f32) -> bool {
        if let Some(chunk) = self.chunks.get(&coord) {
            let distance = self.active_position.distance(coord.to_world_pos());
            let streaming_radius = coord.get_streaming_radius();

            // Unload if outside streaming radius with hysteresis
            if distance > streaming_radius * 1.2 {
                return true;
            }

            // Unload if not accessed recently and memory pressure
            let loaded_count = self.loaded_chunk_count.get(&coord.level).unwrap_or(&0);
            let max_count = self.max_loaded_chunks.get(&coord.level).unwrap_or(&1000);

            if loaded_count > max_count && (current_time - chunk.last_accessed) > 30.0 {
                return true;
            }
        }

        false
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn get_chunks_to_load(&mut self) -> Vec<WorldCoord> {
        let mut candidates = Vec::new();

        // Generate candidates for each LOD level around active position
        for level in [
            LODLevel::Macro,
            LODLevel::Region,
            LODLevel::Local,
            LODLevel::Detail,
            LODLevel::Micro,
        ] {
            if let Some(active_coord) = self.active_coords.get(&level) {
                let streaming_radius = level as i32 + 2; // Radius in chunks

                for dx in -streaming_radius..=streaming_radius {
                    for dz in -streaming_radius..=streaming_radius {
                        let coord =
                            WorldCoord::new(level, active_coord.x + dx, active_coord.z + dz);

                        if self.should_load_chunk(coord) {
                            candidates.push(coord);
                        }
                    }
                }
            }
        }

        // Sort by priority (distance and LOD level)
        candidates.sort_by(|a, b| {
            let dist_a = self.active_position.distance(a.to_world_pos());
            let dist_b = self.active_position.distance(b.to_world_pos());
            let priority_a = 1.0 / (dist_a + 1.0) * (a.level as u8 + 1) as f32;
            let priority_b = 1.0 / (dist_b + 1.0) * (b.level as u8 + 1) as f32;
            priority_b
                .partial_cmp(&priority_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .into_iter()
            .take(MAX_CHUNKS_LOADED_PER_FRAME)
            .collect()
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn get_chunks_to_unload(&mut self, current_time: f32) -> Vec<WorldCoord> {
        let mut candidates: Vec<_> = self
            .chunks
            .keys()
            .filter(|coord| self.should_unload_chunk(**coord, current_time))
            .cloned()
            .collect();

        // Sort by priority (distance and last access time)
        candidates.sort_by(|a, b| {
            let chunk_a = &self.chunks[a];
            let chunk_b = &self.chunks[b];

            let priority_a = chunk_a.distance_to_active + (current_time - chunk_a.last_accessed);
            let priority_b = chunk_b.distance_to_active + (current_time - chunk_b.last_accessed);

            priority_b
                .partial_cmp(&priority_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .into_iter()
            .take(MAX_CHUNKS_UNLOADED_PER_FRAME)
            .collect()
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn mark_chunk_for_loading(&mut self, coord: WorldCoord, current_time: f32) {
        let chunk = WorldChunk::new(coord, current_time);
        let priority = chunk.get_generation_priority();

        self.chunks.insert(coord, chunk);
        self.generation_queue.push_back((coord, priority));

        // Sort generation queue by priority
        self.generation_queue
            .make_contiguous()
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn mark_chunk_for_unloading(&mut self, coord: WorldCoord) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.state = ChunkState::PendingUnload;
            self.unload_queue.push_back(coord);
        }
    }

    pub fn get_next_chunk_to_generate(&mut self) -> Option<WorldCoord> {
        while let Some((coord, _priority)) = self.generation_queue.pop_front() {
            if let Some(chunk) = self.chunks.get(&coord) {
                if matches!(chunk.state, ChunkState::Unloaded) {
                    return Some(coord);
                }
            }
        }
        None
    }

    pub fn finalize_chunk_loading(&mut self, coord: WorldCoord, entities: Vec<Entity>) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.state = ChunkState::Loaded {
                entities: entities.clone(),
            };

            // Update loaded count
            let count = self.loaded_chunk_count.entry(coord.level).or_insert(0);
            *count += 1;

            // Add entities to spatial index
            #[cfg(feature = "unstable_hierarchical_world")]
            {
                let position = coord.to_world_pos();
                for (i, entity) in entities.iter().enumerate() {
                    self.spatial_index
                        .insert_entity(entity.index(), position, coord.level);
                }
            }
        }
    }

    pub fn unload_chunk(&mut self, coord: WorldCoord) -> Option<Vec<Entity>> {
        if let Some(chunk) = self.chunks.remove(&coord) {
            // Update loaded count
            let count = self.loaded_chunk_count.entry(coord.level).or_insert(0);
            *count = count.saturating_sub(1);

            // Remove from spatial index
            #[cfg(feature = "unstable_hierarchical_world")]
            {
                self.spatial_index.remove_node(coord);
            }

            match chunk.state {
                ChunkState::Loaded { entities } => Some(entities),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn reset_frame_counters(&mut self) {
        self.chunks_loaded_this_frame = 0;
        self.chunks_unloaded_this_frame = 0;
        self.content_generated_this_frame = 0;
    }

    pub fn get_memory_usage(&self) -> WorldMemoryUsage {
        let mut usage = WorldMemoryUsage::default();

        for (level, count) in &self.loaded_chunk_count {
            let max_count = self.max_loaded_chunks.get(level).unwrap_or(&1000);
            match level {
                LODLevel::Macro => {
                    usage.macro_chunks = *count;
                    usage.macro_max = *max_count;
                }
                LODLevel::Region => {
                    usage.region_chunks = *count;
                    usage.region_max = *max_count;
                }
                LODLevel::Local => {
                    usage.local_chunks = *count;
                    usage.local_max = *max_count;
                }
                LODLevel::Detail => {
                    usage.detail_chunks = *count;
                    usage.detail_max = *max_count;
                }
                LODLevel::Micro => {
                    usage.micro_chunks = *count;
                    usage.micro_max = *max_count;
                }
            }
        }

        usage.total_chunks = self.chunks.len();
        usage.load_queue_size = self.load_queue.len();
        usage.unload_queue_size = self.unload_queue.len();
        usage.generation_queue_size = self.generation_queue.len();

        usage
    }

    /// Query entities within a radius for LOD-aware rendering
    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn query_entities_in_radius(
        &self,
        center: Vec3,
        radius: f32,
        lod_level: LODLevel,
    ) -> Vec<u32> {
        self.spatial_index.query_entities(center, radius, lod_level)
    }

    /// Get streaming coordinates for the current active position
    #[cfg(feature = "unstable_hierarchical_world")]
    pub fn get_streaming_coords(&self) -> Vec<WorldCoord> {
        self.spatial_index
            .get_streaming_coords(self.active_position)
    }
}

/// Marker component for the active entity (usually the player)
#[derive(Component)]
pub struct ActiveEntity;

/// Main infinite world streaming system with async generation
#[cfg(feature = "unstable_hierarchical_world")]
pub fn hierarchical_world_streaming_system(
    mut commands: Commands,
    mut world_manager: ResMut<WorldLODManager>,
    mut async_generation: ResMut<super::async_generation::AsyncGenerationManager>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
) {
    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let current_time = time.elapsed_secs();

    // Reset frame counters
    world_manager.reset_frame_counters();

    // Update active position and chunk coordinates
    world_manager.update_active_position(active_transform.translation, current_time);

    // Unload distant chunks first to free memory
    let chunks_to_unload = world_manager.get_chunks_to_unload(current_time);
    for coord in chunks_to_unload {
        if world_manager.chunks_unloaded_this_frame >= MAX_CHUNKS_UNLOADED_PER_FRAME {
            break;
        }

        if let Some(entities) = world_manager.unload_chunk(coord) {
            // Despawn all entities in the chunk
            for entity in entities {
                commands.entity(entity).despawn();
            }
            world_manager.chunks_unloaded_this_frame += 1;
        }
    }

    // Load new chunks within streaming radius
    let chunks_to_load = world_manager.get_chunks_to_load();
    for coord in chunks_to_load {
        if world_manager.chunks_loaded_this_frame >= MAX_CHUNKS_LOADED_PER_FRAME {
            break;
        }

        world_manager.mark_chunk_for_loading(coord, current_time);
        world_manager.chunks_loaded_this_frame += 1;
    }

    // Queue chunks for async generation instead of blocking main thread
    while world_manager.content_generated_this_frame < MAX_CONTENT_GENERATED_PER_FRAME {
        if let Some(coord) = world_manager.get_next_chunk_to_generate() {
            if let Some(chunk) = world_manager.chunks.get(&coord) {
                let job = super::async_generation::GenerationJob {
                    coord,
                    generation_seed: chunk.generation_seed,
                    priority: chunk.get_generation_priority(),
                    content_layers: chunk.content_layers.clone(),
                };
                async_generation.queue_chunk_generation(job);
                world_manager.content_generated_this_frame += 1;
            }
        } else {
            break;
        }
    }
}

/// Initiate chunk content generation based on LOD level
#[cfg(feature = "unstable_hierarchical_world")]
fn initiate_chunk_generation(
    commands: &mut Commands,
    world_manager: &mut WorldLODManager,
    coord: WorldCoord,
) {
    let chunk_center = coord.to_world_pos();
    let mut entities = Vec::new();

    // Generate content based on LOD level
    match coord.level {
        LODLevel::Macro => {
            // Macro level: Procedural heightmap and biome data only
            // No actual entities, just mark as loaded
        }
        LODLevel::Region => {
            // Region level: Basic terrain features and major roads
            let road_entity = commands
                .spawn((
                    Name::new(format!(
                        "Region_Road_{}_{}_{}",
                        coord.level as u8, coord.x, coord.z
                    )),
                    Transform::from_translation(chunk_center),
                    GlobalTransform::default(),
                    Visibility::default(),
                ))
                .id();
            entities.push(road_entity);
        }
        LODLevel::Local => {
            // Local level: Buildings, major vegetation, secondary roads
            for i in 0..3 {
                let building_pos = chunk_center + Vec3::new((i as f32 - 1.0) * 100.0, 0.0, 0.0);

                let building_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Local_Building_{}_{}_{}_{}",
                            coord.level as u8, coord.x, coord.z, i
                        )),
                        Transform::from_translation(building_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(building_entity);
            }
        }
        LODLevel::Detail => {
            // Detail level: Detailed objects, vehicles, some NPCs
            for i in 0..2 {
                let vehicle_pos = chunk_center + Vec3::new((i as f32 - 0.5) * 50.0, 0.0, 25.0);

                let vehicle_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Detail_Vehicle_{}_{}_{}_{}",
                            coord.level as u8, coord.x, coord.z, i
                        )),
                        Transform::from_translation(vehicle_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(vehicle_entity);
            }
        }
        LODLevel::Micro => {
            // Micro level: Full detail with physics, NPCs, interactive objects
            for i in 0..4 {
                let detail_pos = chunk_center
                    + Vec3::new(
                        (i % 2) as f32 * 15.0 - 7.5,
                        0.0,
                        (i / 2) as f32 * 15.0 - 7.5,
                    );

                let detail_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Micro_Detail_{}_{}_{}_{}",
                            coord.level as u8, coord.x, coord.z, i
                        )),
                        Transform::from_translation(detail_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(detail_entity);
            }
        }
    }

    // Mark chunk as loaded
    world_manager.finalize_chunk_loading(coord, entities);
}

/// Process async chunk generation results and integrate them
#[cfg(feature = "unstable_hierarchical_world")]
pub fn process_async_chunk_results(
    mut commands: Commands,
    mut world_manager: ResMut<WorldLODManager>,
    mut async_generation: ResMut<super::async_generation::AsyncGenerationManager>,
) {
    let completed_results = async_generation.process_completed_results(&mut commands);

    for result in completed_results {
        if result.error.is_some() {
            warn!(
                "Chunk generation failed for {:?}: {:?}",
                result.coord, result.error
            );
            continue;
        }

        // Create actual entities from async generation results
        let entities = create_entities_from_async_result(&mut commands, &result);

        // Mark chunk as loaded in world manager
        world_manager.finalize_chunk_loading(result.coord, entities);

        info!(
            "✅ Async chunk generation completed for {:?} in {:.2}ms",
            result.coord,
            result.generation_time * 1000.0
        );
    }
}

/// Create entities from async generation result
#[cfg(feature = "unstable_hierarchical_world")]
fn create_entities_from_async_result(
    commands: &mut Commands,
    result: &super::async_generation::ChunkGenerationResult,
) -> Vec<Entity> {
    let chunk_center = result.coord.to_world_pos();
    let mut entities = Vec::new();

    // Generate content based on LOD level (similar to original sync version)
    match result.coord.level {
        amp_math::spatial::LODLevel::Macro => {
            // Macro level: Procedural heightmap and biome data only
            // No actual entities, just mark as loaded
        }
        amp_math::spatial::LODLevel::Region => {
            // Region level: Basic terrain features and major roads
            let road_entity = commands
                .spawn((
                    Name::new(format!(
                        "Region_Road_{}_{}_{}",
                        result.coord.level as u8, result.coord.x, result.coord.z
                    )),
                    Transform::from_translation(chunk_center),
                    GlobalTransform::default(),
                    Visibility::default(),
                ))
                .id();
            entities.push(road_entity);
        }
        amp_math::spatial::LODLevel::Local => {
            // Local level: Buildings, major vegetation, secondary roads
            for i in 0..3 {
                let building_pos = chunk_center + Vec3::new((i as f32 - 1.0) * 100.0, 0.0, 0.0);

                let building_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Local_Building_{}_{}_{}_{}",
                            result.coord.level as u8, result.coord.x, result.coord.z, i
                        )),
                        Transform::from_translation(building_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(building_entity);
            }
        }
        amp_math::spatial::LODLevel::Detail => {
            // Detail level: Detailed objects, vehicles, some NPCs
            for i in 0..2 {
                let vehicle_pos = chunk_center + Vec3::new((i as f32 - 0.5) * 50.0, 0.0, 25.0);

                let vehicle_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Detail_Vehicle_{}_{}_{}_{}",
                            result.coord.level as u8, result.coord.x, result.coord.z, i
                        )),
                        Transform::from_translation(vehicle_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(vehicle_entity);
            }
        }
        amp_math::spatial::LODLevel::Micro => {
            // Micro level: Full detail with physics, NPCs, interactive objects
            for i in 0..4 {
                let detail_pos = chunk_center
                    + Vec3::new(
                        (i % 2) as f32 * 15.0 - 7.5,
                        0.0,
                        (i / 2) as f32 * 15.0 - 7.5,
                    );

                let detail_entity = commands
                    .spawn((
                        Name::new(format!(
                            "Micro_Detail_{}_{}_{}_{}",
                            result.coord.level as u8, result.coord.x, result.coord.z, i
                        )),
                        Transform::from_translation(detail_pos),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();
                entities.push(detail_entity);
            }
        }
    }

    entities
}

/// Debug system to display world streaming status
#[cfg(feature = "unstable_hierarchical_world")]
pub fn hierarchical_world_debug_system(world_manager: Res<WorldLODManager>, time: Res<Time>) {
    // Only update debug info every second to avoid spam
    if (time.elapsed_secs() % 1.0) < time.delta_secs() {
        let usage = world_manager.get_memory_usage();

        info!(
            "🌍 HIERARCHICAL WORLD STATUS:\n\
            📊 Memory Usage:\n\
            • Macro:  {}/{} regions\n\
            • Region: {}/{} chunks\n\
            • Local:  {}/{} chunks\n\
            • Detail: {}/{} chunks\n\
            • Micro:  {}/{} chunks\n\
            📈 Queue Status:\n\
            • Load Queue: {} chunks\n\
            • Unload Queue: {} chunks\n\
            • Generation Queue: {} chunks\n\
            🎯 Active Position: {:.1}, {:.1}, {:.1}",
            usage.macro_chunks,
            usage.macro_max,
            usage.region_chunks,
            usage.region_max,
            usage.local_chunks,
            usage.local_max,
            usage.detail_chunks,
            usage.detail_max,
            usage.micro_chunks,
            usage.micro_max,
            usage.load_queue_size,
            usage.unload_queue_size,
            usage.generation_queue_size,
            world_manager.active_position.x,
            world_manager.active_position.y,
            world_manager.active_position.z
        );
    }
}

/// Plugin for hierarchical world streaming
#[derive(Default)]
pub struct HierarchicalWorldPlugin;

impl Plugin for HierarchicalWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldLODManager>()
            .init_resource::<super::async_generation::AsyncGenerationManager>();

        #[cfg(feature = "unstable_hierarchical_world")]
        app.add_systems(
            Update,
            (
                hierarchical_world_streaming_system,
                hierarchical_world_debug_system,
                super::async_generation::async_chunk_generation_system,
                super::async_generation::async_generation_debug_system,
                process_async_chunk_results,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_lod_manager_creation() {
        let manager = WorldLODManager::default();

        assert_eq!(manager.active_position, Vec3::ZERO);
        assert!(manager.chunks.is_empty());
        assert!(manager.load_queue.is_empty());
        assert!(manager.unload_queue.is_empty());
        assert!(manager.generation_queue.is_empty());
    }

    #[cfg(feature = "unstable_hierarchical_world")]
    #[test]
    fn test_chunk_state_transitions() {
        let coord = WorldCoord::new(LODLevel::Local, 0, 0);
        let chunk = WorldChunk::new(coord, 0.0);

        assert!(matches!(chunk.state, ChunkState::Unloaded));
        assert_eq!(chunk.coord, coord);

        match chunk.state {
            ChunkState::Loaded { ref entities } => assert!(entities.is_empty()),
            _ => {} // Other states don't have entities field
        }
    }

    #[test]
    fn test_memory_usage_tracking() {
        let manager = WorldLODManager::default();
        let usage = manager.get_memory_usage();

        assert_eq!(usage.total_chunks, 0);
        assert_eq!(usage.macro_chunks, 0);
        assert_eq!(usage.region_chunks, 0);
        assert_eq!(usage.local_chunks, 0);
        assert_eq!(usage.detail_chunks, 0);
        assert_eq!(usage.micro_chunks, 0);
    }
}
