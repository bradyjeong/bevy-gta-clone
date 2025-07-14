use config_core::vehicle::*;

#[test]
fn test_sports_car_config_loads() {
    let ron_content = include_str!("../../../assets/prefabs/vehicles/sports_car.ron");
    let config: VehicleConfig = ron::from_str(ron_content).unwrap();

    // Verify basic properties
    assert_eq!(config.mass, 1200.0);
    assert_eq!(config.engine.max_power, 500.0);
    assert_eq!(
        config.transmission.transmission_type,
        TransmissionType::DualClutch
    );

    // Verify all wheels have same config
    for wheel in &config.wheels {
        assert_eq!(wheel.radius, 0.35);
        assert_eq!(wheel.grip, 1.3);
    }
}

#[test]
fn test_sedan_config_loads() {
    let ron_content = include_str!("../../../assets/prefabs/vehicles/sedan.ron");
    let config: VehicleConfig = ron::from_str(ron_content).unwrap();

    // Verify basic properties
    assert_eq!(config.mass, 1500.0);
    assert_eq!(config.engine.max_power, 180.0);
    assert_eq!(
        config.transmission.transmission_type,
        TransmissionType::Automatic
    );

    // Verify suspension is more comfortable than sports car
    assert!(config.suspension.spring_stiffness < 45000.0);
    assert!(config.suspension.travel > 0.3);
}

#[test]
fn test_truck_config_loads() {
    let ron_content = include_str!("../../../assets/prefabs/vehicles/truck.ron");
    let config: VehicleConfig = ron::from_str(ron_content).unwrap();

    // Verify basic properties
    assert_eq!(config.mass, 3500.0);
    assert_eq!(config.engine.max_power, 400.0);
    assert_eq!(
        config.transmission.transmission_type,
        TransmissionType::Manual
    );

    // Verify heavy duty characteristics
    assert!(config
        .engine
        .torque_curve_torque
        .iter()
        .any(|&torque| torque > 700.0));
    assert!(config.wheels[0].radius > 0.4);
    assert!(config.wheels[0].mass > 40.0);
}

#[test]
fn test_motorcycle_config_loads() {
    let ron_content = include_str!("../../../assets/prefabs/vehicles/motorcycle.ron");
    let config: VehicleConfig = ron::from_str(ron_content).unwrap();

    // Verify basic properties
    assert_eq!(config.mass, 180.0);
    assert_eq!(config.engine.max_power, 150.0);
    assert_eq!(
        config.transmission.transmission_type,
        TransmissionType::Manual
    );

    // Verify lightweight characteristics
    assert!(config.wheels[0].mass < 10.0);
    assert!(config.suspension.spring_stiffness < 30000.0);
    assert!(config.engine.max_rpm > 10000.0);
}

#[test]
fn test_all_configs_roundtrip() {
    let configs = [
        include_str!("../../../assets/prefabs/vehicles/sports_car.ron"),
        include_str!("../../../assets/prefabs/vehicles/sedan.ron"),
        include_str!("../../../assets/prefabs/vehicles/truck.ron"),
        include_str!("../../../assets/prefabs/vehicles/motorcycle.ron"),
    ];

    for (i, ron_content) in configs.iter().enumerate() {
        let config: VehicleConfig = ron::from_str(ron_content)
            .unwrap_or_else(|e| panic!("Failed to parse config {i}: {e}"));

        // Verify roundtrip serialization
        let serialized = ron::to_string(&config).unwrap();
        let deserialized: VehicleConfig = ron::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized, "Config {i} failed roundtrip");
    }
}

#[test]
fn test_vehicle_config_realistic_constraints() {
    let configs = [
        include_str!("../../../assets/prefabs/vehicles/sports_car.ron"),
        include_str!("../../../assets/prefabs/vehicles/sedan.ron"),
        include_str!("../../../assets/prefabs/vehicles/truck.ron"),
        include_str!("../../../assets/prefabs/vehicles/motorcycle.ron"),
    ];

    for (i, ron_content) in configs.iter().enumerate() {
        let config: VehicleConfig = ron::from_str(ron_content)
            .unwrap_or_else(|e| panic!("Failed to parse config {i}: {e}"));

        // Verify realistic mass constraints
        assert!(
            config.mass > 0.0 && config.mass < 10000.0,
            "Config {i} has unrealistic mass"
        );

        // Verify engine constraints
        assert!(
            config.engine.max_power > 0.0 && config.engine.max_power < 2000.0,
            "Config {i} has unrealistic power"
        );
        assert!(
            config.engine.idle_rpm > 0.0 && config.engine.idle_rpm < config.engine.max_rpm,
            "Config {i} has invalid RPM range"
        );

        // Verify wheel constraints
        for (j, wheel) in config.wheels.iter().enumerate() {
            assert!(
                wheel.radius > 0.0 && wheel.radius < 1.0,
                "Config {i} wheel {j} has unrealistic radius"
            );
            assert!(
                wheel.mass > 0.0 && wheel.mass < 100.0,
                "Config {i} wheel {j} has unrealistic mass"
            );
            assert!(
                wheel.grip >= 0.0 && wheel.grip <= 2.0,
                "Config {i} wheel {j} has unrealistic grip"
            );
        }

        // Verify transmission constraints
        assert!(
            !config.transmission.gear_ratios.is_empty(),
            "Config {i} has no gear ratios"
        );
        assert!(
            config.transmission.gear_ratios[0] < 0.0,
            "Config {i} reverse gear should be negative"
        );
        assert_eq!(
            config.transmission.gear_ratios[1], 0.0,
            "Config {i} neutral gear should be zero"
        );
        assert!(
            config.transmission.final_drive_ratio > 0.0,
            "Config {i} final drive ratio should be positive"
        );
    }
}
