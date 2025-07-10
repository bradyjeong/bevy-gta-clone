//! Async trait for streaming region data
//!
//! This module provides the RegionProvider trait for asynchronous streaming
//! of spatial region data, enabling efficient loading and unloading of
//! world content based on player position and LOD requirements.

use crate::spatial::region::{Region, RegionId};
use amp_core::Error;
use async_trait::async_trait;
use glam::Vec2;
use std::collections::HashMap;

/// Result type for region operations
pub type RegionResult<T> = Result<T, Error>;

/// Async trait for streaming region data
///
/// Implementors of this trait provide asynchronous access to spatial region data,
/// enabling efficient streaming of world content based on player position and
/// level-of-detail requirements.
#[async_trait]
pub trait RegionProvider: Send + Sync {
    /// The type of data associated with each region
    type Data: Send + Sync;

    /// Load data for a specific region
    ///
    /// This method should asynchronously load the data for the specified region.
    /// The implementation should handle caching and resource management internally.
    ///
    /// # Arguments
    /// * `region_id` - The unique identifier for the region to load
    ///
    /// # Returns
    /// * `Ok(data)` - The loaded region data
    /// * `Err(error)` - An error if the region could not be loaded
    async fn load_region(&self, region_id: RegionId) -> RegionResult<Self::Data>;

    /// Unload data for a specific region
    ///
    /// This method should asynchronously unload the data for the specified region,
    /// freeing up resources and memory.
    ///
    /// # Arguments
    /// * `region_id` - The unique identifier for the region to unload
    ///
    /// # Returns
    /// * `Ok(())` - If the region was successfully unloaded
    /// * `Err(error)` - An error if the region could not be unloaded
    async fn unload_region(&self, region_id: RegionId) -> RegionResult<()>;

    /// Check if a region is currently loaded
    ///
    /// # Arguments
    /// * `region_id` - The unique identifier for the region to check
    ///
    /// # Returns
    /// * `true` if the region is loaded, `false` otherwise
    async fn is_region_loaded(&self, region_id: RegionId) -> bool;

    /// Get all currently loaded regions
    ///
    /// # Returns
    /// * A vector of all currently loaded region IDs
    async fn get_loaded_regions(&self) -> Vec<RegionId>;

    /// Prefetch regions around a given position
    ///
    /// This method should asynchronously prefetch region data around the specified
    /// world position, preparing them for potential future access.
    ///
    /// # Arguments
    /// * `center` - The world position to prefetch around
    /// * `radius` - The radius in world units to prefetch
    /// * `level` - The LOD level to prefetch at
    ///
    /// # Returns
    /// * `Ok(())` - If prefetching was initiated successfully
    /// * `Err(error)` - An error if prefetching could not be initiated
    async fn prefetch_around(&self, center: Vec2, radius: f32, level: u8) -> RegionResult<()>;

    /// Update streaming based on player position and view distance
    ///
    /// This method should update the streaming system based on the current player
    /// position and view distance, loading nearby regions and unloading distant ones.
    ///
    /// # Arguments
    /// * `player_pos` - The current player position in world coordinates
    /// * `view_distance` - The maximum distance for loading regions
    ///
    /// # Returns
    /// * `Ok(())` - If streaming was updated successfully
    /// * `Err(error)` - An error if streaming could not be updated
    async fn update_streaming(&self, player_pos: Vec2, view_distance: f32) -> RegionResult<()>;
}

/// In-memory implementation of RegionProvider for testing and prototyping
///
/// This implementation stores all region data in memory and provides
/// immediate access without actual streaming behavior.
#[derive(Debug)]
pub struct MemoryRegionProvider<T> {
    regions: HashMap<RegionId, T>,
    loaded_regions: HashMap<RegionId, T>,
}

impl<T> MemoryRegionProvider<T> {
    /// Create a new empty memory region provider
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            loaded_regions: HashMap::new(),
        }
    }

    /// Insert region data into the provider
    pub fn insert_region(&mut self, region_id: RegionId, data: T) {
        self.regions.insert(region_id, data);
    }

    /// Get the number of regions stored in the provider
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Get the number of currently loaded regions
    pub fn loaded_count(&self) -> usize {
        self.loaded_regions.len()
    }
}

impl<T> Default for MemoryRegionProvider<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> RegionProvider for MemoryRegionProvider<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Data = T;

    async fn load_region(&self, region_id: RegionId) -> RegionResult<Self::Data> {
        self.regions
            .get(&region_id)
            .cloned()
            .ok_or_else(|| Error::internal(format!("Region {region_id} not found")))
    }

    async fn unload_region(&self, _region_id: RegionId) -> RegionResult<()> {
        // In memory implementation doesn't need to unload
        Ok(())
    }

    async fn is_region_loaded(&self, region_id: RegionId) -> bool {
        self.regions.contains_key(&region_id)
    }

    async fn get_loaded_regions(&self) -> Vec<RegionId> {
        self.regions.keys().copied().collect()
    }

    async fn prefetch_around(&self, _center: Vec2, _radius: f32, _level: u8) -> RegionResult<()> {
        // Memory implementation doesn't need prefetching
        Ok(())
    }

    async fn update_streaming(&self, _player_pos: Vec2, _view_distance: f32) -> RegionResult<()> {
        // Memory implementation doesn't need streaming updates
        Ok(())
    }
}

/// File-based implementation of RegionProvider for persistent storage
///
/// This implementation loads region data from the file system,
/// providing a realistic streaming behavior for development and testing.
#[derive(Debug)]
pub struct FileRegionProvider {
    base_path: std::path::PathBuf,
    loaded_regions: std::sync::Arc<tokio::sync::RwLock<HashMap<RegionId, Vec<u8>>>>,
}

