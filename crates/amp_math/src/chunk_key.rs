use crate::morton::Morton2D;
use serde::{Deserialize, Serialize};
use std::fmt;

/// ChunkKey represents a 2D chunk coordinate with Morton encoding for spatial locality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkKey {
    pub x: i32,
    pub z: i32,
}

impl ChunkKey {
    /// Create a new ChunkKey from 2D coordinates
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Create ChunkKey from world position and chunk size
    pub fn from_position(x: f32, z: f32, chunk_size: f32) -> Self {
        let chunk_x = (x / chunk_size).floor() as i32;
        let chunk_z = (z / chunk_size).floor() as i32;
        Self::new(chunk_x, chunk_z)
    }

    /// Get world position of chunk center
    pub fn to_world_position(&self, chunk_size: f32) -> (f32, f32) {
        let x = (self.x as f32 + 0.5) * chunk_size;
        let z = (self.z as f32 + 0.5) * chunk_size;
        (x, z)
    }

    /// Get Morton code for spatial locality
    pub fn morton_code(&self) -> u64 {
        // Convert signed coordinates to unsigned for Morton encoding
        let ux = (self.x as u32).wrapping_add(0x8000_0000);
        let uz = (self.z as u32).wrapping_add(0x8000_0000);
        Morton2D::encode(ux, uz)
    }

    /// Get neighboring chunk keys in 8 directions
    pub fn neighbors(&self) -> [ChunkKey; 8] {
        [
            ChunkKey::new(self.x - 1, self.z - 1), // SW
            ChunkKey::new(self.x, self.z - 1),     // S
            ChunkKey::new(self.x + 1, self.z - 1), // SE
            ChunkKey::new(self.x - 1, self.z),     // W
            ChunkKey::new(self.x + 1, self.z),     // E
            ChunkKey::new(self.x - 1, self.z + 1), // NW
            ChunkKey::new(self.x, self.z + 1),     // N
            ChunkKey::new(self.x + 1, self.z + 1), // NE
        ]
    }

    /// Get Manhattan distance to another chunk
    pub fn manhattan_distance(&self, other: &ChunkKey) -> u32 {
        ((self.x - other.x).abs() + (self.z - other.z).abs()) as u32
    }

    /// Get Euclidean distance squared to another chunk
    pub fn distance_squared(&self, other: &ChunkKey) -> u32 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz) as u32
    }
}

impl fmt::Display for ChunkKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk({}, {})", self.x, self.z)
    }
}

impl From<(i32, i32)> for ChunkKey {
    fn from((x, z): (i32, i32)) -> Self {
        Self::new(x, z)
    }
}

impl From<ChunkKey> for (i32, i32) {
    fn from(key: ChunkKey) -> Self {
        (key.x, key.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_key_creation() {
        let key = ChunkKey::new(5, -3);
        assert_eq!(key.x, 5);
        assert_eq!(key.z, -3);
    }

    #[test]
    fn test_from_position() {
        let chunk_size = 200.0;
        let key = ChunkKey::from_position(450.0, -150.0, chunk_size);
        assert_eq!(key.x, 2);
        assert_eq!(key.z, -1);
    }

    #[test]
    fn test_to_world_position() {
        let key = ChunkKey::new(2, -1);
        let chunk_size = 200.0;
        let (x, z) = key.to_world_position(chunk_size);
        assert_eq!(x, 500.0);
        assert_eq!(z, -100.0);
    }

    #[test]
    fn test_morton_code() {
        let key1 = ChunkKey::new(0, 0);
        let key2 = ChunkKey::new(1, 0);
        let key3 = ChunkKey::new(0, 1);

        let code1 = key1.morton_code();
        let code2 = key2.morton_code();
        let code3 = key3.morton_code();

        // Morton codes should be different for different positions
        assert_ne!(code1, code2);
        assert_ne!(code1, code3);
        assert_ne!(code2, code3);
    }

    #[test]
    fn test_neighbors() {
        let key = ChunkKey::new(0, 0);
        let neighbors = key.neighbors();

        assert_eq!(neighbors.len(), 8);
        assert!(neighbors.contains(&ChunkKey::new(-1, -1)));
        assert!(neighbors.contains(&ChunkKey::new(0, -1)));
        assert!(neighbors.contains(&ChunkKey::new(1, -1)));
        assert!(neighbors.contains(&ChunkKey::new(-1, 0)));
        assert!(neighbors.contains(&ChunkKey::new(1, 0)));
        assert!(neighbors.contains(&ChunkKey::new(-1, 1)));
        assert!(neighbors.contains(&ChunkKey::new(0, 1)));
        assert!(neighbors.contains(&ChunkKey::new(1, 1)));
    }

    #[test]
    fn test_manhattan_distance() {
        let key1 = ChunkKey::new(0, 0);
        let key2 = ChunkKey::new(3, 4);

        assert_eq!(key1.manhattan_distance(&key2), 7);
        assert_eq!(key2.manhattan_distance(&key1), 7);
    }

    #[test]
    fn test_distance_squared() {
        let key1 = ChunkKey::new(0, 0);
        let key2 = ChunkKey::new(3, 4);

        assert_eq!(key1.distance_squared(&key2), 25);
        assert_eq!(key2.distance_squared(&key1), 25);
    }

    #[test]
    fn test_display() {
        let key = ChunkKey::new(5, -3);
        assert_eq!(format!("{key}"), "Chunk(5, -3)");
    }

    #[test]
    fn test_from_tuple() {
        let key: ChunkKey = (5, -3).into();
        assert_eq!(key.x, 5);
        assert_eq!(key.z, -3);
    }

    #[test]
    fn test_to_tuple() {
        let key = ChunkKey::new(5, -3);
        let (x, z): (i32, i32) = key.into();
        assert_eq!(x, 5);
        assert_eq!(z, -3);
    }
}
