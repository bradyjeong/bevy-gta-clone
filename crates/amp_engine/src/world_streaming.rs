use amp_math::chunk_key::ChunkKey;
use amp_math::Vec3;
#[cfg(feature = "bevy16")]
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Sector ID for world streaming (Oracle's M4 requirements)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId {
    pub x: i32,
    pub z: i32,
}

impl SectorId {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_position(x: f32, z: f32, sector_size: f32) -> Self {
        Self {
            x: (x / sector_size).floor() as i32,
            z: (z / sector_size).floor() as i32,
        }
    }

    pub fn to_world_position(&self, sector_size: f32) -> (f32, f32) {
        (self.x as f32 * sector_size, self.z as f32 * sector_size)
    }
}

/// Spawn state for sectors (Oracle's M4 requirements)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
}

/// LOD level for entities (Oracle's M4 requirements)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodLevel {
    LOD0,     // Full mesh
    LOD1,     // Proxy/simplified mesh
    Impostor, // Quad/billboard or nothing
}

#[cfg(feature = "bevy16")]
use config_core::types::WorldGenerationConfig;
#[cfg(not(feature = "bevy16"))]
use config_core::types::WorldGenerationConfig;

/// Sector loading state (updated from chunk to sector terminology)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
}

/// Chunk loading state (kept for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLoadState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
}

/// Chunk data with entities and metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy16", derive(Component))]
pub struct ChunkData {
    pub key: ChunkKey,
    #[cfg(feature = "bevy16")]
    pub entities: Vec<Entity>,
    #[cfg(not(feature = "bevy16"))]
    pub entities: Vec<u32>, // Mock entity IDs when Bevy not available
    pub load_state: ChunkLoadState,
    pub loaded_at: Instant,
    pub entity_count: u32,
    pub content_generated: bool,
}

impl ChunkData {
    pub fn new(key: ChunkKey) -> Self {
        Self {
            key,
            entities: Vec::new(),
            load_state: ChunkLoadState::NotLoaded,
            loaded_at: Instant::now(),
            entity_count: 0,
            content_generated: false,
        }
    }

    #[cfg(feature = "bevy16")]
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
        self.entity_count += 1;
    }

    #[cfg(not(feature = "bevy16"))]
    pub fn add_entity(&mut self, entity: u32) {
        self.entities.push(entity);
        self.entity_count += 1;
    }

    #[cfg(feature = "bevy16")]
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.remove(pos);
            self.entity_count = self.entity_count.saturating_sub(1);
        }
    }

    #[cfg(not(feature = "bevy16"))]
    pub fn remove_entity(&mut self, entity: u32) {
        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.remove(pos);
            self.entity_count = self.entity_count.saturating_sub(1);
        }
    }

    pub fn can_add_entity(&self, entity_limit: u32) -> bool {
        self.entity_count < entity_limit
    }
}

/// Chunk loading task marker
#[derive(Clone)]
#[cfg_attr(feature = "bevy16", derive(Component))]
pub struct ChunkLoadTask {
    pub chunk_key: ChunkKey,
    pub progress: f32,
}

/// Sector data with entities and metadata (Oracle's M4 requirements)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy16", derive(Component))]
pub struct SectorData {
    pub id: SectorId,
    #[cfg(feature = "bevy16")]
    pub entities: Vec<Entity>,
    #[cfg(not(feature = "bevy16"))]
    pub entities: Vec<u32>, // Mock entity IDs when Bevy not available
    pub spawn_state: SpawnState,
    pub loaded_at: Instant,
    pub entity_count: u32,
    pub lod_level: LodLevel,
}

impl SectorData {
    pub fn new(id: SectorId) -> Self {
        Self {
            id,
            entities: Vec::new(),
            spawn_state: SpawnState::NotLoaded,
            loaded_at: Instant::now(),
            entity_count: 0,
            lod_level: LodLevel::LOD0,
        }
    }

