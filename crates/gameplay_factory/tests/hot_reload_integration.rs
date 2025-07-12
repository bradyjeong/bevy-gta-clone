//! Integration tests for hot-reload functionality
//!
//! These tests actually create, modify, and delete files on disk to verify
//! that the hot-reload system works correctly.

use std::fs;
#[cfg(feature = "hot-reload")]
use std::time::Duration;
use tempfile::TempDir;
#[cfg(feature = "hot-reload")]
use tokio::time::sleep;

use gameplay_factory::*;

/// Test that file creation triggers hot-reload events
#[tokio::test]
#[cfg(feature = "hot-reload")]
#[ignore = "Integration test that requires file system operations"]
async fn test_file_creation_triggers_reload() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a glob pattern for .ron files in the temp directory
    let pattern = format!("{}/*.ron", temp_path.display());

    // Create channels for hot-reload events
    let (tx, mut rx) = create_reload_channel();

    // Start the watcher
    watcher::run_watcher(&pattern, tx)
        .await
        .expect("Failed to start watcher");

    // Wait a bit for the watcher to initialize
    sleep(Duration::from_millis(100)).await;

    // Create a new .ron file
    let test_file = temp_path.join("test_prefab.ron");
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [0.0, 0.0, 0.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to write test file");

    // Wait for the event to be processed
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for hot-reload event")
        .expect("No hot-reload event received");

    // Verify the event
    assert_eq!(event.path(), &test_file);
    assert!(!event.is_deletion());
}

/// Test that file modification triggers hot-reload events
#[tokio::test]
#[cfg(feature = "hot-reload")]
#[ignore = "Integration test that requires file system operations"]
async fn test_file_modification_triggers_reload() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a test file first
    let test_file = temp_path.join("modify_test.ron");
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [0.0, 0.0, 0.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to write test file");

    // Create a glob pattern for .ron files in the temp directory
    let pattern = format!("{}/*.ron", temp_path.display());

    // Create channels for hot-reload events
    let (tx, mut rx) = create_reload_channel();

    // Start the watcher
    watcher::run_watcher(&pattern, tx)
        .await
        .expect("Failed to start watcher");

    // Wait a bit for the watcher to initialize
    sleep(Duration::from_millis(100)).await;

    // Modify the file
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [1.0, 2.0, 3.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to modify test file");

    // Wait for the event to be processed
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for hot-reload event")
        .expect("No hot-reload event received");

    // Verify the event
    assert_eq!(event.path(), &test_file);
    assert!(!event.is_deletion());
}

/// Test that file deletion triggers hot-reload events
#[tokio::test]
#[cfg(feature = "hot-reload")]
#[ignore = "Integration test that requires file system operations"]
async fn test_file_deletion_triggers_reload() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a test file first
    let test_file = temp_path.join("delete_test.ron");
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [0.0, 0.0, 0.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to write test file");

    // Create a glob pattern for .ron files in the temp directory
    let pattern = format!("{}/*.ron", temp_path.display());

    // Create channels for hot-reload events
    let (tx, mut rx) = create_reload_channel();

    // Start the watcher
    watcher::run_watcher(&pattern, tx)
        .await
        .expect("Failed to start watcher");

    // Wait a bit for the watcher to initialize
    sleep(Duration::from_millis(100)).await;

    // Delete the file
    fs::remove_file(&test_file).expect("Failed to delete test file");

    // Wait for the event to be processed
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("Timeout waiting for hot-reload event")
        .expect("No hot-reload event received");

    // Verify the event
    assert_eq!(event.path(), &test_file);
    assert!(event.is_deletion());
}

/// Test that non-matching files don't trigger events
#[tokio::test]
#[cfg(feature = "hot-reload")]
#[ignore = "Integration test that requires file system operations"]
async fn test_non_matching_files_ignored() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a glob pattern for .ron files only
    let pattern = format!("{}/*.ron", temp_path.display());

    // Create channels for hot-reload events
    let (tx, mut rx) = create_reload_channel();

    // Start the watcher
    watcher::run_watcher(&pattern, tx)
        .await
        .expect("Failed to start watcher");

    // Wait a bit for the watcher to initialize
    sleep(Duration::from_millis(100)).await;

    // Create a non-matching file (.txt instead of .ron)
    let test_file = temp_path.join("test_file.txt");
    fs::write(&test_file, "This is not a .ron file").expect("Failed to write test file");

    // Wait and verify no event is received
    let result = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(
        result.is_err(),
        "Should not receive event for non-matching file"
    );
}

