//! Spawn Budget Policy (SBP) - Phase 1: Oracle's Archaeological Recovery
//!
//! **Oracle's Mandate**: "Spawn chaos must be contained with professional discipline"
//!
//! This module resurrects and modernizes the spawn budget enforcement discipline
//! from f430bc6, preventing performance spikes from undisciplined entity creation.
//!
//! ## Original f430bc6 Architecture Analysis
//!
//! The original system enforced strict caps through:
//! - **Entity Limits**: buildings: 80, vehicles: 20, npcs: 2, trees: 100, particles: 50
//! - **Per-Frame Limits**: max_spawns_per_frame: 50, max_despawns_per_frame: 100
//! - **Streaming Budget**: max_spawn_queue_length: 1000, time_budget_ms: 16.67
//! - **Distance Culling**: buildings: 300m, vehicles: 150m, npcs: 100m, vegetation: 200m
//!
//! ## Current Spawn Sites Requiring Integration
//!
//! **Factory Systems** (Priority 1):
//! - `gameplay_factory::spawn()` - Main factory entry point
//! - `npc_factory::spawn_npc()` and `spawn_npcs_batch()`
//! - `vehicle_factory::spawn_vehicle()`
//! - `prefab_factory::spawn_prefab()` and `batch_spawn()`
//!
//! **World Streaming** (Priority 2):
//! - `world_streaming::generate_buildings()` - Already has `can_add_entity()` checks
//! - `world_streaming::generate_vehicles()` - Already has limit enforcement
//! - `world_streaming::generate_npcs()` - Already has limit enforcement
//!
//! **Direct Spawning** (Priority 3):
//! - Multiple `commands.spawn()` calls bypassing factory systems
//! - City infrastructure spawning in `city::systems.rs`
//!
//! ## Oracle's Architectural Requirements
//!
//! 1. **Biome-Specific Caps**: Different entity budgets per environment type
//! 2. **Graceful Degradation**: Queue overflow handling without crashes
//! 3. **Performance Monitoring**: Real-time budget utilization tracking
//! 4. **Hot-Reload Support**: Runtime configuration updates for tuning

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Spawn Budget Policy - Central authority for entity creation discipline
///
/// **Oracle's Vision**: "Every spawn must justify its existence against the budget"
#[derive(Resource, Debug, Clone)]
pub struct SpawnBudgetPolicy {
    /// Current entity counts by type
    pub current_counts: EntityCounts,
    /// Maximum allowed entities by type and biome
    pub budget_caps: BiomeBudgetCaps,
    /// Per-frame spawn limits to prevent performance spikes
    pub frame_limits: FrameLimits,
    /// Spawn queue for deferred entity creation
    pub spawn_queue: SpawnQueue,
    /// Performance monitoring and metrics
    pub metrics: BudgetMetrics,
}

/// Entity counts by type - tracks current spawn budget utilization
#[derive(Debug, Clone, Default)]
pub struct EntityCounts {
    pub buildings: u32,
    pub vehicles: u32,
    pub npcs: u32,
    pub trees: u32,
    pub particles: u32,
    pub total: u32,
}

/// Biome-specific budget caps - Oracle's environmental discipline
///
/// Different environments have different performance characteristics and gameplay needs:
/// - **Urban**: High building density, moderate vehicles, low NPCs
/// - **Suburban**: Balanced distribution with more trees
/// - **Rural**: Low buildings, high trees, minimal vehicles
/// - **Industrial**: High particles, moderate buildings, minimal NPCs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeBudgetCaps {
    pub urban: EntityBudget,
    pub suburban: EntityBudget,
    pub rural: EntityBudget,
    pub industrial: EntityBudget,
    pub default: EntityBudget,
}

/// Entity budget for a specific biome type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBudget {
    pub buildings: u32,
    pub vehicles: u32,
    pub npcs: u32,
    pub trees: u32,
    pub particles: u32,
    pub total_cap: u32,
}

/// Per-frame spawn limits to prevent performance spikes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameLimits {
    pub max_spawns_per_frame: u32,
    pub max_despawns_per_frame: u32,
    pub time_budget_ms: f32,
    pub priority_threshold: f32,
}