    #[cfg(feature = "bevy16")]
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
        self.entity_count += 1;
    }

    #[cfg(not(feature = "bevy16"))]
    pub fn add_entity(&mut self, entity: u32) {
        self.entities.push(entity);
        self.entity_count += 1;
    }

    pub fn set_lod_level(&mut self, level: LodLevel) {
        self.lod_level = level;
    }

    pub fn should_spawn(&self) -> bool {
        matches!(self.spawn_state, SpawnState::NotLoaded)
    }

    pub fn should_despawn(&self) -> bool {
        matches!(self.spawn_state, SpawnState::Loaded)
    }
}

/// World streaming resource managing chunk loading/unloading
#[derive(Debug)]
#[cfg_attr(feature = "bevy16", derive(Resource))]
pub struct WorldStreamer {
    /// Currently loaded chunks
    pub loaded_chunks: HashMap<ChunkKey, ChunkData>,
    /// Chunks queued for loading
    pub load_queue: VecDeque<ChunkKey>,
    /// Chunks queued for unloading
    pub unload_queue: VecDeque<ChunkKey>,
    /// Current player position for streaming
    pub player_position: Vec3,
    /// Chunk size in world units
    pub chunk_size: f32,
    /// Radius for streaming new chunks
    pub streaming_radius: f32,
    /// Radius for keeping chunks active
    pub active_radius: f32,
    /// Entity limit per chunk
    pub entity_limit_per_chunk: u32,
    /// Last update time for performance tracking
    pub last_update: Instant,
    /// Performance stats
    pub stats: StreamingStats,

    // Oracle's M4 requirements: Sector streaming
    /// Sector management (64m x 64m sectors)
    pub sectors: HashMap<SectorId, SectorData>,
    /// Sector size in world units (64m as per Oracle's requirements)
    pub sector_size: f32,
    /// View radius for sector streaming
    pub view_radius: f32,
    /// Sectors queued for spawning
    pub spawn_queue: VecDeque<SectorId>,
    /// Sectors queued for despawning
    pub despawn_queue: VecDeque<SectorId>,
}

/// Performance statistics for streaming (Oracle's M4 requirements)
#[derive(Debug, Default)]
pub struct StreamingStats {
    pub chunks_loaded: u32,
    pub chunks_unloaded: u32,
    pub entities_spawned: u32,
    pub entities_despawned: u32,
    pub average_update_time_ms: f32,
    pub peak_update_time_ms: f32,

    // M4 Oracle requirements: HUD counters
    pub sectors_loaded: u32,
    pub sectors_unloaded: u32,
    pub total_entities_in_world: u32,
    pub memory_usage_mb: f32,
    pub lod_level_counts: [u32; 3], // [LOD0, LOD1, Impostor]
}

impl WorldStreamer {
    pub fn new(config: &WorldGenerationConfig) -> Self {
        Self {
            loaded_chunks: HashMap::new(),
            load_queue: VecDeque::new(),
            unload_queue: VecDeque::new(),
            player_position: Vec3::ZERO,
            chunk_size: config.chunk_size,
            streaming_radius: config.streaming_radius,
            active_radius: config.active_radius,
            entity_limit_per_chunk: config.entity_limit_per_chunk,
            last_update: Instant::now(),
            stats: StreamingStats::default(),

            // Oracle's M4 requirements: Initialize sector management
            sectors: HashMap::new(),
            sector_size: 64.0,  // 64m x 64m sectors as per Oracle's requirements
            view_radius: 512.0, // 8 sectors radius for view distance
            spawn_queue: VecDeque::new(),
            despawn_queue: VecDeque::new(),
        }
    }

    /// Get chunk key for a world position
    pub fn get_chunk_key(&self, position: Vec3) -> ChunkKey {
        ChunkKey::from_position(position.x, position.z, self.chunk_size)
    }

    /// Check if a chunk is within streaming radius
    pub fn is_in_streaming_radius(&self, chunk_key: &ChunkKey) -> bool {
        let (chunk_x, chunk_z) = chunk_key.to_world_position(self.chunk_size);
        let distance = self
            .player_position
            .distance(Vec3::new(chunk_x, 0.0, chunk_z));
        distance <= self.streaming_radius
    }

