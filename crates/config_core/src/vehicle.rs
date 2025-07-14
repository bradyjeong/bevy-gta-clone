//! Vehicle configuration types
//!
//! This module contains Oracle's exact VehicleConfig specification for Sprint 2 Day 2.
//! Provides comprehensive configuration for realistic vehicle physics parameters.

use serde::{Deserialize, Serialize};

/// Main vehicle configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VehicleConfig {
    /// Vehicle mass in kilograms
    pub mass: f32,
    /// Engine configuration
    pub engine: EngineConfig,
    /// Wheel configuration for all four wheels
    pub wheels: [WheelConfig; 4],
    /// Suspension configuration
    pub suspension: SuspensionConfig,
    /// Transmission configuration
    pub transmission: TransmissionConfig,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            mass: 1500.0,
            engine: EngineConfig::default(),
            wheels: [WheelConfig::default(); 4],
            suspension: SuspensionConfig::default(),
            transmission: TransmissionConfig::default(),
        }
    }
}

impl crate::Config for VehicleConfig {
    const FILE_NAME: &'static str = "vehicle.ron";
}

/// Engine configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EngineConfig {
    /// Maximum engine power in horsepower
    pub max_power: f32,
    /// Engine power curve RPM points
    pub power_curve_rpm: Vec<f32>,
    /// Engine power curve power values (matching power_curve_rpm length)
    pub power_curve_power: Vec<f32>,
    /// Engine torque curve RPM points
    pub torque_curve_rpm: Vec<f32>,
    /// Engine torque curve torque values (matching torque_curve_rpm length)
    pub torque_curve_torque: Vec<f32>,
    /// Idle RPM
    pub idle_rpm: f32,
    /// Maximum RPM
    pub max_rpm: f32,
    /// Engine braking coefficient
    pub engine_braking: f32,
    /// Fuel consumption rate (liters per hour at max power)
    pub fuel_consumption: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_power: 300.0,
            power_curve_rpm: vec![1000.0, 2000.0, 4000.0, 6000.0, 7000.0],
            power_curve_power: vec![100.0, 200.0, 280.0, 300.0, 280.0],
            torque_curve_rpm: vec![1000.0, 2000.0, 3000.0, 4000.0, 6000.0],
            torque_curve_torque: vec![200.0, 350.0, 400.0, 380.0, 300.0],
            idle_rpm: 800.0,
            max_rpm: 7000.0,
            engine_braking: 0.3,
            fuel_consumption: 15.0,
        }
    }
}

/// Wheel configuration parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WheelConfig {
    /// Wheel radius in meters
    pub radius: f32,
    /// Wheel width in meters
    pub width: f32,
    /// Wheel mass in kilograms
    pub mass: f32,
    /// Tire grip coefficient (0.0 to 2.0)
    pub grip: f32,
    /// Rolling resistance coefficient
    pub rolling_resistance: f32,
    /// Lateral friction coefficient
    pub lateral_friction: f32,
    /// Longitudinal friction coefficient
    pub longitudinal_friction: f32,
    /// Tire stiffness
    pub stiffness: f32,
    /// Tire damping
    pub damping: f32,
}

impl Default for WheelConfig {
    fn default() -> Self {
        Self {
            radius: 0.33,
            width: 0.225,
            mass: 25.0,
            grip: 1.0,
            rolling_resistance: 0.015,
            lateral_friction: 1.2,
            longitudinal_friction: 1.0,
            stiffness: 50000.0,
            damping: 2500.0,
        }
    }
}

/// Suspension configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SuspensionConfig {
    /// Spring stiffness (N/m)
    pub spring_stiffness: f32,
    /// Damper damping coefficient (N·s/m)
    pub damper_damping: f32,
    /// Maximum suspension compression (meters)
    pub max_compression: f32,
    /// Maximum suspension extension (meters)
    pub max_extension: f32,
    /// Rest length of suspension (meters)
    pub rest_length: f32,
    /// Anti-roll bar stiffness (N·m/rad)
    pub anti_roll_bar_stiffness: f32,
    /// Suspension travel (meters)
    pub travel: f32,
    /// Camber angle (degrees)
    pub camber: f32,
    /// Caster angle (degrees)
    pub caster: f32,
    /// Toe angle (degrees)
    pub toe: f32,
}

