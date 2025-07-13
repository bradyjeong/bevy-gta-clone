//! Test utilities for bind group layout validation

use bevy::render::render_resource::*;
use serde::{Deserialize, Serialize};

/// Serializable representation of a BindGroupLayout for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableBindGroupLayout {
    pub label: Option<String>,
    pub entries: Vec<SerializableBindGroupLayoutEntry>,
}

/// Serializable representation of a BindGroupLayoutEntry for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: u32, // ShaderStages as u32
    pub ty: SerializableBindingType,
    pub count: Option<u32>,
}

/// Serializable representation of a BindingType for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SerializableBindingType {
    Buffer {
        ty: SerializableBufferBindingType,
        has_dynamic_offset: bool,
        min_binding_size: Option<u64>,
    },
    Sampler {
        filtering: bool,
        comparison: bool,
    },
    Texture {
        multisampled: bool,
        view_dimension: String,
        sample_type: String,
    },
    StorageTexture {
        access: String,
        format: String,
        view_dimension: String,
    },
    AccelerationStructure,
}

/// Serializable representation of a BufferBindingType for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SerializableBufferBindingType {
    Uniform,
    Storage { read_only: bool },
}

impl From<&BindGroupLayoutDescriptor<'_>> for SerializableBindGroupLayout {
    fn from(desc: &BindGroupLayoutDescriptor<'_>) -> Self {
        Self {
            label: desc.label.map(|s| s.to_string()),
            entries: desc.entries.iter().map(|e| e.into()).collect(),
        }
    }
}

impl From<&BindGroupLayoutEntry> for SerializableBindGroupLayoutEntry {
    fn from(entry: &BindGroupLayoutEntry) -> Self {
        Self {
            binding: entry.binding,
            visibility: entry.visibility.bits(),
            ty: (&entry.ty).into(),
            count: entry.count.map(|c| c.get()),
        }
    }
}

impl From<&BindingType> for SerializableBindingType {
    fn from(ty: &BindingType) -> Self {
        match ty {
            BindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size,
            } => Self::Buffer {
                ty: ty.into(),
                has_dynamic_offset: *has_dynamic_offset,
                min_binding_size: min_binding_size.map(|s| s.get()),
            },
            BindingType::Sampler(sampler_binding_type) => Self::Sampler {
                filtering: matches!(sampler_binding_type, SamplerBindingType::Filtering),
                comparison: matches!(sampler_binding_type, SamplerBindingType::Comparison),
            },
            BindingType::Texture {
                multisampled,
                view_dimension,
                sample_type,
            } => Self::Texture {
                multisampled: *multisampled,
                view_dimension: format!("{:?}", view_dimension),
                sample_type: format!("{:?}", sample_type),
            },
            BindingType::StorageTexture {
                access,
                format,
                view_dimension,
            } => Self::StorageTexture {
                access: format!("{:?}", access),
                format: format!("{:?}", format),
                view_dimension: format!("{:?}", view_dimension),
            },
            BindingType::AccelerationStructure => Self::AccelerationStructure,
        }
    }
}

impl From<&BufferBindingType> for SerializableBufferBindingType {
    fn from(ty: &BufferBindingType) -> Self {
        match ty {
            BufferBindingType::Uniform => Self::Uniform,
            BufferBindingType::Storage { read_only } => Self::Storage {
                read_only: *read_only,
            },
        }
    }
}

/// Macro to assert that a bind group layout matches a golden JSON representation
#[macro_export]
macro_rules! assert_layout_matches {
    ($layout_desc:expr, $golden_json:expr) => {
        let actual = $crate::tests::bind_group_layout_tests::SerializableBindGroupLayout::from(
            &$layout_desc,
        );
        let expected: $crate::tests::bind_group_layout_tests::SerializableBindGroupLayout =
            serde_json::from_str($golden_json).expect("Failed to deserialize golden JSON");
        assert_eq!(
            actual, expected,
            "Bind group layout does not match golden JSON"
        );
    };
}

#[cfg(test)]
mod layout_validation_tests {
    use super::*;

    #[test]
    fn test_serializable_bind_group_layout_creation() {
        let desc = BindGroupLayoutDescriptor {
            label: Some("Test Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

        let serializable = SerializableBindGroupLayout::from(&desc);

        assert_eq!(serializable.label, Some("Test Layout".to_string()));
        assert_eq!(serializable.entries.len(), 1);
        assert_eq!(serializable.entries[0].binding, 0);
        assert_eq!(
            serializable.entries[0].visibility,
            ShaderStages::COMPUTE.bits()
        );

        match &serializable.entries[0].ty {
            SerializableBindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size,
            } => {
                assert_eq!(
                    ty,
                    &SerializableBufferBindingType::Storage { read_only: true }
                );
                assert!(!has_dynamic_offset);
                assert_eq!(min_binding_size, &None);
            }
            _ => panic!("Expected Buffer binding type"),
        }
    }

    #[test]
    fn test_assert_layout_matches_macro() {
        let desc = BindGroupLayoutDescriptor {
            label: Some("Test Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

        let golden_json = r#"
        {
            "label": "Test Layout",
            "entries": [
                {
                    "binding": 0,
                    "visibility": 4,
                    "ty": {
                        "Buffer": {
                            "ty": "Uniform",
                            "has_dynamic_offset": false,
                            "min_binding_size": null
                        }
                    },
                    "count": null
                }
            ]
        }
        "#;

        assert_layout_matches!(desc, golden_json);
    }
}