/// Spawn queue for deferred entity creation when budget is exceeded
#[derive(Debug, Clone)]
pub struct SpawnQueue {
    pub pending_spawns: Vec<PendingSpawn>,
    pub max_queue_length: u32,
    pub current_frame_spawns: u32,
}

/// Pending spawn request in the budget queue
#[derive(Debug, Clone)]
pub struct PendingSpawn {
    pub entity_type: EntityType,
    pub biome: BiomeType,
    pub priority: SpawnPriority,
    pub spawn_data: SpawnData,
    pub queued_at: f32, // Game time when queued
}

/// Entity types for budget tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Building,
    Vehicle,
    Npc,
    Tree,
    Particle,
}

/// Biome types for environment-specific budgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Urban,
    Suburban,
    Rural,
    Industrial,
}

/// Spawn priority for queue management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpawnPriority {
    Critical = 0, // Essential gameplay elements
    High = 1,     // Important visual elements
    Medium = 2,   // Standard content
    Low = 3,      // Background/ambient content
}

/// Spawn request data - polymorphic spawn parameters
#[derive(Debug, Clone)]
pub enum SpawnData {
    Building {
        position: Vec3,
        building_type: String,
    },
    Vehicle {
        position: Vec3,
        vehicle_type: String,
    },
    Npc {
        position: Vec3,
        npc_type: String,
    },
    Tree {
        position: Vec3,
        tree_type: String,
    },
    Particle {
        position: Vec3,
        effect_type: String,
    },
}

/// Performance metrics for budget monitoring
#[derive(Debug, Clone, Default)]
pub struct BudgetMetrics {
    pub total_spawn_requests: u64,
    pub successful_spawns: u64,
    pub rejected_spawns: u64,
    pub queued_spawns: u64,
    pub average_queue_time: f32,
    pub budget_utilization: f32, // Percentage of total budget used
    pub frame_time_impact: f32,  // Milliseconds spent on spawning this frame
}

/// Spawn budget enforcement result
#[derive(Debug, Clone, PartialEq)]
pub enum SpawnResult {
    /// Spawn approved and executed immediately
    Approved,
    /// Spawn queued for later execution due to budget constraints
    Queued,
    /// Spawn rejected due to hard limits or queue overflow
    Rejected(RejectionReason),
}

/// Reasons for spawn rejection
#[derive(Debug, Clone, PartialEq)]
pub enum RejectionReason {
    /// Entity type budget exceeded for current biome
    EntityCapExceeded(EntityType),
    /// Total entity cap exceeded
    TotalCapExceeded,
    /// Per-frame spawn limit exceeded
    FrameLimitExceeded,
    /// Spawn queue is full
    QueueOverflow,
    /// Time budget for this frame exceeded
    TimeBudgetExceeded,
}

impl Default for SpawnBudgetPolicy {
    fn default() -> Self {
        Self {
            current_counts: EntityCounts::default(),
            budget_caps: BiomeBudgetCaps::default(),
            frame_limits: FrameLimits::default(),
            spawn_queue: SpawnQueue::default(),
            metrics: BudgetMetrics::default(),
        }
    }
}

impl Default for BiomeBudgetCaps {
    /// Default budget caps based on f430bc6 original values
    fn default() -> Self {
        let default_budget = EntityBudget {
            buildings: 80,
            vehicles: 20,
            npcs: 2,
            trees: 100,
            particles: 50,
            total_cap: 252, // Sum of individual caps
        };

        Self {
            urban: EntityBudget {
                buildings: 120, // Higher building density in urban areas
                vehicles: 30,   // More traffic
                npcs: 5,        // More people
                trees: 50,      // Fewer trees
                particles: 75,  // More effects
                total_cap: 280,
            },
            suburban: EntityBudget {
                buildings: 60, // Moderate buildings
                vehicles: 25,  // Residential traffic
                npcs: 3,       // Some pedestrians
                trees: 150,    // More vegetation
                particles: 40, // Fewer effects
                total_cap: 278,
            },
            rural: EntityBudget {
                buildings: 20, // Few buildings
                vehicles: 10,  // Minimal traffic
                npcs: 1,       // Rare pedestrians
                trees: 200,    // Lots of vegetation
                particles: 20, // Natural effects only
                total_cap: 251,
            },
            industrial: EntityBudget {
                buildings: 80,  // Factories/warehouses
                vehicles: 40,   // Heavy machinery
                npcs: 2,        // Workers only
                trees: 20,      // Minimal vegetation
                particles: 100, // Smoke/steam effects
                total_cap: 242,
            },
            default: default_budget,
        }
    }
}

