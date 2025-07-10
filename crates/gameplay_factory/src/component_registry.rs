//! Thread-safe component registry for RON deserialization

use crate::Error;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Component deserializer function type
pub type ComponentDeserializer =
    Box<dyn Fn(&ron::Value, &mut Commands, Entity) -> Result<(), Error> + Send + Sync>;

/// Thread-safe global component registry
static COMPONENT_REGISTRY: Lazy<RwLock<HashMap<&'static str, ComponentDeserializer>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a component deserializer
///
/// # Arguments
///
/// * `name` - The component type name to register
/// * `deserializer` - Function that deserializes RON data into component insertion
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the component is already registered
///
/// # Examples
///
/// ```
/// use gameplay_factory::{register_component, Error};
/// use bevy::prelude::*;
///
/// fn my_component_deserializer(
///     value: &ron::Value,
///     cmd: &mut Commands,
///     entity: Entity,
/// ) -> Result<(), Error> {
///     // Deserialize component data and insert into entity
///     Ok(())
/// }
///
/// register_component("MyComponent", Box::new(my_component_deserializer)).unwrap();
/// ```
pub fn register_component(
    name: &'static str,
    deserializer: ComponentDeserializer,
) -> Result<(), Error> {
    let mut registry = COMPONENT_REGISTRY.write().unwrap();

    if registry.contains_key(name) {
        return Err(Error::validation(format!(
            "Component '{name}' already registered"
        )));
    }

    registry.insert(name, deserializer);
    log::debug!("Registered component deserializer for '{name}'");
    Ok(())
}

/// Call a component deserializer by name
///
/// # Arguments
///
/// * `name` - The component type name to look up
/// * `value` - The RON value to deserialize
/// * `cmd` - The commands object for entity manipulation
/// * `entity` - The entity to attach the component to
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the component is not found or deserialization fails
///
/// # Examples
///
/// ```ignore
/// // This example uses bevy_ecs 0.13 types which conflict with bevy 0.16.1
/// // Will be fixed in Phase 5B migration
/// use gameplay_factory::call_component_deserializer;
/// call_component_deserializer("Transform", &ron_value, &mut commands, entity)?;
/// ```
pub fn call_component_deserializer(
    name: &str,
    value: &ron::Value,
    cmd: &mut Commands,
    entity: Entity,
) -> Result<(), Error> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    if let Some(deserializer) = registry.get(name) {
        deserializer(value, cmd, entity)
    } else {
        Err(Error::validation(format!(
            "Component type '{name}' not found in registry"
        )))
    }
}

/// Get a list of all registered component names
///
/// # Returns
///
/// Returns a vector of all registered component type names
///
/// # Examples
///
/// ```
/// use gameplay_factory::registered_components;
///
/// let components = registered_components();
/// println!("Available components: {:?}", components);
/// ```
pub fn registered_components() -> Vec<&'static str> {
    let registry = COMPONENT_REGISTRY.read().unwrap();
    registry.keys().copied().collect()
}

/// Clear all registered components (primarily for testing)
///
/// # Safety
///
/// This function is safe to call but will remove all registered components.
/// It's primarily intended for testing scenarios.
#[cfg(test)]
pub fn clear_registry() {
    let mut registry = COMPONENT_REGISTRY.write().unwrap();
    registry.clear();
    log::debug!("Component registry cleared");
}

/// Register default Bevy components
///
/// This function registers deserializers for basic Bevy components like Transform, Name, etc.
/// It should be called during App initialization.
///
/// # Examples
///
/// ```
/// use gameplay_factory::register_default_components;
///
/// register_default_components();
/// ```
pub fn register_default_components() {
    // Register Transform component
    let _ = register_component(
        "Transform",
        Box::new(|value, cmd, entity| {
            let transform = deserialize_transform(value)?;
            cmd.entity(entity).insert(transform);
            Ok(())
        }),
    );

    // Register Name component
    let _ = register_component(
        "Name",
        Box::new(|value, cmd, entity| {
            let name = deserialize_name(value)?;
            cmd.entity(entity).insert(name);
            Ok(())
        }),
    );

    // Register Visibility component
    let _ = register_component(
        "Visibility",
        Box::new(|value, cmd, entity| {
            let visibility = deserialize_visibility(value)?;
            cmd.entity(entity).insert(visibility);
            Ok(())
        }),
    );

    log::info!("Default components registered");
}