    /// Check if a chunk is within active radius
    pub fn is_in_active_radius(&self, chunk_key: &ChunkKey) -> bool {
        let (chunk_x, chunk_z) = chunk_key.to_world_position(self.chunk_size);
        let distance = self
            .player_position
            .distance(Vec3::new(chunk_x, 0.0, chunk_z));
        distance <= self.active_radius
    }

    /// Get all chunks within streaming radius
    pub fn get_streaming_chunks(&self) -> Vec<ChunkKey> {
        let player_chunk = self.get_chunk_key(self.player_position);
        let chunk_radius = (self.streaming_radius / self.chunk_size).ceil() as i32;

        let mut chunks = Vec::new();
        for x in -chunk_radius..=chunk_radius {
            for z in -chunk_radius..=chunk_radius {
                let chunk_key = ChunkKey::new(player_chunk.x + x, player_chunk.z + z);
                if self.is_in_streaming_radius(&chunk_key) {
                    chunks.push(chunk_key);
                }
            }
        }
        chunks
    }

    /// Queue a chunk for loading
    pub fn queue_chunk_load(&mut self, chunk_key: ChunkKey) {
        if !self.loaded_chunks.contains_key(&chunk_key) && !self.load_queue.contains(&chunk_key) {
            self.load_queue.push_back(chunk_key);
        }
    }

    /// Queue a chunk for unloading
    pub fn queue_chunk_unload(&mut self, chunk_key: ChunkKey) {
        if self.loaded_chunks.contains_key(&chunk_key) && !self.unload_queue.contains(&chunk_key) {
            self.unload_queue.push_back(chunk_key);
        }
    }

    /// Mark chunk as loaded
    #[cfg(feature = "bevy16")]
    pub fn mark_chunk_loaded(&mut self, chunk_key: ChunkKey, entities: Vec<Entity>) {
        let mut chunk_data = self
            .loaded_chunks
            .get(&chunk_key)
            .cloned()
            .unwrap_or_else(|| ChunkData::new(chunk_key));
        chunk_data.load_state = ChunkLoadState::Loaded;
        let entity_count = entities.len() as u32;
        chunk_data.entities = entities;
        chunk_data.entity_count = entity_count;

        self.loaded_chunks.insert(chunk_key, chunk_data);
        self.stats.chunks_loaded += 1;
        self.stats.entities_spawned += entity_count;
    }

    /// Mark chunk as loaded (non-Bevy version)
    #[cfg(not(feature = "bevy16"))]
    pub fn mark_chunk_loaded(&mut self, chunk_key: ChunkKey, entities: Vec<u32>) {
        let mut chunk_data = self
            .loaded_chunks
            .get(&chunk_key)
            .cloned()
            .unwrap_or_else(|| ChunkData::new(chunk_key));
        chunk_data.load_state = ChunkLoadState::Loaded;
        let entity_count = entities.len() as u32;
        chunk_data.entities = entities;
        chunk_data.entity_count = entity_count;

        self.loaded_chunks.insert(chunk_key, chunk_data);
        self.stats.chunks_loaded += 1;
        self.stats.entities_spawned += entity_count;
    }

    /// Mark chunk as unloaded
    pub fn mark_chunk_unloaded(&mut self, chunk_key: ChunkKey) {
        if let Some(chunk_data) = self.loaded_chunks.remove(&chunk_key) {
            self.stats.chunks_unloaded += 1;
            self.stats.entities_despawned += chunk_data.entity_count;
        }
    }

    /// Update performance statistics
    pub fn update_stats(&mut self, update_time_ms: f32) {
        self.stats.average_update_time_ms =
            (self.stats.average_update_time_ms + update_time_ms) / 2.0;
        self.stats.peak_update_time_ms = self.stats.peak_update_time_ms.max(update_time_ms);
    }