impl Default for FrameLimits {
    /// Default frame limits based on f430bc6 performance tuning
    fn default() -> Self {
        Self {
            max_spawns_per_frame: 50,
            max_despawns_per_frame: 100,
            time_budget_ms: 16.67, // 60 FPS target
            priority_threshold: 0.5,
        }
    }
}

impl Default for SpawnQueue {
    fn default() -> Self {
        Self {
            pending_spawns: Vec::new(),
            max_queue_length: 1000,
            current_frame_spawns: 0,
        }
    }
}

impl SpawnBudgetPolicy {
    /// Create new spawn budget policy with configuration
    pub fn new(config: BiomeBudgetCaps, frame_limits: FrameLimits) -> Self {
        Self {
            current_counts: EntityCounts::default(),
            budget_caps: config,
            frame_limits,
            spawn_queue: SpawnQueue::default(),
            metrics: BudgetMetrics::default(),
        }
    }

    /// Check if spawn request can be approved for given biome
    pub fn can_spawn(&self, entity_type: EntityType, biome: BiomeType) -> bool {
        let budget = self.get_budget_for_biome(biome);
        let current_count = self.get_current_count(entity_type);
        let entity_cap = self.get_entity_cap(entity_type, budget);

        current_count < entity_cap && self.current_counts.total < budget.total_cap
    }

    /// Request spawn with budget enforcement
    pub fn request_spawn(
        &mut self,
        entity_type: EntityType,
        biome: BiomeType,
        priority: SpawnPriority,
        spawn_data: SpawnData,
        game_time: f32,
    ) -> SpawnResult {
        // Check frame limits first
        if self.spawn_queue.current_frame_spawns >= self.frame_limits.max_spawns_per_frame {
            return SpawnResult::Rejected(RejectionReason::FrameLimitExceeded);
        }

        // Check budget availability
        if self.can_spawn(entity_type, biome) {
            // Immediate spawn approval
            self.record_spawn(entity_type);
            self.spawn_queue.current_frame_spawns += 1;
            self.metrics.successful_spawns += 1;
            SpawnResult::Approved
        } else {
            // Queue for later or reject
            self.queue_spawn(entity_type, biome, priority, spawn_data, game_time)
        }
    }

    /// Queue spawn request for later execution
    fn queue_spawn(
        &mut self,
        entity_type: EntityType,
        biome: BiomeType,
        priority: SpawnPriority,
        spawn_data: SpawnData,
        game_time: f32,
    ) -> SpawnResult {
        if self.spawn_queue.pending_spawns.len() >= self.spawn_queue.max_queue_length as usize {
            self.metrics.rejected_spawns += 1;
            return SpawnResult::Rejected(RejectionReason::QueueOverflow);
        }

        let pending_spawn = PendingSpawn {
            entity_type,
            biome,
            priority,
            spawn_data,
            queued_at: game_time,
        };

        self.spawn_queue.pending_spawns.push(pending_spawn);
        self.metrics.queued_spawns += 1;
        SpawnResult::Queued
    }

    /// Process queued spawns within frame budget
    pub fn process_spawn_queue(&mut self, game_time: f32) -> Vec<(EntityType, SpawnData)> {
        let mut spawned = Vec::new();
        let mut remaining_spawns = self
            .frame_limits
            .max_spawns_per_frame
            .saturating_sub(self.spawn_queue.current_frame_spawns);

        // Sort by priority (Critical first)
        self.spawn_queue
            .pending_spawns
            .sort_by_key(|spawn| spawn.priority);

        let mut i = 0;
        while i < self.spawn_queue.pending_spawns.len() && remaining_spawns > 0 {
            let spawn = &self.spawn_queue.pending_spawns[i];

            if self.can_spawn(spawn.entity_type, spawn.biome) {
                let spawn = self.spawn_queue.pending_spawns.remove(i);
                self.record_spawn(spawn.entity_type);
                spawned.push((spawn.entity_type, spawn.spawn_data));
                remaining_spawns -= 1;
                self.spawn_queue.current_frame_spawns += 1;
                self.metrics.successful_spawns += 1;

                // Update average queue time metric
                let queue_time = game_time - spawn.queued_at;
                self.metrics.average_queue_time =
                    (self.metrics.average_queue_time * 0.9) + (queue_time * 0.1);
            } else {
                i += 1;
            }
        }

        spawned
    }

