//! Hot-reload functionality for file watching and automatic prefab reloading
//!
//! This module provides file watching capabilities that trigger prefab reloads
//! when files are modified, created, or deleted.

use std::path::{Path, PathBuf};

use amp_core::Error;
use bevy::prelude::*;

/// Events that can trigger a hot-reload
#[derive(Debug, Clone, PartialEq)]
pub enum HotReloadEvent {
    /// A file was created
    Created(PathBuf),
    /// A file was modified
    Modified(PathBuf),
    /// A file was deleted
    Deleted(PathBuf),
}

impl HotReloadEvent {
    /// Get the path associated with this event
    pub fn path(&self) -> &Path {
        match self {
            HotReloadEvent::Created(path) => path,
            HotReloadEvent::Modified(path) => path,
            HotReloadEvent::Deleted(path) => path,
        }
    }

    /// Check if this event represents a file deletion
    pub fn is_deletion(&self) -> bool {
        matches!(self, HotReloadEvent::Deleted(_))
    }
}

/// Handle for controlling a file watcher
pub struct WatcherHandle {
    #[cfg(feature = "hot-reload")]
    _handle: tokio::task::JoinHandle<()>,
    #[cfg(not(feature = "hot-reload"))]
    _phantom: std::marker::PhantomData<()>,
}

impl WatcherHandle {
    /// Create a new watcher handle
    #[cfg(feature = "hot-reload")]
    pub fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self { _handle: handle }
    }

    /// Create a stub handle when hot-reload is disabled
    #[cfg(not(feature = "hot-reload"))]
    pub fn stub() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Channel for receiving hot-reload events
#[cfg(feature = "hot-reload")]
#[derive(Resource)]
pub struct HotReloadReceiver {
    receiver: tokio::sync::mpsc::UnboundedReceiver<HotReloadEvent>,
}

#[cfg(feature = "hot-reload")]
impl HotReloadReceiver {
    /// Create a new HotReloadReceiver
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<HotReloadEvent>) -> Self {
        Self { receiver }
    }

    /// Try to receive a hot-reload event
    pub fn try_recv(&mut self) -> Result<HotReloadEvent, tokio::sync::mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Receive a hot-reload event (async)
    pub async fn recv(&mut self) -> Option<HotReloadEvent> {
        self.receiver.recv().await
    }
}

/// Channel for sending hot-reload events
#[cfg(feature = "hot-reload")]
pub type HotReloadSender = tokio::sync::mpsc::UnboundedSender<HotReloadEvent>;

/// Dummy types when hot-reload is disabled
#[cfg(not(feature = "hot-reload"))]
#[derive(Resource)]
pub struct HotReloadReceiver {
    _phantom: std::marker::PhantomData<HotReloadEvent>,
}

#[cfg(not(feature = "hot-reload"))]
impl HotReloadReceiver {
    /// Stub implementation that always returns an error
    pub fn try_recv(&mut self) -> Result<HotReloadEvent, &'static str> {
        Err("Hot-reload feature not enabled")
    }
}

#[cfg(not(feature = "hot-reload"))]
pub type HotReloadSender = std::marker::PhantomData<HotReloadEvent>;

/// Create a hot-reload channel pair
#[cfg(feature = "hot-reload")]
pub fn create_reload_channel() -> (HotReloadSender, HotReloadReceiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (tx, HotReloadReceiver::new(rx))
}

/// Create a stub channel when hot-reload is disabled
#[cfg(not(feature = "hot-reload"))]
pub fn create_reload_channel() -> (HotReloadSender, HotReloadReceiver) {
    // Return dummy phantoms
    (
        std::marker::PhantomData,
        HotReloadReceiver {
            _phantom: std::marker::PhantomData,
        },
    )
}

/// File watcher implementation
#[cfg(feature = "hot-reload")]
pub mod watcher {
    use super::*;
    use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use tokio::time::{sleep, Duration};

    /// Run a file watcher that monitors files matching a glob pattern
    pub async fn run_watcher(glob_pattern: &str, reload_tx: HotReloadSender) -> Result<(), Error> {
        let pattern = glob_pattern.to_string();

        // Expand the glob pattern to get parent directories to watch
        let watch_dirs = expand_glob_to_watch_dirs(&pattern)?;

        // Run the watcher directly
        run_watcher_loop(watch_dirs, pattern, reload_tx).await
    }

    /// Get the parent directories that need to be watched for a glob pattern
    fn expand_glob_to_watch_dirs(pattern: &str) -> Result<Vec<PathBuf>, Error> {
        // For patterns like "/assets/**/*.ron", we need to watch "/assets"
        // For patterns like "assets/prefabs/*.ron", we need to watch "assets/prefabs"
        let path = Path::new(pattern);
        let mut dirs = Vec::new();

        // Walk up the path until we find the first component with wildcards
        let mut current = path;
        while let Some(parent) = current.parent() {
            let parent_str = parent.to_str().unwrap_or("");
            if parent_str.contains('*') || parent_str.contains('?') {
                current = parent;
            } else {
                // This is a concrete directory we can watch
                if parent.exists() {
                    dirs.push(parent.to_path_buf());
                }
                break;
            }
        }

        // If we didn't find any concrete directories, watch the current directory
        if dirs.is_empty() {
            dirs.push(PathBuf::from("."));
        }

        Ok(dirs)
    }