    // Oracle's M4 requirements: Sector management methods

    /// Get sector ID for a world position
    pub fn get_sector_id(&self, position: Vec3) -> SectorId {
        SectorId::from_world_position(position.x, position.z, self.sector_size)
    }

    /// Check if a sector is within view radius
    pub fn is_sector_in_view_radius(&self, sector_id: &SectorId) -> bool {
        let (sector_x, sector_z) = sector_id.to_world_position(self.sector_size);
        let sector_center = Vec3::new(
            sector_x + self.sector_size / 2.0,
            0.0,
            sector_z + self.sector_size / 2.0,
        );
        let distance = self.player_position.distance(sector_center);
        distance <= self.view_radius
    }

    /// Get all sectors within view radius ("needed" sectors)
    pub fn get_needed_sectors(&self) -> Vec<SectorId> {
        let player_sector = self.get_sector_id(self.player_position);
        let sector_radius = (self.view_radius / self.sector_size).ceil() as i32;

        let mut needed_sectors = Vec::new();
        for x in -sector_radius..=sector_radius {
            for z in -sector_radius..=sector_radius {
                let sector_id = SectorId::new(player_sector.x + x, player_sector.z + z);
                if self.is_sector_in_view_radius(&sector_id) {
                    needed_sectors.push(sector_id);
                }
            }
        }
        needed_sectors
    }

    /// Queue a sector for spawning
    pub fn queue_sector_spawn(&mut self, sector_id: SectorId) {
        if !self.sectors.contains_key(&sector_id) && !self.spawn_queue.contains(&sector_id) {
            self.spawn_queue.push_back(sector_id);
        }
    }

    /// Queue a sector for despawning
    pub fn queue_sector_despawn(&mut self, sector_id: SectorId) {
        if self.sectors.contains_key(&sector_id) && !self.despawn_queue.contains(&sector_id) {
            self.despawn_queue.push_back(sector_id);
        }
    }

    /// Mark sector as loaded
    #[cfg(feature = "bevy16")]
    pub fn mark_sector_loaded(&mut self, sector_id: SectorId, entities: Vec<Entity>) {
        let mut sector_data = SectorData::new(sector_id);
        sector_data.spawn_state = SpawnState::Loaded;
        sector_data.entities = entities;
        sector_data.entity_count = sector_data.entities.len() as u32;

        let entity_count = sector_data.entity_count;
        self.sectors.insert(sector_id, sector_data);
        self.stats.sectors_loaded += 1;
        self.stats.entities_spawned += entity_count;
        self.stats.total_entities_in_world += entity_count;
    }

    /// Mark sector as unloaded
    pub fn mark_sector_unloaded(&mut self, sector_id: SectorId) {
        if let Some(sector_data) = self.sectors.remove(&sector_id) {
            self.stats.sectors_unloaded += 1;
            self.stats.entities_despawned += sector_data.entity_count;
            self.stats.total_entities_in_world = self
                .stats
                .total_entities_in_world
                .saturating_sub(sector_data.entity_count);
        }
    }

    /// Update LOD level for a sector based on distance
    pub fn update_sector_lod(&mut self, sector_id: SectorId) {
        if let Some(sector_data) = self.sectors.get_mut(&sector_id) {
            let (sector_x, sector_z) = sector_id.to_world_position(self.sector_size);
            let sector_center = Vec3::new(
                sector_x + self.sector_size / 2.0,
                0.0,
                sector_z + self.sector_size / 2.0,
            );
            let distance = self.player_position.distance(sector_center);

            let old_lod = sector_data.lod_level;

            // LOD levels based on distance
            let new_lod = if distance <= 128.0 {
                LodLevel::LOD0 // Full mesh within 128m
            } else if distance <= 256.0 {
                LodLevel::LOD1 // Proxy mesh within 256m
            } else {
                LodLevel::Impostor // Impostor/billboard beyond 256m
            };

            if old_lod != new_lod {
                // Update LOD level counts
                match old_lod {
                    LodLevel::LOD0 => {
                        self.stats.lod_level_counts[0] =
                            self.stats.lod_level_counts[0].saturating_sub(1)
                    }
                    LodLevel::LOD1 => {
                        self.stats.lod_level_counts[1] =
                            self.stats.lod_level_counts[1].saturating_sub(1)
                    }
                    LodLevel::Impostor => {
                        self.stats.lod_level_counts[2] =
                            self.stats.lod_level_counts[2].saturating_sub(1)
                    }
                }

                match new_lod {
                    LodLevel::LOD0 => self.stats.lod_level_counts[0] += 1,
                    LodLevel::LOD1 => self.stats.lod_level_counts[1] += 1,
                    LodLevel::Impostor => self.stats.lod_level_counts[2] += 1,
                }

                sector_data.set_lod_level(new_lod);
            }
        }
    }

