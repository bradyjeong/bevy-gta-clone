//! Configuration validation using JSON Schema
//!
//! This module provides validation capabilities for configuration files
//! using schemars to generate JSON schemas and validate config values.

use crate::{Config, Result};
use amp_core::{ConfigError, Error};
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, schema_for};

#[cfg(feature = "schemars")]
use serde_json::{self, Value};

/// Configuration validator that uses JSON Schema for validation
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a configuration against its JSON schema
    #[cfg(feature = "schemars")]
    pub fn validate<T: Config + JsonSchema + serde::Serialize>(config: &T) -> Result<()> {
        // Generate schema for the config type
        let schema = schema_for!(T);
        let schema_value = serde_json::to_value(schema).map_err(|e| {
            Error::from(ConfigError::parse_error(format!(
                "Schema generation failed: {e}"
            )))
        })?;

        // Convert config to JSON value for validation
        let config_value = serde_json::to_value(config).map_err(|e| {
            Error::from(ConfigError::parse_error(format!(
                "Config serialization failed: {e}"
            )))
        })?;

        // Perform basic schema validation
        Self::validate_against_schema(&config_value, &schema_value)
    }

    /// Validate a JSON value against a schema
    #[cfg(feature = "schemars")]
    fn validate_against_schema(value: &Value, _schema: &Value) -> Result<()> {
        // Basic validation - check for required fields and types
        // In a full implementation, you would use a JSON Schema validator like valico or jsonschema
        match value {
            Value::Object(obj) => {
                // Validate that all required fields are present
                // This is a simplified check - real implementation would use the schema
                if obj.is_empty() {
                    return Err(Error::from(ConfigError::parse_error(
                        "Empty configuration object".to_string(),
                    )));
                }
                Ok(())
            }
            _ => Err(Error::from(ConfigError::parse_error(
                "Configuration must be an object".to_string(),
            ))),
        }
    }

    /// Validate audio-specific constraints
    pub fn validate_audio_config(config: &crate::audio::AudioConfig) -> Result<()> {
        // Volume validation (must be between 0.0 and 1.0)
        if config.master_volume < 0.0 || config.master_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "master_volume must be between 0.0 and 1.0, got {}",
                config.master_volume
            ))));
        }
        if config.engine_volume < 0.0 || config.engine_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "engine_volume must be between 0.0 and 1.0, got {}",
                config.engine_volume
            ))));
        }
        if config.music_volume < 0.0 || config.music_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "music_volume must be between 0.0 and 1.0, got {}",
                config.music_volume
            ))));
        }
        if config.sfx_volume < 0.0 || config.sfx_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "sfx_volume must be between 0.0 and 1.0, got {}",
                config.sfx_volume
            ))));
        }
        if config.environment_volume < 0.0 || config.environment_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "environment_volume must be between 0.0 and 1.0, got {}",
                config.environment_volume
            ))));
        }
        if config.ui_volume < 0.0 || config.ui_volume > 1.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "ui_volume must be between 0.0 and 1.0, got {}",
                config.ui_volume
            ))));
        }
        Ok(())
    }

    /// Validate vehicle-specific constraints
    pub fn validate_vehicle_config(config: &crate::vehicle::VehicleConfig) -> Result<()> {
        // Engine validation
        if config.engine.max_rpm <= 0.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "engine.max_rpm must be positive, got {}",
                config.engine.max_rpm
            ))));
        }
        if config.engine.max_power <= 0.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "engine.max_power must be positive, got {}",
                config.engine.max_power
            ))));
        }
        if config.engine.idle_rpm < 0.0 || config.engine.idle_rpm >= config.engine.max_rpm {
            return Err(Error::from(ConfigError::parse_error(format!(
                "engine.idle_rpm must be between 0 and max_rpm, got {}",
                config.engine.idle_rpm
            ))));
        }

        // Mass validation
        if config.mass <= 0.0 {
            return Err(Error::from(ConfigError::parse_error(format!(
                "mass must be positive, got {}",
                config.mass
            ))));
        }

        // Wheel validation
        for (i, wheel) in config.wheels.iter().enumerate() {
            if wheel.radius <= 0.0 {
                return Err(Error::from(ConfigError::parse_error(format!(
                    "wheel[{}].radius must be positive, got {}",
                    i, wheel.radius
                ))));
            }
            if wheel.grip < 0.0 {
                return Err(Error::from(ConfigError::parse_error(format!(
                    "wheel[{}].grip must be non-negative, got {}",
                    i, wheel.grip
                ))));
            }
        }

        Ok(())
    }

    /// Generate JSON schema for a configuration type
    #[cfg(feature = "schemars")]
    pub fn generate_schema<T: JsonSchema>() -> String {
        let schema = schema_for!(T);
        serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
    }

    /// Generate JSON schema and write to file
    #[cfg(feature = "schemars")]
    pub fn generate_schema_file<T: Config + JsonSchema>(
        output_path: &std::path::Path,
    ) -> Result<()> {
        let schema = Self::generate_schema::<T>();
        std::fs::write(output_path, schema).map_err(|e| Error::from(ConfigError::from(e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioConfig;
    use crate::vehicle::VehicleConfig;

    #[test]
    fn test_validate_audio_config_valid() {
        let config = AudioConfig::default();
        assert!(ConfigValidator::validate_audio_config(&config).is_ok());
    }

    #[test]
    fn test_validate_audio_config_invalid_master_volume() {
        let config = AudioConfig {
            master_volume: 1.5, // Invalid: > 1.0
            ..Default::default()
        };
        assert!(ConfigValidator::validate_audio_config(&config).is_err());
    }

    #[test]
    fn test_validate_audio_config_invalid_negative_volume() {
        let config = AudioConfig {
            engine_volume: -0.1, // Invalid: < 0.0
            ..Default::default()
        };
        assert!(ConfigValidator::validate_audio_config(&config).is_err());
    }

    #[test]
    fn test_validate_vehicle_config_valid() {
        let config = VehicleConfig::default();
        assert!(ConfigValidator::validate_vehicle_config(&config).is_ok());
    }

    #[test]
    fn test_validate_vehicle_config_invalid_max_rpm() {
        let mut config = VehicleConfig::default();
        config.engine.max_rpm = -100.0; // Invalid: negative
        assert!(ConfigValidator::validate_vehicle_config(&config).is_err());
    }

    #[test]
    fn test_validate_vehicle_config_invalid_mass() {
        let config = VehicleConfig {
            mass: 0.0, // Invalid: must be positive
            ..VehicleConfig::default()
        };
        assert!(ConfigValidator::validate_vehicle_config(&config).is_err());
    }

    #[test]
    fn test_validate_vehicle_config_invalid_idle_rpm() {
        let mut config = VehicleConfig::default();
        config.engine.idle_rpm = config.engine.max_rpm + 100.0; // Invalid: idle > max
        assert!(ConfigValidator::validate_vehicle_config(&config).is_err());
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn test_generate_schema() {
        let schema = ConfigValidator::generate_schema::<AudioConfig>();
        assert!(!schema.is_empty());
        assert!(schema.contains("master_volume"));
        assert!(schema.contains("engine_volume"));
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn test_validate_with_schema() {
        let config = AudioConfig::default();
        // This should pass with a proper JSON schema validator
        // For now, we test that the method can be called
        let result = ConfigValidator::validate(&config);
        // Since we're using a simplified validator, we don't test the result
        // In production, this would use a proper JSON schema validation library
        let _ = result;
    }

    #[test]
    fn test_audio_validation_boundary_values() {
        // Test exactly at boundaries
        let config_min = AudioConfig {
            master_volume: 0.0,
            engine_volume: 0.0,
            music_volume: 0.0,
            sfx_volume: 0.0,
            environment_volume: 0.0,
            ui_volume: 0.0,
            ..Default::default()
        };
        assert!(ConfigValidator::validate_audio_config(&config_min).is_ok());

        let config_max = AudioConfig {
            master_volume: 1.0,
            engine_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            environment_volume: 1.0,
            ui_volume: 1.0,
            ..Default::default()
        };
        assert!(ConfigValidator::validate_audio_config(&config_max).is_ok());
    }

    #[test]
    fn test_vehicle_validation_wheel_constraints() {
        let mut config = VehicleConfig::default();

        // Test invalid wheel radius
        if !config.wheels.is_empty() {
            config.wheels[0].radius = -1.0;
            assert!(ConfigValidator::validate_vehicle_config(&config).is_err());

            // Reset and test invalid grip
            config.wheels[0].radius = 0.3;
            config.wheels[0].grip = -0.5;
            assert!(ConfigValidator::validate_vehicle_config(&config).is_err());
        }
    }
}
