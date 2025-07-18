//! High-performance math library for spatial calculations and Morton encoding.
//!
//! This crate provides efficient implementations for:
//! - Morton encoding/decoding for spatial indexing
//! - Axis-aligned bounding boxes (AABB) and spheres
//! - Transform utilities wrapping glam
//!
//! # Examples
//!
//! ```rust
//! use amp_math::morton::Morton3D;
//! use glam::Vec3;
//!
//! let pos = Vec3::new(1.0, 2.0, 3.0);
//! let morton = Morton3D::encode(pos);
//! let decoded = Morton3D::decode(morton);
//! ```

pub mod bounds;
pub mod chunk_key;
pub mod coordinate_conversion;
pub mod morton;
pub mod spatial;
pub mod transforms;

pub use glam::*;