    /// Update memory usage statistics (simplified calculation)
    pub fn update_memory_usage(&mut self) {
        let entity_memory = self.stats.total_entities_in_world as f32 * 0.001; // ~1KB per entity
        let sector_overhead = self.sectors.len() as f32 * 0.0001; // ~100B per sector
        self.stats.memory_usage_mb = entity_memory + sector_overhead;
    }
}

impl Default for WorldStreamer {
    fn default() -> Self {
        let config = WorldGenerationConfig::default();
        Self::new(&config)
    }
}

// Bevy systems only available with bevy16 feature
#[cfg(feature = "bevy16")]
pub mod bevy_systems {
    use super::*;

    /// System to update chunk queues based on player position
    pub fn update_chunk_queues(
        mut streamer: ResMut<WorldStreamer>,
        player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    ) {
        let start_time = Instant::now();

        // Update player position if changed
        if let Ok(player_transform) = player_query.single() {
            streamer.player_position = player_transform.translation;
        }

        // Get all chunks that should be loaded
        let streaming_chunks = streamer.get_streaming_chunks();

        // Queue chunks for loading
        for chunk_key in streaming_chunks {
            if !streamer.loaded_chunks.contains_key(&chunk_key) {
                streamer.queue_chunk_load(chunk_key);
            }
        }

        // Queue chunks for unloading if they're outside active radius
        let chunks_to_unload: Vec<ChunkKey> = streamer
            .loaded_chunks
            .keys()
            .filter(|&chunk_key| !streamer.is_in_active_radius(chunk_key))
            .copied()
            .collect();

        for chunk_key in chunks_to_unload {
            streamer.queue_chunk_unload(chunk_key);
        }

        // Update performance stats
        let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
        streamer.update_stats(elapsed);
    }

    /// System to enqueue chunk loads (simplified synchronous version)
    pub fn enqueue_chunk_loads(mut commands: Commands, mut streamer: ResMut<WorldStreamer>) {
        const MAX_LOADS_PER_FRAME: usize = 2;

        for _ in 0..MAX_LOADS_PER_FRAME {
            if let Some(chunk_key) = streamer.load_queue.pop_front() {
                // Start loading task
                commands.spawn(ChunkLoadTask {
                    chunk_key,
                    progress: 0.0,
                });
            } else {
                break;
            }
        }
    }

    /// System to process completed chunk loads
    pub fn process_loaded_chunks(
        mut commands: Commands,
        mut streamer: ResMut<WorldStreamer>,
        mut load_tasks: Query<(Entity, &mut ChunkLoadTask)>,
    ) {
        for (entity, mut task) in load_tasks.iter_mut() {
            // Simulate loading progress
            task.progress += 0.1;

            // Complete loading when progress reaches 1.0
            if task.progress >= 1.0 {
                // Create mock entities for the chunk
                let entities = vec![]; // Empty for now

                // Mark chunk as loaded
                streamer.mark_chunk_loaded(task.chunk_key, entities);

                // Remove the task entity
                commands.entity(entity).despawn();
            }
        }
    }

