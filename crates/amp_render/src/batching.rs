//! Instance batching system for efficient rendering
//!
//! Groups instances by material and mesh for optimal draw calls.

use crate::{Batch, BatchKey, ExtractedInstance};
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource containing all render batches
#[derive(Resource, Default)]
pub struct BatchManager {
    /// Active batches grouped by key
    pub batches: HashMap<BatchKey, Batch>,
    /// Maximum instances per batch
    pub max_instances_per_batch: usize,
}

impl BatchManager {
    /// Create a new batch manager
    pub fn new() -> Self {
        Self {
            batches: HashMap::new(),
            max_instances_per_batch: 1000,
        }
    }

    /// Add an instance to the appropriate batch
    pub fn add_instance(&mut self, instance: &ExtractedInstance) {
        if !instance.visible {
            return;
        }

        let batch = self
            .batches
            .entry(instance.batch_key.clone())
            .or_insert_with(|| Batch::new(instance.batch_key.clone()));

        if batch.len() < self.max_instances_per_batch {
            batch.add_instance(instance.transform);
        }
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        for batch in self.batches.values_mut() {
            batch.clear();
        }
    }

    /// Get the total number of batches
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Get the total number of instances across all batches
    pub fn instance_count(&self) -> usize {
        self.batches.values().map(|b| b.len()).sum()
    }
}

/// System to collect instances into batches
pub fn collect_instances_system(
    mut batch_manager: ResMut<BatchManager>,
    instances: Query<&ExtractedInstance>,
) {
    batch_manager.clear();

    for instance in instances.iter() {
        batch_manager.add_instance(instance);
    }
}

/// Plugin for batching systems
pub struct BatchingSystemPlugin;

impl Plugin for BatchingSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatchManager>()
            .add_systems(PostUpdate, collect_instances_system);
    }
}

/// Re-exports for convenience
pub mod prelude {
    pub use crate::batching::{BatchManager, BatchingSystemPlugin, collect_instances_system};
}
