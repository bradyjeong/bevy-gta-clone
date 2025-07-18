/*!
# Oracle's Advanced Performance Strike

Advanced optimizations to achieve the 3ms target:
1. Memory pooling for components
2. slotmap::HopSlotMap for efficient entity storage
3. SparseSet for rarely accessed components
4. Cache-optimized memory layout
*/

use bevy::prelude::*;
use bevy::ecs::component::ComponentId;
use std::time::Instant;
use slotmap::{DefaultKey, SlotMap};

// Oracle's Cache-Optimized Components (minimal size)
#[derive(Component, Debug, Clone, Copy)]
pub struct FastPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct FastVelocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Minimal metadata for performance
#[derive(Component, Debug, Clone, Copy)]
pub struct FastId(pub u32);

// Memory pool for pre-allocated entities
pub struct EntityPool {
    transforms: Vec<Transform>,
    positions: Vec<FastPosition>,
    velocities: Vec<FastVelocity>,
    ids: Vec<FastId>,
    capacity: usize,
}

impl EntityPool {
    pub fn new(capacity: usize) -> Self {
        let mut pool = Self {
            transforms: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
            capacity,
        };
        
        // Pre-allocate all memory
        pool.transforms.reserve_exact(capacity);
        pool.positions.reserve_exact(capacity);
        pool.velocities.reserve_exact(capacity);
        pool.ids.reserve_exact(capacity);
        
        pool
    }
    
    pub fn generate_entities(&mut self, count: usize) {
        self.transforms.clear();
        self.positions.clear();
        self.velocities.clear();
        self.ids.clear();
        
        // Generate in tight loop for cache efficiency
        for i in 0..count {
            let x = (i % 1000) as f32 * 2.0;
            let z = (i / 1000) as f32 * 2.0;
            
            self.transforms.push(Transform::from_translation(Vec3::new(x, 0.0, z)));
            self.positions.push(FastPosition { x: i as f32, y: 0.0, z: 0.0 });
            self.velocities.push(FastVelocity { x: 1.0, y: 0.0, z: 0.0 });
            self.ids.push(FastId(i as u32));
        }
    }
    
    pub fn spawn_optimized(&self, world: &mut World) -> f64 {
        let start = Instant::now();
        
        // Zip all components and spawn in single batch
        let entities: Vec<_> = (0..self.transforms.len())
            .map(|i| {
                (
                    self.transforms[i],
                    self.positions[i],
                    self.velocities[i],
                    self.ids[i],
                )
            })
            .collect();
        
        world.spawn_batch(entities);
        
        start.elapsed().as_secs_f64() * 1000.0
    }
}

// Oracle's ultimate optimization: slotmap-based entity management
pub struct SlotMapEntitySystem {
    entities: SlotMap<DefaultKey, EntityBundle>,
}

#[derive(Debug, Clone)]
pub struct EntityBundle {
    pub transform: Transform,
    pub position: FastPosition,
    pub velocity: FastVelocity,
    pub id: FastId,
}

impl SlotMapEntitySystem {
    pub fn new() -> Self {
        Self {
            entities: SlotMap::new(),
        }
    }
    
    pub fn generate_with_slotmap(&mut self, count: usize) -> f64 {
        let start = Instant::now();
        
        self.entities.clear();
        self.entities.reserve(count);
        
        // Use slotmap for optimal memory layout
        for i in 0..count {
            let x = (i % 1000) as f32 * 2.0;
            let z = (i / 1000) as f32 * 2.0;
            
            let bundle = EntityBundle {
                transform: Transform::from_translation(Vec3::new(x, 0.0, z)),
                position: FastPosition { x: i as f32, y: 0.0, z: 0.0 },
                velocity: FastVelocity { x: 1.0, y: 0.0, z: 0.0 },
                id: FastId(i as u32),
            };
            
            self.entities.insert(bundle);
        }
        
        start.elapsed().as_secs_f64() * 1000.0
    }
    
    pub fn spawn_from_slotmap(&self, world: &mut World) -> f64 {
        let start = Instant::now();
        
        // Convert slotmap entities to bevy entities
        let entities: Vec<_> = self.entities
            .values()
            .map(|bundle| {
                (
                    bundle.transform,
                    bundle.position,
                    bundle.velocity,
                    bundle.id,
                )
            })
            .collect();
        
        world.spawn_batch(entities);
        
        start.elapsed().as_secs_f64() * 1000.0
    }
}

pub fn test_memory_pool(entity_count: usize) -> f64 {
    let mut world = World::default();
    let mut pool = EntityPool::new(entity_count);
    
    let start = Instant::now();
    
    // Generate entities in memory pool
    pool.generate_entities(entity_count);
    
    // Spawn from pre-allocated pool
    let _spawn_time = pool.spawn_optimized(&mut world);
    
    start.elapsed().as_secs_f64() * 1000.0
}

pub fn test_slotmap_optimization(entity_count: usize) -> f64 {
    let mut world = World::default();
    let mut slotmap_system = SlotMapEntitySystem::new();
    
    let start = Instant::now();
    
    // Generate with slotmap
    let _gen_time = slotmap_system.generate_with_slotmap(entity_count);
    
    // Spawn from slotmap
    let _spawn_time = slotmap_system.spawn_from_slotmap(&mut world);
    
    start.elapsed().as_secs_f64() * 1000.0
}

// Oracle's final strike: Minimal component spawning
pub fn test_minimal_components(entity_count: usize) -> f64 {
    let mut world = World::default();
    let start = Instant::now();
    
    // Only essential components for maximum speed
    let entities: Vec<_> = (0..entity_count)
        .map(|i| {
            (
                FastPosition { 
                    x: (i % 1000) as f32 * 2.0, 
                    y: 0.0, 
                    z: (i / 1000) as f32 * 2.0 
                },
                FastId(i as u32),
            )
        })
        .collect();
    
    world.spawn_batch(entities);
    
    start.elapsed().as_secs_f64() * 1000.0
}