    /// Main watcher loop
    async fn run_watcher_loop(
        watch_dirs: Vec<PathBuf>,
        pattern: String,
        reload_tx: HotReloadSender,
    ) -> Result<(), Error> {
        let (tx, rx) = mpsc::channel();

        // Configure the watcher with a 500ms debounce
        let mut watcher = RecommendedWatcher::new(
            tx,
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| Error::resource_load("file watcher", e.to_string()))?;

        // Start watching the directories
        for dir in &watch_dirs {
            watcher
                .watch(dir, RecursiveMode::Recursive)
                .map_err(|e| Error::resource_load("file watcher", e.to_string()))?;
            log::info!("Watching directory: {}", dir.display());
        }

        // Process events
        let mut debounce_map = std::collections::HashMap::new();
        let debounce_delay = Duration::from_millis(250);

        loop {
            // Handle notify events
            while let Ok(event) = rx.try_recv() {
                if let Ok(event) = event {
                    process_notify_event(&event, &pattern, &mut debounce_map).await;
                }
            }

            // Process debounced events
            let now = std::time::Instant::now();
            let mut to_send = Vec::new();
            debounce_map.retain(|path, &mut last_time| {
                if now.duration_since(last_time) >= debounce_delay {
                    to_send.push(path.clone());
                    false
                } else {
                    true
                }
            });

            // Send debounced events
            for path in to_send {
                if path.exists() {
                    let event = HotReloadEvent::Modified(path.clone());
                    if reload_tx.send(event).is_err() {
                        log::warn!("Hot-reload channel closed, stopping watcher");
                        break;
                    }
                } else {
                    let event = HotReloadEvent::Deleted(path.clone());
                    if reload_tx.send(event).is_err() {
                        log::warn!("Hot-reload channel closed, stopping watcher");
                        break;
                    }
                }
            }

            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(50)).await;
        }
    }

    /// Process a notify event and update debounce map
    async fn process_notify_event(
        event: &Event,
        pattern: &str,
        debounce_map: &mut std::collections::HashMap<PathBuf, std::time::Instant>,
    ) {
        for path in &event.paths {
            // Check if the path matches our glob pattern
            if !path_matches_pattern(path, pattern) {
                continue;
            }

            match event.kind {
                EventKind::Create(_) => {
                    // Debounce create events
                    debounce_map.insert(path.clone(), std::time::Instant::now());
                }
                EventKind::Modify(_) => {
                    // Debounce modify events
                    debounce_map.insert(path.clone(), std::time::Instant::now());
                }
                EventKind::Remove(_) => {
                    // Don't debounce remove events - send immediately
                    debounce_map.remove(path);
                }
                _ => {}
            }
        }
    }

    /// Check if a path matches the glob pattern
    fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
        // Use glob matching to check if the path matches the pattern
        match glob::Pattern::new(pattern) {
            Ok(glob_pattern) => {
                if let Some(path_str) = path.to_str() {
                    glob_pattern.matches(path_str)
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

/// Stub watcher implementation when hot-reload is disabled
#[cfg(not(feature = "hot-reload"))]
pub mod watcher {
    use super::*;

    /// Stub implementation that returns immediately
    pub async fn run_watcher(
        _glob_pattern: &str,
        _reload_tx: HotReloadSender,
    ) -> Result<WatcherHandle, Error> {
        log::warn!("Hot-reload watcher requested but hot-reload feature is disabled");
        Ok(WatcherHandle::stub())
    }
}

/// Bevy system for processing hot-reload events
#[cfg(feature = "hot-reload")]
pub fn process_hot_reload_events(
    mut receiver: ResMut<HotReloadReceiver>,
    // Add other system parameters as needed for prefab reloading
) {
    // Process all pending events
    while let Ok(event) = receiver.try_recv() {
        match event {
            HotReloadEvent::Created(path) => {
                log::info!("Hot-reload: File created: {}", path.display());
                // TODO: Load new prefab
            }
            HotReloadEvent::Modified(path) => {
                log::info!("Hot-reload: File modified: {}", path.display());
                // TODO: Reload existing prefab
            }
            HotReloadEvent::Deleted(path) => {
                log::info!("Hot-reload: File deleted: {}", path.display());
                // TODO: Remove prefab from registry
            }
        }
    }
}

/// Stub system when hot-reload is disabled
#[cfg(not(feature = "hot-reload"))]
pub fn process_hot_reload_events() {
    // No-op when hot-reload is disabled
}
