#![deny(missing_docs, rust_2018_idioms, unsafe_code)]

//! Amp Foundation - Unified facade for core Amp functionality
//!
//! This crate provides a single import point for commonly used Amp types and functions.
//! It re-exports carefully selected items from the core Amp crates to provide a clean API.

/// The Amp Foundation prelude - commonly used items for game development
pub mod prelude {
    // Core error handling and results
    pub use amp_core::{Error, Result};

    // Math utilities
    pub use amp_math::bounds::{Aabb, Sphere};
    pub use amp_math::morton::Morton3D;
    pub use amp_math::transforms::Transform as AmpTransform;
    pub use amp_math::Vec3;

    // Configuration utilities
    pub use config_core::{ConfigLoader, FactorySettings, GameConfig};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_facade_imports() {
        // Test that basic types are available
        let _position = Vec3::new(1.0, 2.0, 3.0);
        let _aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);

        // Test error handling
        let result: Result<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_morton_encoding() {
        let morton_code = Morton3D::encode(Vec3::new(1.0, 2.0, 3.0));
        assert!(morton_code > 0);
    }
}
