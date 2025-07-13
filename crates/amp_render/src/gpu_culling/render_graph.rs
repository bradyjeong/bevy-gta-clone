//! Render graph integration for GPU culling
//!
//! Provides the GpuCullNode that integrates GPU culling into Bevy's render graph.
//! This node handles compute shader dispatch for frustum and LOD culling.

use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::ViewTarget;
use bevy::render::{Extract, RenderApp, RenderSet};
use std::borrow::Cow;

use super::{GpuCullingConfig, GpuCullingStats, GpuCullingResources};
use crate::culling::GpuCullBindGroupLayouts;
use crate::render_world::TransientBufferPool;

/// Render graph node for GPU culling compute dispatch
#[derive(Default)]
pub struct GpuCullNode;

impl Node for GpuCullNode {
    fn update(&mut self, world: &mut World) {
        // Update any per-frame culling parameters
        if let Some(mut resources) = world.get_resource_mut::<GpuCullingResources>() {
            // Reset frame statistics
            resources.stats.instances_processed = 0;
            resources.stats.instances_visible = 0;
            resources.stats.gpu_time_ms = 0.0;
            resources.stats.upload_time_ms = 0.0;
            resources.stats.readback_time_ms = 0.0;
        }
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get required resources
        let Some(culling_resources) = world.get_resource::<GpuCullingResources>() else {
            warn!("GpuCullingResources not found, skipping GPU culling");
            return Ok(());
        };

        let Some(pipeline) = &culling_resources.pipeline else {
            warn!("GPU culling pipeline not initialized, skipping");
            return Ok(());
        };

        let Some(bind_group_layouts) = world.get_resource::<GpuCullBindGroupLayouts>() else {
            warn!("GpuCullBindGroupLayouts not found, skipping GPU culling");
            return Ok(());
        };

        let Some(mut buffer_pool) = world.get_resource_mut::<TransientBufferPool>() else {
            warn!("TransientBufferPool not found, skipping GPU culling");
            return Ok(());
        };

        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // For now, create a dummy instance count
        // In a real implementation, this would come from the extracted instances
        let instance_count = 1000u32;

        if instance_count == 0 {
            return Ok(());
        }

        // Create buffers for GPU culling
        let camera_buffer = buffer_pool.get_buffer(
            std::mem::size_of::<super::compute::GpuCameraData>() as u64,
            render_device,
        );

        let instance_buffer = buffer_pool.get_buffer(
            (instance_count as u64) * std::mem::size_of::<super::compute::GpuInstanceData>() as u64,
            render_device,
        );

        let params_buffer = buffer_pool.get_buffer(
            std::mem::size_of::<super::compute::GpuCullingParams>() as u64,
            render_device,
        );

        // Output buffers
        let visibility_buffer = buffer_pool.get_buffer(
            ((instance_count + 31) / 32) as u64 * 4, // Bitset: 32 instances per u32
            render_device,
        );

        let count_buffer = buffer_pool.get_buffer(
            4, // Single u32 for visible count
            render_device,
        );

        // Create bind group using the pipeline's bind group layout
        let bind_group = render_device.create_bind_group(
            "gpu_culling_bind_group",
            pipeline.bind_group_layout(),
            &BindGroupEntries::sequential((
                camera_buffer.as_entire_binding(),
                instance_buffer.as_entire_binding(),
                params_buffer.as_entire_binding(),
                visibility_buffer.as_entire_binding(),
                count_buffer.as_entire_binding(),
            )),
        );

        // TODO: Upload actual camera, instance, and parameter data
        // For now, we'll just dispatch the compute shader with dummy data

        // Dispatch GPU culling compute shader
        let mut command_encoder = render_context.command_encoder();
        
        if let Err(e) = pipeline.dispatch_culling(
            &mut command_encoder,
            &bind_group,
            instance_count,
            pipeline_cache,
        ) {
            warn!("GPU culling dispatch failed: {}", e);
            return Ok(());
        }

        // In a real implementation, we would:
        // 1. Read back the visibility buffer
        // 2. Update the instance visibility flags
        // 3. Update statistics

        info!("GPU culling dispatch completed for {} instances", instance_count);

        Ok(())
    }
}

/// Plugin for GPU culling render graph integration
pub struct GpuCullingPipelinePlugin;

impl Plugin for GpuCullingPipelinePlugin {
    fn build(&self, app: &mut App) {
        info!("GPU culling pipeline plugin enabled");

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("RenderApp not found, cannot add GPU culling pipeline");
            return;
        };

        // Add the GPU culling node to the render graph
        // Insert before the main 3D render pass
        use bevy::render::graph::{RenderGraph, Node3d, Core3d};
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(Core3d, GpuCullingLabel, GpuCullNode);
        render_graph.add_node_edge(Core3d, Node3d::StartMainPass, GpuCullingLabel);
        render_graph.add_node_edge(Core3d, GpuCullingLabel, Node3d::MainOpaquePass);
    }
}

/// Label for the GPU culling render graph node
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct GpuCullingLabel;

/// System to check if GPU culling pipeline is available
pub fn is_gpu_culling_pipeline_available(world: &World) -> bool {
    world.get_resource::<GpuCullingResources>()
        .and_then(|res| res.pipeline.as_ref())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::RenderPlugin;

    #[test]
    fn test_gpu_culling_pipeline_plugin() {
        let mut app = App::new();
        app.add_plugins((
            bevy::log::LogPlugin::default(),
            bevy::core::TaskPoolPlugin::default(),
            bevy::core::TypeRegistrationPlugin,
            bevy::core::FrameCountPlugin,
            bevy::time::TimePlugin,
            bevy::transform::TransformPlugin,
            bevy::hierarchy::HierarchyPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin::default(),
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            RenderPlugin::default(),
        ));

        // Add GPU culling pipeline plugin
        app.add_plugins(GpuCullingPipelinePlugin);

        // Plugin should add without panicking
        assert!(app.world().contains_resource::<App>());
    }

    #[test]
    fn test_gpu_culling_label() {
        let label1 = GpuCullingLabel;
        let label2 = GpuCullingLabel;
        assert_eq!(label1, label2);
    }

    #[test]
    fn test_is_gpu_culling_pipeline_available() {
        let mut world = World::new();
        
        // Should return false when no resources present
        assert!(!is_gpu_culling_pipeline_available(&world));

        // Should return false when pipeline is None
        world.insert_resource(GpuCullingResources::default());
        assert!(!is_gpu_culling_pipeline_available(&world));

        // Should return true when pipeline is present
        // (This would require a real pipeline, so we'll test the resource check)
        let resources = world.get_resource::<GpuCullingResources>().unwrap();
        assert!(resources.pipeline.is_none());
    }
}