    /// Record successful spawn in current counts
    pub fn record_spawn(&mut self, entity_type: EntityType) {
        match entity_type {
            EntityType::Building => self.current_counts.buildings += 1,
            EntityType::Vehicle => self.current_counts.vehicles += 1,
            EntityType::Npc => self.current_counts.npcs += 1,
            EntityType::Tree => self.current_counts.trees += 1,
            EntityType::Particle => self.current_counts.particles += 1,
        }
        self.current_counts.total += 1;
        self.metrics.total_spawn_requests += 1;
    }

    /// Record entity despawn
    pub fn record_despawn(&mut self, entity_type: EntityType) {
        match entity_type {
            EntityType::Building => {
                self.current_counts.buildings = self.current_counts.buildings.saturating_sub(1)
            }
            EntityType::Vehicle => {
                self.current_counts.vehicles = self.current_counts.vehicles.saturating_sub(1)
            }
            EntityType::Npc => {
                self.current_counts.npcs = self.current_counts.npcs.saturating_sub(1)
            }
            EntityType::Tree => {
                self.current_counts.trees = self.current_counts.trees.saturating_sub(1)
            }
            EntityType::Particle => {
                self.current_counts.particles = self.current_counts.particles.saturating_sub(1)
            }
        }
        self.current_counts.total = self.current_counts.total.saturating_sub(1);
    }

    /// Reset frame counters (call at start of each frame)
    pub fn reset_frame_counters(&mut self) {
        self.spawn_queue.current_frame_spawns = 0;
    }

    /// Get budget for specific biome
    fn get_budget_for_biome(&self, biome: BiomeType) -> &EntityBudget {
        match biome {
            BiomeType::Urban => &self.budget_caps.urban,
            BiomeType::Suburban => &self.budget_caps.suburban,
            BiomeType::Rural => &self.budget_caps.rural,
            BiomeType::Industrial => &self.budget_caps.industrial,
        }
    }

    /// Get current count for entity type
    fn get_current_count(&self, entity_type: EntityType) -> u32 {
        match entity_type {
            EntityType::Building => self.current_counts.buildings,
            EntityType::Vehicle => self.current_counts.vehicles,
            EntityType::Npc => self.current_counts.npcs,
            EntityType::Tree => self.current_counts.trees,
            EntityType::Particle => self.current_counts.particles,
        }
    }

    /// Get entity cap for biome budget
    fn get_entity_cap(&self, entity_type: EntityType, budget: &EntityBudget) -> u32 {
        match entity_type {
            EntityType::Building => budget.buildings,
            EntityType::Vehicle => budget.vehicles,
            EntityType::Npc => budget.npcs,
            EntityType::Tree => budget.trees,
            EntityType::Particle => budget.particles,
        }
    }

    /// Get current budget utilization percentage
    pub fn get_budget_utilization(&self, biome: BiomeType) -> f32 {
        let budget = self.get_budget_for_biome(biome);
        (self.current_counts.total as f32 / budget.total_cap as f32) * 100.0
    }

    /// Get queue status information
    pub fn get_queue_status(&self) -> (usize, u32, f32) {
        (
            self.spawn_queue.pending_spawns.len(),
            self.spawn_queue.max_queue_length,
            (self.spawn_queue.pending_spawns.len() as f32
                / self.spawn_queue.max_queue_length as f32)
                * 100.0,
        )
    }
}

/// Oracle's Spawn Budget System Plugin
pub struct SpawnBudgetPlugin;