/// Deserialize a Transform component from RON data
fn deserialize_transform(value: &ron::Value) -> Result<Transform, Error> {
    // Vec3, Quat, Transform are now imported via bevy::prelude::*

    match value {
        ron::Value::Map(map) => {
            let mut translation = Vec3::ZERO;
            let mut rotation = Quat::IDENTITY;
            let mut scale = Vec3::ONE;

            // Extract values from the map
            for (k, v) in map.iter() {
                if let ron::Value::String(s) = k {
                    match s.as_str() {
                        "translation" => {
                            translation = deserialize_vec3(v)?;
                        }
                        "rotation" => {
                            rotation = deserialize_quat(v)?;
                        }
                        "scale" => {
                            scale = deserialize_vec3(v)?;
                        }
                        _ => {
                            // Ignore unknown fields
                        }
                    }
                }
            }

            Ok(Transform {
                translation,
                rotation,
                scale,
            })
        }
        _ => Err(Error::validation("Transform component must be a map")),
    }
}

/// Deserialize a Name component from RON data
fn deserialize_name(value: &ron::Value) -> Result<Name, Error> {
    // Name is now imported via bevy::prelude::*

    match value {
        ron::Value::String(s) => Ok(Name::new(s.clone())),
        _ => Err(Error::validation("Name component must be a string")),
    }
}

/// Deserialize a Visibility component from RON data
fn deserialize_visibility(value: &ron::Value) -> Result<Visibility, Error> {
    // Visibility is now imported via bevy::prelude::*

    match value {
        ron::Value::String(s) => match s.as_str() {
            "Inherited" => Ok(Visibility::Inherited),
            "Hidden" => Ok(Visibility::Hidden),
            "Visible" => Ok(Visibility::Visible),
            _ => Err(Error::validation(format!(
                "Invalid visibility value: '{s}'. Must be 'Inherited', 'Hidden', or 'Visible'"
            ))),
        },
        _ => Err(Error::validation("Visibility component must be a string")),
    }
}

/// Deserialize a Vec3 from RON data
fn deserialize_vec3(value: &ron::Value) -> Result<Vec3, Error> {
    // Vec3 is now imported via bevy::prelude::*

    match value {
        ron::Value::Map(map) => {
            let x = extract_number(map, "x")?;
            let y = extract_number(map, "y")?;
            let z = extract_number(map, "z")?;
            Ok(Vec3::new(x, y, z))
        }
        _ => Err(Error::validation("Vec3 must be a map with x, y, z fields")),
    }
}

/// Deserialize a Quat from RON data
fn deserialize_quat(value: &ron::Value) -> Result<Quat, Error> {
    // Quat is now imported via bevy::prelude::*

    match value {
        ron::Value::Map(map) => {
            let x = extract_number(map, "x")?;
            let y = extract_number(map, "y")?;
            let z = extract_number(map, "z")?;
            let w = extract_number(map, "w")?;
            Ok(Quat::from_xyzw(x, y, z, w))
        }
        _ => Err(Error::validation(
            "Quat must be a map with x, y, z, w fields",
        )),
    }
}