impl Default for SuspensionConfig {
    fn default() -> Self {
        Self {
            spring_stiffness: 35000.0,
            damper_damping: 3500.0,
            max_compression: 0.15,
            max_extension: 0.15,
            rest_length: 0.5,
            anti_roll_bar_stiffness: 15000.0,
            travel: 0.3,
            camber: 0.0,
            caster: 6.0,
            toe: 0.0,
        }
    }
}

/// Transmission configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TransmissionConfig {
    /// Gear ratios (including reverse gear at index 0)
    pub gear_ratios: Vec<f32>,
    /// Final drive ratio
    pub final_drive_ratio: f32,
    /// Clutch engagement RPM
    pub clutch_engagement_rpm: f32,
    /// Shift up RPM threshold
    pub shift_up_rpm: f32,
    /// Shift down RPM threshold
    pub shift_down_rpm: f32,
    /// Shift time (seconds)
    pub shift_time: f32,
    /// Transmission efficiency (0.0 to 1.0)
    pub efficiency: f32,
    /// Transmission type
    pub transmission_type: TransmissionType,
}

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            gear_ratios: vec![-2.9, 0.0, 3.5, 2.0, 1.4, 1.0, 0.8, 0.6], // reverse, neutral, 1st-6th
            final_drive_ratio: 3.73,
            clutch_engagement_rpm: 1200.0,
            shift_up_rpm: 6500.0,
            shift_down_rpm: 2000.0,
            shift_time: 0.3,
            efficiency: 0.95,
            transmission_type: TransmissionType::Automatic,
        }
    }
}

/// Transmission type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransmissionType {
    Manual,
    Automatic,
    CVT,
    DualClutch,
}