impl FileRegionProvider {
    /// Create a new file-based region provider
    ///
    /// # Arguments
    /// * `base_path` - The base directory path where region files are stored
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            loaded_regions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get the file path for a region
    fn get_region_path(&self, region_id: RegionId) -> std::path::PathBuf {
        let (x, y) = region_id.to_coords();
        self.base_path.join(format!("region_{x}_{y}.bin"))
    }
}

#[async_trait]
impl RegionProvider for FileRegionProvider {
    type Data = Vec<u8>;

    async fn load_region(&self, region_id: RegionId) -> RegionResult<Self::Data> {
        let path = self.get_region_path(region_id);

        match tokio::fs::read(&path).await {
            Ok(data) => {
                let mut loaded = self.loaded_regions.write().await;
                loaded.insert(region_id, data.clone());
                Ok(data)
            }
            Err(e) => Err(Error::internal(format!(
                "Failed to load region {region_id}: {e}"
            ))),
        }
    }

    async fn unload_region(&self, region_id: RegionId) -> RegionResult<()> {
        let mut loaded = self.loaded_regions.write().await;
        loaded.remove(&region_id);
        Ok(())
    }

    async fn is_region_loaded(&self, region_id: RegionId) -> bool {
        let loaded = self.loaded_regions.read().await;
        loaded.contains_key(&region_id)
    }

    async fn get_loaded_regions(&self) -> Vec<RegionId> {
        let loaded = self.loaded_regions.read().await;
        loaded.keys().copied().collect()
    }

    async fn prefetch_around(&self, center: Vec2, radius: f32, level: u8) -> RegionResult<()> {
        // Calculate which regions to prefetch
        let region_size = 1000.0; // TODO: Make configurable
        let area = crate::spatial::region::RegionBounds::new(
            center - Vec2::splat(radius),
            center + Vec2::splat(radius),
        );

        let regions = Region::get_regions_in_area(&area, level, region_size);

        // Spawn tasks to prefetch each region
        let tasks: Vec<_> = regions
            .into_iter()
            .map(|region| {
                let provider = self.clone();
                tokio::spawn(async move {
                    let _ = provider.load_region(region.id).await;
                })
            })
            .collect();

        // Wait for all prefetch tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        Ok(())
    }

    async fn update_streaming(&self, player_pos: Vec2, view_distance: f32) -> RegionResult<()> {
        // Determine which regions should be loaded based on player position
        let region_size = 1000.0; // TODO: Make configurable
        let area = crate::spatial::region::RegionBounds::new(
            player_pos - Vec2::splat(view_distance),
            player_pos + Vec2::splat(view_distance),
        );

        let required_regions: std::collections::HashSet<RegionId> =
            Region::get_regions_in_area(&area, 0, region_size)
                .into_iter()
                .map(|r| r.id)
                .collect();

        let currently_loaded = self.get_loaded_regions().await;
        let currently_loaded_set: std::collections::HashSet<RegionId> =
            currently_loaded.into_iter().collect();

        // Unload regions that are no longer needed
        for region_id in currently_loaded_set.difference(&required_regions) {
            let _ = self.unload_region(*region_id).await;
        }

        // Load regions that are now needed
        for region_id in required_regions.difference(&currently_loaded_set) {
            let _ = self.load_region(*region_id).await;
        }

        Ok(())
    }
}

// Clone implementation for FileRegionProvider
impl Clone for FileRegionProvider {
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            loaded_regions: self.loaded_regions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_provider_basic_operations() {
        let mut provider = MemoryRegionProvider::new();
        let region_id = RegionId::from_coords(1, 1);
        let data = "test data".to_string();

        provider.insert_region(region_id, data.clone());

        // Test loading
        let loaded = provider.load_region(region_id).await.unwrap();
        assert_eq!(loaded, data);

        // Test is_loaded
        assert!(provider.is_region_loaded(region_id).await);

        // Test get_loaded_regions
        let loaded_regions = provider.get_loaded_regions().await;
        assert_eq!(loaded_regions.len(), 1);
        assert_eq!(loaded_regions[0], region_id);

        // Test unload
        provider.unload_region(region_id).await.unwrap();

        // Test streaming operations
        provider
            .prefetch_around(Vec2::ZERO, 100.0, 0)
            .await
            .unwrap();
        provider.update_streaming(Vec2::ZERO, 100.0).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_provider_error_handling() {
        let provider = MemoryRegionProvider::<String>::new();
        let region_id = RegionId::from_coords(999, 999);

        // Test loading non-existent region
        let result = provider.load_region(region_id).await;
        assert!(result.is_err());

        // Test is_loaded for non-existent region
        assert!(!provider.is_region_loaded(region_id).await);
    }

    #[tokio::test]
    async fn test_file_provider_path_generation() {
        let provider = FileRegionProvider::new("/tmp/regions");
        let region_id = RegionId::from_coords(42, 84);

        let path = provider.get_region_path(region_id);
        assert_eq!(path.to_str().unwrap(), "/tmp/regions/region_42_84.bin");
    }

    #[tokio::test]
    async fn test_file_provider_basic_operations() {
        let provider = FileRegionProvider::new("/tmp/test_regions");
        let region_id = RegionId::from_coords(1, 1);

        // Test is_loaded for non-existent region
        assert!(!provider.is_region_loaded(region_id).await);

        // Test get_loaded_regions when empty
        let loaded_regions = provider.get_loaded_regions().await;
        assert_eq!(loaded_regions.len(), 0);

        // Test streaming operations
        provider
            .prefetch_around(Vec2::ZERO, 100.0, 0)
            .await
            .unwrap();
        provider.update_streaming(Vec2::ZERO, 100.0).await.unwrap();
    }
}