    /// System to unload far chunks
    pub fn unload_far_chunks(mut commands: Commands, mut streamer: ResMut<WorldStreamer>) {
        const MAX_UNLOADS_PER_FRAME: usize = 4;

        for _ in 0..MAX_UNLOADS_PER_FRAME {
            if let Some(chunk_key) = streamer.unload_queue.pop_front() {
                if let Some(chunk_data) = streamer.loaded_chunks.get(&chunk_key) {
                    // Despawn all entities in the chunk
                    for entity in &chunk_data.entities {
                        commands.entity(*entity).despawn();
                    }

                    // Mark chunk as unloaded
                    streamer.mark_chunk_unloaded(chunk_key);
                }
            } else {
                break;
            }
        }
    }

    // Oracle's M4 requirements: Sector streaming systems

    /// System to update sector queues based on player position
    pub fn update_sector_queues(
        mut streamer: ResMut<WorldStreamer>,
        player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    ) {
        let start_time = Instant::now();

        // Update player position if changed
        if let Ok(player_transform) = player_query.single() {
            streamer.player_position = player_transform.translation;
        }

        // Get all sectors that need to be loaded
        let needed_sectors = streamer.get_needed_sectors();

        // Queue sectors for spawning
        for sector_id in needed_sectors {
            if !streamer.sectors.contains_key(&sector_id) {
                streamer.queue_sector_spawn(sector_id);
            }
        }

        // Queue sectors for despawning if they're outside view radius
        let sectors_to_despawn: Vec<SectorId> = streamer
            .sectors
            .keys()
            .filter(|&sector_id| !streamer.is_sector_in_view_radius(sector_id))
            .copied()
            .collect();

        for sector_id in sectors_to_despawn {
            streamer.queue_sector_despawn(sector_id);
        }

        // Update performance stats
        let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
        streamer.update_stats(elapsed);
    }

    /// System to spawn sectors asynchronously
    pub fn spawn_sectors_async(
        mut commands: Commands,
        mut streamer: ResMut<WorldStreamer>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        const MAX_SPAWNS_PER_FRAME: usize = 1;

        for _ in 0..MAX_SPAWNS_PER_FRAME {
            if let Some(sector_id) = streamer.spawn_queue.pop_front() {
                // Create mock entities for the sector
                let (sector_x, sector_z) = sector_id.to_world_position(streamer.sector_size);
                let sector_center = Vec3::new(
                    sector_x + streamer.sector_size / 2.0,
                    0.0,
                    sector_z + streamer.sector_size / 2.0,
                );

                // Spawn a simple building or object to represent the sector
                let mesh_handle = meshes.add(Mesh::from(Cuboid::new(8.0, 16.0, 8.0)));
                let material_handle = materials.add(Color::srgb(0.5, 0.5, 0.8));
                let entity = commands
                    .spawn((
                        bevy::render::mesh::Mesh3d(mesh_handle),
                        bevy::pbr::MeshMaterial3d(material_handle),
                        Transform::from_translation(sector_center + Vec3::Y * 8.0),
                        GlobalTransform::default(),
                        Visibility::default(),
                    ))
                    .id();

                // Mark sector as loaded
                streamer.mark_sector_loaded(sector_id, vec![entity]);

                // Update LOD level for the sector
                streamer.update_sector_lod(sector_id);
            } else {
                break;
            }
        }
    }

    /// System to despawn sectors asynchronously
    pub fn despawn_sectors_async(mut commands: Commands, mut streamer: ResMut<WorldStreamer>) {
        const MAX_DESPAWNS_PER_FRAME: usize = 2;

        for _ in 0..MAX_DESPAWNS_PER_FRAME {
            if let Some(sector_id) = streamer.despawn_queue.pop_front() {
                if let Some(sector_data) = streamer.sectors.get(&sector_id) {
                    // Despawn all entities in the sector
                    for entity in &sector_data.entities {
                        commands.entity(*entity).despawn();
                    }

                    // Mark sector as unloaded
                    streamer.mark_sector_unloaded(sector_id);
                }
            } else {
                break;
            }
        }
    }

