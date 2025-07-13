//! Tests for the amp_render crate

#[cfg(feature = "gpu")]
pub mod gpu_culling_simple_tests;

// TODO: Fix GPU culling tests
// #[cfg(feature = "gpu_culling")]
// pub mod gpu_culling_tests;

pub mod bind_group_layout_tests;
pub mod memory_leak_gpu_buffers;
