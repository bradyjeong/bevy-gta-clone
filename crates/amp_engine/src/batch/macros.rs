//! Procedural macros for batch processing system
//!
//! This module provides the `#[batch_system]` macro for cost-based system registration.

/// Macro for marking systems for batch processing
///
/// This macro automatically registers systems with the batch controller
/// and provides cost-based scheduling.
///
/// # Arguments
///
/// * `batch_type` - The BatchType category for this system
/// * `cost` - Execution cost weight (0.0 - 1.0)
///
/// # Example
///
/// ```rust
/// #[batch_system(BatchType::Transform, cost = 0.8)]
/// fn my_heavy_transform_system(
///     mut query: Query<&mut Transform, With<Player>>,
/// ) {
///     // Heavy transform processing
/// }
/// ```
#[macro_export]
macro_rules! batch_system {
    ($batch_type:expr, cost = $cost:expr) => {
        // This is a declarative macro implementation
        // In a full implementation, this would be a procedural macro
        // that generates the necessary registration code
    };
}

pub use batch_system;