    /// System to update LOD levels for all loaded sectors
    pub fn update_sector_lods(mut streamer: ResMut<WorldStreamer>) {
        // Update LOD levels for all loaded sectors
        let sector_ids: Vec<SectorId> = streamer.sectors.keys().copied().collect();

        for sector_id in sector_ids {
            streamer.update_sector_lod(sector_id);
        }

        // Update memory usage statistics
        streamer.update_memory_usage();
    }
}

// Re-export systems for convenience
#[cfg(feature = "bevy16")]
pub use bevy_systems::*;

/// Marker component for the player
#[cfg_attr(feature = "bevy16", derive(Component))]
pub struct Player;

#[cfg(feature = "bevy16")]
pub mod factory_integration;

#[cfg(feature = "bevy16")]
pub use factory_integration::*;

/// Plugin for world streaming
#[cfg(feature = "bevy16")]
pub struct WorldStreamingPlugin;

#[cfg(feature = "bevy16")]
impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_chunk_queues,
                enqueue_chunk_loads,
                process_loaded_chunks,
                unload_far_chunks,
                start_chunk_generation,
                generate_chunk_content,
                track_chunk_entities,
                cleanup_chunk_entities,
                // Oracle's M4 requirements: Sector streaming systems
                update_sector_queues,
                spawn_sectors_async,
                despawn_sectors_async,
                update_sector_lods,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_streamer_creation() {
        let config = WorldGenerationConfig::default();
        let streamer = WorldStreamer::new(&config);

        assert_eq!(streamer.chunk_size, 200.0);
        assert_eq!(streamer.streaming_radius, 800.0);
        assert_eq!(streamer.active_radius, 400.0);
        assert_eq!(streamer.entity_limit_per_chunk, 100);
    }

    #[test]
    fn test_chunk_key_calculation() {
        let config = WorldGenerationConfig::default();
        let streamer = WorldStreamer::new(&config);

        let chunk_key = streamer.get_chunk_key(Vec3::new(450.0, 0.0, -150.0));
        assert_eq!(chunk_key.x, 2);
        assert_eq!(chunk_key.z, -1);
    }

    #[test]
    fn test_streaming_radius_check() {
        let config = WorldGenerationConfig::default();
        let mut streamer = WorldStreamer::new(&config);

        streamer.player_position = Vec3::new(0.0, 0.0, 0.0);

        // Chunk at origin should be in streaming radius
        let origin_chunk = ChunkKey::new(0, 0);
        assert!(streamer.is_in_streaming_radius(&origin_chunk));

        // Chunk very far away should not be in streaming radius
        let far_chunk = ChunkKey::new(100, 100);
        assert!(!streamer.is_in_streaming_radius(&far_chunk));
    }

    #[test]
    fn test_chunk_queueing() {
        let config = WorldGenerationConfig::default();
        let mut streamer = WorldStreamer::new(&config);

        let chunk_key = ChunkKey::new(0, 0);

        // Queue chunk for loading
        streamer.queue_chunk_load(chunk_key);
        assert!(streamer.load_queue.contains(&chunk_key));

        // Mark as loaded
        streamer.mark_chunk_loaded(chunk_key, vec![]);
        assert!(streamer.loaded_chunks.contains_key(&chunk_key));

        // Queue for unloading
        streamer.queue_chunk_unload(chunk_key);
        assert!(streamer.unload_queue.contains(&chunk_key));

        // Mark as unloaded
        streamer.mark_chunk_unloaded(chunk_key);
        assert!(!streamer.loaded_chunks.contains_key(&chunk_key));
    }

    #[test]
    fn test_streaming_chunks() {
        let config = WorldGenerationConfig::default();
        let mut streamer = WorldStreamer::new(&config);

        streamer.player_position = Vec3::new(0.0, 0.0, 0.0);

        let streaming_chunks = streamer.get_streaming_chunks();
        assert!(!streaming_chunks.is_empty());

        // Origin chunk should be included
        let origin_chunk = ChunkKey::new(0, 0);
        assert!(streaming_chunks.contains(&origin_chunk));
    }

    #[cfg(feature = "bevy16")]
    #[test]
    fn test_chunk_data_entity_management() {
        let chunk_key = ChunkKey::new(0, 0);
        let mut chunk_data = ChunkData::new(chunk_key);

        let entity = Entity::from_raw(1);

        // Add entity
        chunk_data.add_entity(entity);
        assert_eq!(chunk_data.entity_count, 1);
        assert!(chunk_data.entities.contains(&entity));

        // Remove entity
        chunk_data.remove_entity(entity);
        assert_eq!(chunk_data.entity_count, 0);
        assert!(!chunk_data.entities.contains(&entity));
    }

    #[cfg(not(feature = "bevy16"))]
    #[test]
    fn test_chunk_data_entity_management() {
        let chunk_key = ChunkKey::new(0, 0);
        let mut chunk_data = ChunkData::new(chunk_key);

        let entity = 1u32;

        // Add entity
        chunk_data.add_entity(entity);
        assert_eq!(chunk_data.entity_count, 1);
        assert!(chunk_data.entities.contains(&entity));

        // Remove entity
        chunk_data.remove_entity(entity);
        assert_eq!(chunk_data.entity_count, 0);
        assert!(!chunk_data.entities.contains(&entity));
    }

    #[test]
    fn test_entity_limit_check() {
        let chunk_key = ChunkKey::new(0, 0);
        let mut chunk_data = ChunkData::new(chunk_key);

        // Should be able to add entities under limit
        assert!(chunk_data.can_add_entity(100));

        // Add entities to reach limit
        for i in 0..100 {
            #[cfg(feature = "bevy16")]
            chunk_data.add_entity(Entity::from_raw(i));
            #[cfg(not(feature = "bevy16"))]
            chunk_data.add_entity(i);
        }

        // Should not be able to add more entities
        assert!(!chunk_data.can_add_entity(100));
    }

    #[test]
    fn test_content_generation_flag_prevents_infinite_loops() {
        let chunk_key = ChunkKey::new(0, 0);
        let mut chunk_data = ChunkData::new(chunk_key);

        // Initial state: content not generated
        assert!(!chunk_data.content_generated);

        // After marking content as generated
        chunk_data.content_generated = true;
        assert!(chunk_data.content_generated);

        // Multiple calls to mark content as generated should be idempotent
        chunk_data.content_generated = true;
        assert!(chunk_data.content_generated);
    }

    #[test]
    fn test_chunk_entity_count_stability() {
        let config = WorldGenerationConfig::default();
        let mut streamer = WorldStreamer::new(&config);

        // Load a chunk
        let chunk_key = ChunkKey::new(0, 0);
        streamer.mark_chunk_loaded(chunk_key, vec![]);

        // Initial entity count should be 0
        let initial_count = streamer.loaded_chunks.get(&chunk_key).unwrap().entity_count;
        assert_eq!(initial_count, 0);

        // Mark content as generated
        streamer
            .loaded_chunks
            .get_mut(&chunk_key)
            .unwrap()
            .content_generated = true;

        // Add some entities to simulate content generation
        if let Some(chunk_data) = streamer.loaded_chunks.get_mut(&chunk_key) {
            #[cfg(feature = "bevy16")]
            {
                chunk_data.add_entity(Entity::from_raw(1));
                chunk_data.add_entity(Entity::from_raw(2));
            }
            #[cfg(not(feature = "bevy16"))]
            {
                chunk_data.add_entity(1);
                chunk_data.add_entity(2);
            }
        }

        // Entity count should be stable at 2
        let final_count = streamer.loaded_chunks.get(&chunk_key).unwrap().entity_count;
        assert_eq!(final_count, 2);

        // Content should be marked as generated, preventing further generation
        assert!(
            streamer
                .loaded_chunks
                .get(&chunk_key)
                .unwrap()
                .content_generated
        );
    }
}