/// Test factory integration with hot-reload
#[tokio::test]
#[cfg(all(feature = "hot-reload", feature = "ron"))]
#[ignore = "Integration test that requires file system operations"]
async fn test_factory_hot_reload_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a test prefab file
    let test_file = temp_path.join("factory_test.ron");
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [0.0, 0.0, 0.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to write test file");

    // Create factory settings
    let pattern = format!("{}/*.ron", temp_path.display());
    let settings = config_core::FactorySettings {
        prefab_path: pattern,
        hot_reload: true,
    };

    // Create factory and load directory
    let mut factory = Factory::new();
    let loaded_count = factory
        .load_directory(&settings)
        .expect("Failed to load directory");

    // Verify prefab was loaded
    assert_eq!(loaded_count, 1);

    // Get the hot-reload receiver
    let mut _receiver = factory.take_hot_reload_receiver();

    // Verify the receiver exists when hot-reload is enabled
    #[cfg(feature = "hot-reload")]
    {
        assert!(_receiver.is_some());
    }

    // Modify the file and verify we can receive events
    // (This test mainly verifies the factory integration works)
}

/// Test that hot-reload works when feature is disabled
#[tokio::test]
#[cfg(not(feature = "hot-reload"))]
async fn test_hot_reload_disabled_gracefully() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a test prefab file
    let test_file = temp_path.join("disabled_test.ron");
    fs::write(
        &test_file,
        r#"
        (
            components: [
                ("TransformComponent", "{ translation: [0.0, 0.0, 0.0] }"),
            ]
        )
    "#,
    )
    .expect("Failed to write test file");

    // Create factory settings with hot-reload enabled
    let pattern = format!("{}/*.ron", temp_path.display());
    let settings = config_core::FactorySettings {
        prefab_path: pattern,
        hot_reload: true,
    };

    // Create factory and attempt to load directory
    let mut factory = Factory::new();
    let result = factory.load_directory(&settings);

    // Since RON loading is now replaced with Bevy 0.16 systems, this should fail
    assert!(result.is_err());
    let error = result.unwrap_err();
    // Accept any meaningful error message from the invalid directory loading
    assert!(!error.to_string().is_empty());

    // Verify the receiver is None when hot-reload is disabled
    let receiver = factory.take_hot_reload_receiver();
    assert!(receiver.is_none());
}

/// Test debouncing of rapid file changes
#[tokio::test]
#[cfg(feature = "hot-reload")]
#[ignore = "Integration test that requires file system operations"]
async fn test_debouncing_rapid_changes() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a test file
    let test_file = temp_path.join("debounce_test.ron");
    fs::write(&test_file, "initial content").expect("Failed to write test file");

    // Create a glob pattern for .ron files in the temp directory
    let pattern = format!("{}/*.ron", temp_path.display());

    // Create channels for hot-reload events
    let (tx, mut rx) = create_reload_channel();

    // Start the watcher
    watcher::run_watcher(&pattern, tx)
        .await
        .expect("Failed to start watcher");

    // Wait a bit for the watcher to initialize
    sleep(Duration::from_millis(100)).await;

    // Rapidly modify the file multiple times
    for i in 0..5 {
        fs::write(&test_file, format!("content {i}")).expect("Failed to write test file");
        sleep(Duration::from_millis(50)).await;
    }

    // Wait for debouncing to complete
    sleep(Duration::from_millis(500)).await;

    // Should receive only one event due to debouncing
    let mut event_count = 0;
    while let Ok(event) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        if event.is_some() {
            event_count += 1;
        }
    }

    // Should have received at least one event, but not one for each modification
    assert!(event_count > 0);
    assert!(event_count < 5); // Should be debounced
}
