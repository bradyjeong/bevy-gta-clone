use config_core::{
    EngineConfig, SuspensionConfig, TransmissionConfig, TransmissionType, VehicleConfig,
    WheelConfig,
};

#[test]
fn test_all_types_exported() {
    // Test that all the types are properly exported
    let vehicle_config = VehicleConfig::default();
    let engine_config = EngineConfig::default();
    let wheel_config = WheelConfig::default();
    let suspension_config = SuspensionConfig::default();
    let transmission_config = TransmissionConfig::default();
    let transmission_type = TransmissionType::default();

    // Test serialization roundtrip
    let ron_str = ron::to_string(&vehicle_config).unwrap();
    let deserialized: VehicleConfig = ron::from_str(&ron_str).unwrap();
    assert_eq!(vehicle_config, deserialized);

    // Test default values
    assert_eq!(vehicle_config.mass, 1500.0);
    assert_eq!(engine_config.max_power, 300.0);
    assert_eq!(wheel_config.radius, 0.33);
    assert_eq!(suspension_config.spring_stiffness, 35000.0);
    assert_eq!(transmission_config.final_drive_ratio, 3.73);
    assert_eq!(transmission_type, TransmissionType::Automatic);
}