impl Default for TransmissionType {
    fn default() -> Self {
        Self::Automatic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_config_roundtrip() {
        let config = VehicleConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: VehicleConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_engine_config_roundtrip() {
        let config = EngineConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: EngineConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_wheel_config_roundtrip() {
        let config = WheelConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: WheelConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_suspension_config_roundtrip() {
        let config = SuspensionConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: SuspensionConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_transmission_config_roundtrip() {
        let config = TransmissionConfig::default();
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: TransmissionConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_transmission_type_roundtrip() {
        let transmission_type = TransmissionType::Manual;
        let serialized = ron::to_string(&transmission_type).unwrap();
        let deserialized: TransmissionType = ron::from_str(&serialized).unwrap();
        assert_eq!(transmission_type, deserialized);
    }

    #[test]
    fn test_vehicle_config_serde_default() {
        // Test that partial configs work with serde(default)
        let partial_ron = r#"(
            mass: 1200.0,
            engine: (
                max_power: 400.0,
            ),
        )"#;

        let config: VehicleConfig = ron::from_str(partial_ron).unwrap();
        assert_eq!(config.mass, 1200.0);
        assert_eq!(config.engine.max_power, 400.0);
        // Should use default for other fields
        assert_eq!(config.engine.idle_rpm, 800.0);
        assert_eq!(config.wheels[0].radius, 0.33);
    }

    #[test]
    fn test_engine_config_power_curves() {
        let config = EngineConfig::default();
        assert_eq!(config.power_curve_rpm.len(), config.power_curve_power.len());
        assert_eq!(
            config.torque_curve_rpm.len(),
            config.torque_curve_torque.len()
        );

        // Verify curves are realistic
        assert!(config.power_curve_rpm.iter().all(|&rpm| rpm > 0.0));
        assert!(config.power_curve_power.iter().all(|&power| power > 0.0));
        assert!(config.torque_curve_rpm.iter().all(|&rpm| rpm > 0.0));
        assert!(config
            .torque_curve_torque
            .iter()
            .all(|&torque| torque > 0.0));
    }

    #[test]
    fn test_wheel_config_physical_constraints() {
        let config = WheelConfig::default();
        assert!(config.radius > 0.0);
        assert!(config.width > 0.0);
        assert!(config.mass > 0.0);
        assert!(config.grip >= 0.0);
        assert!(config.rolling_resistance >= 0.0);
        assert!(config.lateral_friction >= 0.0);
        assert!(config.longitudinal_friction >= 0.0);
        assert!(config.stiffness > 0.0);
        assert!(config.damping > 0.0);
    }

    #[test]
    fn test_suspension_config_physical_constraints() {
        let config = SuspensionConfig::default();
        assert!(config.spring_stiffness > 0.0);
        assert!(config.damper_damping > 0.0);
        assert!(config.max_compression > 0.0);
        assert!(config.max_extension > 0.0);
        assert!(config.rest_length > 0.0);
        assert!(config.anti_roll_bar_stiffness >= 0.0);
        assert!(config.travel > 0.0);
    }

    #[test]
    fn test_transmission_config_gear_ratios() {
        let config = TransmissionConfig::default();
        assert!(!config.gear_ratios.is_empty());

        // First gear should be reverse (negative)
        assert!(config.gear_ratios[0] < 0.0);

        // Second gear should be neutral (0.0)
        assert_eq!(config.gear_ratios[1], 0.0);

        // Forward gears should be positive and decreasing
        for i in 2..config.gear_ratios.len() {
            assert!(config.gear_ratios[i] > 0.0);
            if i > 2 {
                assert!(config.gear_ratios[i] <= config.gear_ratios[i - 1]);
            }
        }

        assert!(config.final_drive_ratio > 0.0);
        assert!(config.efficiency > 0.0 && config.efficiency <= 1.0);
        assert!(config.shift_time > 0.0);
    }

    #[test]
    fn test_vehicle_config_array_serialization() {
        let config = VehicleConfig::default();
        let serialized = ron::to_string(&config).unwrap();

        // Verify wheels array is properly serialized
        assert!(serialized.contains("wheels:"));

        let deserialized: VehicleConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config.wheels.len(), deserialized.wheels.len());
        for (original, deserialized) in config.wheels.iter().zip(deserialized.wheels.iter()) {
            assert_eq!(original, deserialized);
        }
    }

    #[test]
    fn test_transmission_type_all_variants() {
        let types = vec![
            TransmissionType::Manual,
            TransmissionType::Automatic,
            TransmissionType::CVT,
            TransmissionType::DualClutch,
        ];

        for transmission_type in types {
            let serialized = ron::to_string(&transmission_type).unwrap();
            let deserialized: TransmissionType = ron::from_str(&serialized).unwrap();
            assert_eq!(transmission_type, deserialized);
        }
    }

    #[test]
    fn test_nested_config_roundtrip() {
        let config = VehicleConfig {
            mass: 1800.0,
            engine: EngineConfig {
                max_power: 500.0,
                idle_rpm: 900.0,
                ..Default::default()
            },
            wheels: [
                WheelConfig {
                    radius: 0.35,
                    grip: 1.2,
                    ..Default::default()
                },
                WheelConfig {
                    radius: 0.35,
                    grip: 1.2,
                    ..Default::default()
                },
                WheelConfig {
                    radius: 0.35,
                    grip: 1.2,
                    ..Default::default()
                },
                WheelConfig {
                    radius: 0.35,
                    grip: 1.2,
                    ..Default::default()
                },
            ],
            suspension: SuspensionConfig {
                spring_stiffness: 40000.0,
                travel: 0.25,
                ..Default::default()
            },
            transmission: TransmissionConfig {
                transmission_type: TransmissionType::Manual,
                gear_ratios: vec![-2.5, 0.0, 4.0, 2.2, 1.5, 1.0, 0.7],
                ..Default::default()
            },
        };

        let serialized = ron::to_string(&config).unwrap();
        let deserialized: VehicleConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
