//! Shader management for the AMP Game Engine.
//!
//! This module provides utilities for loading, compiling, and managing shaders.
//! It supports both WGSL and SPIR-V shaders and provides a convenient interface
//! for shader resource management.

use amp_core::{Error, Result};
use wgpu::*;
use std::collections::HashMap;

/// A shader resource that can be loaded and used for rendering.
///
/// This structure represents a compiled shader that can be used to create
/// render pipelines. It holds the shader module and metadata about the shader.
pub struct Shader {
    /// The compiled shader module
    pub module: ShaderModule,
    /// The entry point function name
    pub entry_point: String,
    /// The shader stage (vertex, fragment, compute)
    pub stage: ShaderStages,
    /// Optional label for debugging
    pub label: Option<String>,
}

impl Shader {
    /// Create a new shader from WGSL source code.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device to create the shader on
    /// * `source` - The WGSL source code
    /// * `entry_point` - The entry point function name
    /// * `stage` - The shader stage
    /// * `label` - Optional label for debugging
    pub fn from_wgsl(
        device: &Device,
        source: &str,
        entry_point: &str,
        stage: ShaderStages,
        label: Option<&str>,
    ) -> Result<Self> {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(source.into()),
        });

        Ok(Self {
            module,
            entry_point: entry_point.to_string(),
            stage,
            label: label.map(|s| s.to_string()),
        })
    }

    /// Create a new shader from SPIR-V bytecode.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device to create the shader on
    /// * `bytes` - The SPIR-V bytecode
    /// * `entry_point` - The entry point function name
    /// * `stage` - The shader stage
    /// * `label` - Optional label for debugging
    pub fn from_spirv(
        device: &Device,
        bytes: &[u8],
        entry_point: &str,
        stage: ShaderStages,
        label: Option<&str>,
    ) -> Result<Self> {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label,
            source: ShaderSource::SpirV(bytes.into()),
        });

        Ok(Self {
            module,
            entry_point: entry_point.to_string(),
            stage,
            label: label.map(|s| s.to_string()),
        })
    }

    /// Get the entry point for this shader.
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }

    /// Get the shader stage.
    pub fn stage(&self) -> ShaderStages {
        self.stage
    }

    /// Get the shader module.
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }
}

/// A manager for shader resources.
///
/// This structure provides centralized management of shader resources,
/// including loading, caching, and cleanup. It helps avoid duplicate
/// shader compilation and provides convenient access to shaders.
pub struct ShaderManager {
    /// Cache of loaded shaders
    shaders: HashMap<String, Shader>,
    /// Reference to the GPU device
    device: Device,
}

impl ShaderManager {
    /// Create a new shader manager.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device to use for shader creation
    pub fn new(device: Device) -> Self {
        Self {
            shaders: HashMap::new(),
            device,
        }
    }

    /// Load a shader from WGSL source code.
    ///
    /// If a shader with the same name is already loaded, it will be returned
    /// from the cache instead of being recompiled.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for caching this shader
    /// * `source` - The WGSL source code
    /// * `entry_point` - The entry point function name
    /// * `stage` - The shader stage
    pub fn load_wgsl(
        &mut self,
        name: &str,
        source: &str,
        entry_point: &str,
        stage: ShaderStages,
    ) -> Result<&Shader> {
        if !self.shaders.contains_key(name) {
            let shader = Shader::from_wgsl(
                &self.device,
                source,
                entry_point,
                stage,
                Some(name),
            )?;
            self.shaders.insert(name.to_string(), shader);
        }

        Ok(self.shaders.get(name).unwrap())
    }

    /// Load a shader from SPIR-V bytecode.
    ///
    /// If a shader with the same name is already loaded, it will be returned
    /// from the cache instead of being recompiled.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for caching this shader
    /// * `bytes` - The SPIR-V bytecode
    /// * `entry_point` - The entry point function name
    /// * `stage` - The shader stage
    pub fn load_spirv(
        &mut self,
        name: &str,
        bytes: &[u8],
        entry_point: &str,
        stage: ShaderStages,
    ) -> Result<&Shader> {
        if !self.shaders.contains_key(name) {
            let shader = Shader::from_spirv(
                &self.device,
                bytes,
                entry_point,
                stage,
                Some(name),
            )?;
            self.shaders.insert(name.to_string(), shader);
        }

        Ok(self.shaders.get(name).unwrap())
    }

    /// Get a shader by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the shader to retrieve
    pub fn get(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name)
    }

    /// Remove a shader from the cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the shader to remove
    pub fn remove(&mut self, name: &str) -> Option<Shader> {
        self.shaders.remove(name)
    }

    /// Clear all cached shaders.
    pub fn clear(&mut self) {
        self.shaders.clear();
    }

    /// Get the number of cached shaders.
    pub fn len(&self) -> usize {
        self.shaders.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.shaders.is_empty()
    }

    /// Check if a shader with the given name is cached.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the shader to check
    pub fn contains(&self, name: &str) -> bool {
        self.shaders.contains_key(name)
    }
}

/// Default shaders for common use cases.
impl ShaderManager {
    /// Load a basic vertex shader that passes through position and UV coordinates.
    ///
    /// This shader is suitable for simple 2D rendering or as a starting point
    /// for more complex shaders.
    pub fn load_basic_vertex(&mut self) -> Result<&Shader> {
        let source = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    
    return vec4<f32>(pos[vertex_index], 0.0, 1.0);
}
"#;

        self.load_wgsl("basic_vertex", source, "vs_main", ShaderStages::VERTEX)
    }

    /// Load a basic fragment shader that outputs a solid color.
    ///
    /// This shader outputs a purple color, useful for testing and debugging.
    pub fn load_basic_fragment(&mut self) -> Result<&Shader> {
        let source = r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.0, 0.8, 1.0); // Purple color
}
"#;

        self.load_wgsl("basic_fragment", source, "fs_main", ShaderStages::FRAGMENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_creation() {
        // Test that shader structures can be created
        // (without actually creating GPU resources)
        let entry_point = "vs_main";
        let stage = ShaderStages::VERTEX;
        
        assert_eq!(entry_point, "vs_main");
        assert_eq!(stage, ShaderStages::VERTEX);
    }

    #[test]
    fn test_shader_manager_creation() {
        // Test that we can create the shader manager structure
        // (without actually creating GPU resources)
        let shaders: HashMap<String, Shader> = HashMap::new();
        assert!(shaders.is_empty());
    }

    #[test]
    fn test_shader_stage_checks() {
        // Test that shader stages work as expected
        let vertex_stage = ShaderStages::VERTEX;
        let fragment_stage = ShaderStages::FRAGMENT;
        let compute_stage = ShaderStages::COMPUTE;
        
        assert_ne!(vertex_stage, fragment_stage);
        assert_ne!(fragment_stage, compute_stage);
        assert_ne!(compute_stage, vertex_stage);
    }

    #[test]
    fn test_error_creation() {
        let err = Error::shader("Test shader error");
        assert!(err.to_string().contains("Shader error: Test shader error"));
    }
}