impl Plugin for SpawnBudgetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnBudgetPolicy>().add_systems(
            Update,
            (
                reset_frame_counters.in_set(amp_core::system_ordering::SpawnBudgetSystemSet::Init),
                process_spawn_queue
                    .in_set(amp_core::system_ordering::SpawnBudgetSystemSet::Processing),
                crate::spawn_budget_integration::process_budget_queue_spawns
                    .in_set(amp_core::system_ordering::SpawnBudgetSystemSet::Execution),
                update_budget_metrics
                    .in_set(amp_core::system_ordering::SpawnBudgetSystemSet::Cleanup),
                crate::spawn_budget_integration::track_entity_despawns
                    .in_set(amp_core::system_ordering::SpawnBudgetSystemSet::Cleanup),
            ),
        );
    }
}

/// System to reset frame counters at the start of each frame
fn reset_frame_counters(mut policy: ResMut<SpawnBudgetPolicy>) {
    policy.reset_frame_counters();
}

/// System to process queued spawns within frame budget
fn process_spawn_queue(
    mut policy: ResMut<SpawnBudgetPolicy>,
    time: Res<Time>,
    // TODO: Add Commands parameter for actual entity spawning
) {
    let game_time = time.elapsed_secs();
    let _spawned = policy.process_spawn_queue(game_time);

    // TODO: Actually spawn the entities using Commands
    // This will be implemented in Phase 2: Factory Integration
}

/// System to update budget utilization metrics
fn update_budget_metrics(mut policy: ResMut<SpawnBudgetPolicy>) {
    // Update budget utilization for default biome (will be enhanced in Phase 2)
    policy.metrics.budget_utilization = policy.get_budget_utilization(BiomeType::Urban);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_budget_policy_creation() {
        let policy = SpawnBudgetPolicy::default();
        assert_eq!(policy.current_counts.total, 0);
        assert_eq!(policy.budget_caps.default.buildings, 80);
        assert_eq!(policy.budget_caps.default.vehicles, 20);
        assert_eq!(policy.budget_caps.default.npcs, 2);
    }

    #[test]
    fn test_can_spawn_within_budget() {
        let policy = SpawnBudgetPolicy::default();
        assert!(policy.can_spawn(EntityType::Building, BiomeType::Urban));
        assert!(policy.can_spawn(EntityType::Vehicle, BiomeType::Rural));
    }

    #[test]
    fn test_spawn_request_approval() {
        let mut policy = SpawnBudgetPolicy::default();
        let result = policy.request_spawn(
            EntityType::Building,
            BiomeType::Urban,
            SpawnPriority::Medium,
            SpawnData::Building {
                position: Vec3::ZERO,
                building_type: "house".to_string(),
            },
            0.0,
        );
        assert_eq!(result, SpawnResult::Approved);
        assert_eq!(policy.current_counts.buildings, 1);
    }

    #[test]
    fn test_spawn_queuing_when_budget_exceeded() {
        let mut policy = SpawnBudgetPolicy::default();

        // Fill up building budget for urban biome (120 buildings)
        for _ in 0..120 {
            policy.record_spawn(EntityType::Building);
        }

        let result = policy.request_spawn(
            EntityType::Building,
            BiomeType::Urban,
            SpawnPriority::Low,
            SpawnData::Building {
                position: Vec3::ZERO,
                building_type: "house".to_string(),
            },
            0.0,
        );
        assert_eq!(result, SpawnResult::Queued);
        assert_eq!(policy.spawn_queue.pending_spawns.len(), 1);
    }

    #[test]
    fn test_frame_limit_enforcement() {
        let mut policy = SpawnBudgetPolicy::default();

        // Exhaust frame spawn limit
        for _ in 0..50 {
            policy.spawn_queue.current_frame_spawns += 1;
        }

        let result = policy.request_spawn(
            EntityType::Vehicle,
            BiomeType::Urban,
            SpawnPriority::High,
            SpawnData::Vehicle {
                position: Vec3::ZERO,
                vehicle_type: "car".to_string(),
            },
            0.0,
        );
        assert_eq!(
            result,
            SpawnResult::Rejected(RejectionReason::FrameLimitExceeded)
        );
    }

    #[test]
    fn test_budget_utilization_calculation() {
        let mut policy = SpawnBudgetPolicy::default();

        // Spawn 50% of urban building budget (60 out of 120)
        for _ in 0..60 {
            policy.record_spawn(EntityType::Building);
        }

        let utilization = policy.get_budget_utilization(BiomeType::Urban);
        assert!((utilization - 21.43).abs() < 0.1); // 60/280 total cap ≈ 21.43%
    }
}
