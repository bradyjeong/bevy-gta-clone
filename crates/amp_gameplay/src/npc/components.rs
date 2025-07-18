//! NPC components for ECS
//!
//! Contains all NPC-related components that define NPC behavior,
//! state management, and AI processing.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Main NPC component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct NPC {
    /// NPC unique identifier
    pub id: u32,
    /// NPC type for different behaviors
    pub npc_type: NpcType,
    /// Movement speed in m/s
    pub speed: f32,
    /// Health points
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Current energy level (0-100)
    pub energy: f32,
    /// Current stress level (0-100)
    pub stress: f32,
    /// Time since last decision
    pub last_decision_time: f32,
    /// Current target entity (if any)
    pub target: Option<Entity>,
    /// Memory of recent events
    pub memory_duration: f32,
}

impl Default for NPC {
    fn default() -> Self {
        Self {
            id: 0,
            npc_type: NpcType::Civilian,
            speed: 1.5,
            health: 100.0,
            max_health: 100.0,
            energy: 100.0,
            stress: 0.0,
            last_decision_time: 0.0,
            target: None,
            memory_duration: 60.0,
        }
    }
}

/// NPC type enum for different behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum NpcType {
    /// Standard civilian NPC
    Civilian,
    /// Police officer NPC
    Police,
    /// Security guard NPC
    Security,
    /// Vendor/shopkeeper NPC
    Vendor,
}

impl Default for NpcType {
    fn default() -> Self {
        Self::Civilian
    }
}

/// NPC finite state machine component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct NpcState {
    /// Current behavior state
    pub current: NpcBehaviorState,
    /// Previous behavior state
    pub previous: NpcBehaviorState,
    /// Time when current state was entered
    pub state_start_time: f32,
    /// Time spent in current state
    pub state_duration: f32,
    /// State-specific data
    pub state_data: StateData,
}

impl Default for NpcState {
    fn default() -> Self {
        Self {
            current: NpcBehaviorState::Idle,
            previous: NpcBehaviorState::Idle,
            state_start_time: 0.0,
            state_duration: 0.0,
            state_data: StateData::default(),
        }
    }
}

/// NPC behavior states for finite state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum NpcBehaviorState {
    /// Idle - standing still, looking around
    Idle,
    /// Wander - moving randomly within area
    Wander,
    /// Flee - running away from danger
    Flee,
    /// Follow - following a target (future expansion)
    Follow,
    /// Interact - interacting with objects/other NPCs (future expansion)
    Interact,
}

impl Default for NpcBehaviorState {
    fn default() -> Self {
        Self::Idle
    }
}

/// State-specific data for NPC behavior
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StateData {
    /// Target position for movement states
    pub target_position: Option<Vec3>,
    /// Direction vector for movement
    pub direction: Vec3,
    /// Speed multiplier for current state
    pub speed_multiplier: f32,
    /// State-specific timer
    pub timer: f32,
    /// Maximum duration for current state
    pub max_duration: f32,
}

impl Default for StateData {
    fn default() -> Self {
        Self {
            target_position: None,
            direction: Vec3::ZERO,
            speed_multiplier: 1.0,
            timer: 0.0,
            max_duration: 10.0,
        }
    }
}

/// NPC brain handle for batch processing
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct NpcBrainHandle {
    /// Handle for batch processing system
    pub batch_handle: u32,
    /// Priority for processing (lower = higher priority)
    pub priority: u32,
    /// Cost for batch processing (0.0-1.0)
    pub cost: f32,
    /// Distance to player for tick rate calculation
    pub distance_to_player: f32,
    /// Frames since last update
    pub frames_since_update: u32,
    /// Required frames between updates based on distance
    pub update_interval: u32,
}

impl Default for NpcBrainHandle {
    fn default() -> Self {
        Self {
            batch_handle: 0,
            priority: 4, // BatchType::AI priority
            cost: 1.0,
            distance_to_player: f32::MAX,
            frames_since_update: 0,
            update_interval: 1, // Every frame by default
        }
    }
}

/// Component to track last update frame for timing
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct LastUpdateFrame {
    /// Frame number when last updated
    pub frame: u64,
    /// Time when last updated
    pub time: f32,
}

impl Default for LastUpdateFrame {
    fn default() -> Self {
        Self {
            frame: 0,
            time: 0.0,
        }
    }
}

/// Metrics component for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Resource)]
pub struct NpcMetrics {
    /// Number of NPCs updated this frame
    pub npcs_updated_this_frame: u32,
    /// Total NPC updates per second
    pub updates_per_second: f32,
    /// Time spent on NPC processing this frame
    pub processing_time_ms: f32,
    /// Average processing time per NPC
    pub avg_processing_time_per_npc: f32,
    /// Total NPCs in world
    pub total_npcs: u32,
    /// NPCs by distance category
    pub npcs_by_distance: [u32; 3], // [close, medium, far]
    /// Frame counter for averaging
    pub frame_counter: u64,
    /// Accumulated update count for averaging
    pub accumulated_updates: u32,
}

impl Default for NpcMetrics {
    fn default() -> Self {
        Self {
            npcs_updated_this_frame: 0,
            updates_per_second: 0.0,
            processing_time_ms: 0.0,
            avg_processing_time_per_npc: 0.0,
            total_npcs: 0,
            npcs_by_distance: [0; 3],
            frame_counter: 0,
            accumulated_updates: 0,
        }
    }
}

/// Bundle for spawning NPCs
#[derive(Bundle, Default)]
pub struct NpcBundle {
    /// NPC component
    pub npc: NPC,
    /// State machine
    pub state: NpcState,
    /// Brain handle for batch processing
    pub brain_handle: NpcBrainHandle,
    /// Update frame tracking
    pub last_update_frame: LastUpdateFrame,
    /// Transform for position
    pub transform: Transform,
    /// Global transform
    pub global_transform: GlobalTransform,
    /// Visibility
    pub visibility: Visibility,
    /// Inherited visibility
    pub inherited_visibility: InheritedVisibility,
    /// View visibility
    pub view_visibility: ViewVisibility,
}
