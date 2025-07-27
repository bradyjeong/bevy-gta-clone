//! # AMP Core
//!
//! This crate provides core error handling and utilities for the AMP Game Engine.
//! It defines the primary error types and result aliases used throughout the engine.

pub mod allocation_tracking;

#[cfg(feature = "unstable_advanced_input")]
pub mod input;

#[cfg(feature = "bevy")]
pub mod system_ordering;
#[cfg(all(test, feature = "bevy"))]
mod system_ordering_tests;

/// A specialized `Result` type for operations that may fail within the AMP engine.
///
/// This type is used as the return type for functions that may encounter errors
/// during execution. It provides a convenient way to handle both successful
/// results and error conditions.
pub type Result<T> = std::result::Result<T, Error>;

/// The main error type for the AMP Game Engine.
///
/// This enum represents all possible errors that can occur within the engine.
/// It uses `thiserror` for automatic error trait implementations and provides
/// structured error handling across all engine components.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O related errors (file operations, network, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Error message
        message: String,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
    },

    /// Resource loading errors
    #[error("Failed to load resource '{resource}': {reason}")]
    ResourceLoad {
        /// Resource name
        resource: String,
        /// Failure reason
        reason: String,
    },

    /// Validation errors
    #[error("Validation failed: {message}")]
    Validation {
        /// Error message
        message: String,
    },

    /// Invalid state errors
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Error message
        message: String,
    },

    /// GPU-related errors
    #[error("GPU error: {message}")]
    Gpu {
        /// Error message
        message: String,
    },

    /// Configuration-specific errors
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// Generic error for cases not covered by specific variants
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
    },
}

/// Configuration-specific error types for the config_core crate
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        /// Path to the missing file
        path: String,
    },

    /// Configuration parsing error
    #[error("Failed to parse configuration: {message}")]
    ParseError {
        /// Error message
        message: String,
    },

    /// Invalid configuration format
    #[error("Invalid configuration format: {message}")]
    InvalidFormat {
        /// Error message
        message: String,
    },

    /// I/O error during configuration operations
    #[error("I/O error in configuration: {0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new resource loading error
    pub fn resource_load<R: Into<String>, S: Into<String>>(resource: R, reason: S) -> Self {
        Self::ResourceLoad {
            resource: resource.into(),
            reason: reason.into(),
        }
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state<S: Into<String>>(message: S) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a new GPU error
    pub fn gpu<S: Into<String>>(message: S) -> Self {
        Self::Gpu {
            message: message.into(),
        }
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

impl ConfigError {
    /// Create a new file not found error
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a new parse error
    pub fn parse_error<S: Into<String>>(message: S) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create a new invalid format error
    pub fn invalid_format<S: Into<String>>(message: S) -> Self {
        Self::InvalidFormat {
            message: message.into(),
        }
    }
}

// DEPRECATED: Use amp_foundation::prelude instead
#[deprecated(note = "Use amp_foundation::prelude instead")]
pub mod prelude {
    pub use crate::{ConfigError, Error, Result};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::configuration("Invalid config value");
        assert!(matches!(err, Error::Configuration { .. }));
        assert_eq!(err.to_string(), "Configuration error: Invalid config value");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_serialization_error() {
        let err = Error::serialization("Failed to parse JSON");
        assert!(matches!(err, Error::Serialization { .. }));
        assert_eq!(err.to_string(), "Serialization error: Failed to parse JSON");
    }

    #[test]
    fn test_resource_load_error() {
        let err = Error::resource_load("texture.png", "File corrupted");
        assert!(matches!(err, Error::ResourceLoad { .. }));
        assert_eq!(
            err.to_string(),
            "Failed to load resource 'texture.png': File corrupted"
        );
    }

    #[test]
    fn test_validation_error() {
        let err = Error::validation("Input value out of range");
        assert!(matches!(err, Error::Validation { .. }));
        assert_eq!(
            err.to_string(),
            "Validation failed: Input value out of range"
        );
    }

    #[test]
    fn test_invalid_state_error() {
        let err = Error::invalid_state("Component not initialized");
        assert!(matches!(err, Error::InvalidState { .. }));
        assert_eq!(err.to_string(), "Invalid state: Component not initialized");
    }

    #[test]
    fn test_gpu_error() {
        let err = Error::gpu("GPU device lost");
        assert!(matches!(err, Error::Gpu { .. }));
        assert_eq!(err.to_string(), "GPU error: GPU device lost");
    }

    #[test]
    fn test_internal_error() {
        let err = Error::internal("Unexpected condition");
        assert!(matches!(err, Error::Internal { .. }));
        assert_eq!(err.to_string(), "Internal error: Unexpected condition");
    }

    #[test]
    fn test_result_alias() {
        fn test_function() -> Result<i32> {
            Ok(42)
        }

        fn test_function_error() -> Result<i32> {
            Err(Error::internal("Test error"))
        }

        assert_eq!(test_function().unwrap(), 42);
        assert!(test_function_error().is_err());
    }

    #[test]
    fn test_error_debug() {
        let err = Error::configuration("Test config error");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Configuration"));
        assert!(debug_str.contains("Test config error"));
    }

    #[test]
    fn test_error_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let err = Error::from(io_err);

        let error_chain = format!("{err}");
        assert!(error_chain.contains("I/O error"));
        assert!(error_chain.contains("Access denied"));
    }

    // ConfigError tests
    #[test]
    fn test_config_error_file_not_found() {
        let err = ConfigError::file_not_found("config.ron");
        assert!(matches!(err, ConfigError::FileNotFound { .. }));
        assert_eq!(err.to_string(), "Configuration file not found: config.ron");
    }

    #[test]
    fn test_config_error_parse_error() {
        let err = ConfigError::parse_error("Invalid RON syntax");
        assert!(matches!(err, ConfigError::ParseError { .. }));
        assert_eq!(
            err.to_string(),
            "Failed to parse configuration: Invalid RON syntax"
        );
    }

    #[test]
    fn test_config_error_invalid_format() {
        let err = ConfigError::invalid_format("Missing required field");
        assert!(matches!(err, ConfigError::InvalidFormat { .. }));
        assert_eq!(
            err.to_string(),
            "Invalid configuration format: Missing required field"
        );
    }

    #[test]
    fn test_config_error_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err = ConfigError::from(io_err);
        assert!(matches!(err, ConfigError::IoError(_)));
        assert!(err.to_string().contains("I/O error in configuration"));
    }

    #[test]
    fn test_config_error_into_main_error() {
        let config_err = ConfigError::parse_error("Invalid syntax");
        let main_err = Error::from(config_err);
        assert!(matches!(main_err, Error::Config(_)));
        assert!(main_err.to_string().contains("Config error"));
        assert!(main_err
            .to_string()
            .contains("Failed to parse configuration"));
    }

    #[test]
    fn test_config_error_debug() {
        let err = ConfigError::file_not_found("test.ron");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("FileNotFound"));
        assert!(debug_str.contains("test.ron"));
    }

    #[test]
    fn test_config_error_chain_through_main() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let config_err = ConfigError::from(io_err);
        let main_err = Error::from(config_err);

        let error_chain = format!("{main_err}");
        assert!(error_chain.contains("Config error"));
        assert!(error_chain.contains("I/O error in configuration"));
        assert!(error_chain.contains("Access denied"));
    }
}