/// Extract a number from a RON map
fn extract_number(map: &ron::Map, key: &str) -> Result<f32, Error> {
    // RON maps are actually IndexMap<ron::Value, ron::Value>
    for (k, v) in map.iter() {
        if let ron::Value::String(s) = k {
            if s == key {
                if let ron::Value::Number(num) = v {
                    return Ok(num.into_f64() as f32);
                } else {
                    return Err(Error::validation(format!("Field '{key}' must be a number")));
                }
            }
        }
    }
    Err(Error::validation(format!("Missing required field '{key}'")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use crossbeam_utils::thread;
    use rstest::*;
    use serial_test::serial;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[rstest]
    #[serial]
    fn test_register_component_success() {
        clear_registry();

        let result = register_component("TestComponent", Box::new(|_, _, _| Ok(())));

        assert!(result.is_ok());

        // Test that we can call the deserializer
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmd = Commands::new(&mut queue, &world);
        let entity = cmd.spawn_empty().id();
        let test_value = ron::Value::Number(ron::Number::new(42.0));

        let result = call_component_deserializer("TestComponent", &test_value, &mut cmd, entity);
        assert!(result.is_ok());
    }

    #[rstest]
    #[serial]
    fn test_register_component_duplicate() {
        clear_registry();

        let first_result = register_component("TestComponent", Box::new(|_, _, _| Ok(())));
        assert!(first_result.is_ok());

        let second_result = register_component("TestComponent", Box::new(|_, _, _| Ok(())));
        assert!(second_result.is_err());
        assert!(
            second_result
                .unwrap_err()
                .to_string()
                .contains("already registered")
        );
    }

    #[rstest]
    fn test_component_deserializer_not_found() {
        clear_registry();

        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmd = Commands::new(&mut queue, &world);
        let entity = cmd.spawn_empty().id();
        let test_value = ron::Value::Number(ron::Number::new(42.0));

        let result =
            call_component_deserializer("NonExistentComponent", &test_value, &mut cmd, entity);
        assert!(result.is_err());
    }

    #[rstest]
    #[serial]
    fn test_registered_components() {
        clear_registry();

        let _ = register_component("Component1", Box::new(|_, _, _| Ok(())));
        let _ = register_component("Component2", Box::new(|_, _, _| Ok(())));

        let components = registered_components();
        assert_eq!(components.len(), 2);
        assert!(components.contains(&"Component1"));
        assert!(components.contains(&"Component2"));
    }

    #[rstest]
    #[serial]
    fn test_clear_registry() {
        clear_registry();

        let _ = register_component("TestComponent", Box::new(|_, _, _| Ok(())));

        // Test that we can call the deserializer
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmd = Commands::new(&mut queue, &world);
        let entity = cmd.spawn_empty().id();
        let test_value = ron::Value::Number(ron::Number::new(42.0));

        let result = call_component_deserializer("TestComponent", &test_value, &mut cmd, entity);
        assert!(result.is_ok());

        clear_registry();
        let result = call_component_deserializer("TestComponent", &test_value, &mut cmd, entity);
        assert!(result.is_err());
        assert_eq!(registered_components().len(), 0);
    }

    #[rstest]
    #[serial]
    fn test_thread_safety_concurrent_registration() {
        clear_registry();

        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        thread::scope(|s| {
            for i in 0..10 {
                let success_count = Arc::clone(&success_count);
                let error_count = Arc::clone(&error_count);

                s.spawn(move |_| {
                    let component_name = format!("Component{i}");
                    // We need to leak the string to get a 'static str
                    let static_name: &'static str = Box::leak(component_name.into_boxed_str());

                    let result = register_component(static_name, Box::new(|_, _, _| Ok(())));

                    if result.is_ok() {
                        success_count.fetch_add(1, Ordering::SeqCst);
                    } else {
                        error_count.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(success_count.load(Ordering::SeqCst), 10);
        assert_eq!(error_count.load(Ordering::SeqCst), 0);
        assert_eq!(registered_components().len(), 10);
    }

    #[rstest]
    #[serial]
    fn test_thread_safety_duplicate_registration() {
        clear_registry();

        let success_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        thread::scope(|s| {
            for _ in 0..5 {
                let success_count = Arc::clone(&success_count);
                let error_count = Arc::clone(&error_count);

                s.spawn(move |_| {
                    let result =
                        register_component("DuplicateComponent", Box::new(|_, _, _| Ok(())));

                    if result.is_ok() {
                        success_count.fetch_add(1, Ordering::SeqCst);
                    } else {
                        error_count.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        })
        .unwrap();

        // Only one should succeed, the rest should fail
        assert_eq!(success_count.load(Ordering::SeqCst), 1);
        assert_eq!(error_count.load(Ordering::SeqCst), 4);
        assert_eq!(registered_components().len(), 1);
    }

    #[rstest]
    #[serial]
    fn test_thread_safety_concurrent_read_write() {
        clear_registry();

        // Register some initial components
        let _ = register_component("InitialComponent", Box::new(|_, _, _| Ok(())));

        let read_count = Arc::new(AtomicUsize::new(0));
        let write_count = Arc::new(AtomicUsize::new(0));

        thread::scope(|s| {
            // Spawn reader threads
            for _ in 0..5 {
                let read_count = Arc::clone(&read_count);
                s.spawn(move |_| {
                    for i in 0..10 {
                        if i % 2 == 0 {
                            // Test calling a component deserializer
                            let world = World::new();
                            let mut queue = CommandQueue::default();
                            let mut cmd = Commands::new(&mut queue, &world);
                            let entity = cmd.spawn_empty().id();
                            let test_value = ron::Value::Number(ron::Number::new(42.0));
                            let _ = call_component_deserializer(
                                "InitialComponent",
                                &test_value,
                                &mut cmd,
                                entity,
                            );
                        } else {
                            let _ = registered_components();
                        }
                        read_count.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }

            // Spawn writer threads
            for i in 0..3 {
                let write_count = Arc::clone(&write_count);
                s.spawn(move |_| {
                    let component_name = format!("WriterComponent{i}");
                    let static_name: &'static str = Box::leak(component_name.into_boxed_str());

                    let result = register_component(static_name, Box::new(|_, _, _| Ok(())));

                    if result.is_ok() {
                        write_count.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(read_count.load(Ordering::SeqCst), 50);
        assert_eq!(write_count.load(Ordering::SeqCst), 3);
        assert_eq!(registered_components().len(), 4); // 1 initial + 3 writer components
    }

    #[rstest]
    #[serial]
    fn test_register_default_components() {
        clear_registry();

        register_default_components();

        let components = registered_components();
        assert!(components.contains(&"Transform"));
        assert!(components.contains(&"Name"));
        assert!(components.contains(&"Visibility"));
        assert!(components.len() >= 3);
    }

    #[rstest]
    fn test_deserialize_transform() {
        use ron::Value;

        let mut map = ron::Map::new();
        let mut translation_map = ron::Map::new();
        translation_map.insert(
            Value::String("x".to_string()),
            Value::Number(ron::Number::new(1.0)),
        );
        translation_map.insert(
            Value::String("y".to_string()),
            Value::Number(ron::Number::new(2.0)),
        );
        translation_map.insert(
            Value::String("z".to_string()),
            Value::Number(ron::Number::new(3.0)),
        );

        map.insert(
            Value::String("translation".to_string()),
            Value::Map(translation_map),
        );

        let transform_value = Value::Map(map);
        let transform = deserialize_transform(&transform_value).unwrap();

        assert_eq!(transform.translation.x, 1.0);
        assert_eq!(transform.translation.y, 2.0);
        assert_eq!(transform.translation.z, 3.0);
    }

    #[rstest]
    fn test_deserialize_name() {
        let name_value = ron::Value::String("TestEntity".to_string());
        let name = deserialize_name(&name_value).unwrap();

        assert_eq!(name.as_str(), "TestEntity");
    }

    #[rstest]
    fn test_deserialize_visibility() {
        // Visibility is now imported via bevy::prelude::*

        let visible_value = ron::Value::String("Visible".to_string());
        let visibility = deserialize_visibility(&visible_value).unwrap();
        assert_eq!(visibility, Visibility::Visible);

        let hidden_value = ron::Value::String("Hidden".to_string());
        let visibility = deserialize_visibility(&hidden_value).unwrap();
        assert_eq!(visibility, Visibility::Hidden);

        let inherited_value = ron::Value::String("Inherited".to_string());
        let visibility = deserialize_visibility(&inherited_value).unwrap();
        assert_eq!(visibility, Visibility::Inherited);
    }

    #[rstest]
    fn test_deserialize_invalid_visibility() {
        let invalid_value = ron::Value::String("Invalid".to_string());
        let result = deserialize_visibility(&invalid_value);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid visibility value")
        );
    }

    #[rstest]
    #[serial]
    fn test_component_deserializer_execution() {
        clear_registry();
        register_default_components();

        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmd = Commands::new(&mut queue, &world);
        let entity = cmd.spawn_empty().id();

        let name_value = ron::Value::String("TestName".to_string());
        let result = call_component_deserializer("Name", &name_value, &mut cmd, entity);
        assert!(result.is_ok());
    }
}
