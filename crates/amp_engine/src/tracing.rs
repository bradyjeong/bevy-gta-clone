//! Performance tracing infrastructure for Amp engine
//!
//! This module provides tracing capabilities for performance analysis,
//! including Chrome DevTools integration and span instrumentation.
//!
//! Feature-gated behind `perf_trace` - disabled by default to avoid overhead.

#[cfg(feature = "perf_trace")]
use tracing_chrome::ChromeLayerBuilder;
#[cfg(feature = "perf_trace")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for performance tracing
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Enable Chrome DevTools output
    pub chrome_output: bool,
    /// Path for Chrome trace file
    pub chrome_path: String,
    /// Enable console output
    pub console_output: bool,
    /// Filter level for tracing
    pub filter_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            chrome_output: true,
            chrome_path: "./trace.json".to_string(),
            console_output: false,
            filter_level:
                "amp_engine=trace,amp_render=trace,amp_gameplay=trace,gameplay_factory=trace"
                    .to_string(),
        }
    }
}

/// Initialize performance tracing with the given configuration
#[cfg(feature = "perf_trace")]
pub fn init_tracing(config: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let registry = tracing_subscriber::registry();

    // Filter layer
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(&config.filter_level))
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("trace"));

    // Simple approach - just use Chrome layer for now
    if config.chrome_output {
        let (chrome_layer, _guard) = ChromeLayerBuilder::new().file(&config.chrome_path).build();
        registry.with(filter_layer).with(chrome_layer).init();
        std::mem::forget(_guard); // Keep the guard alive for the duration of the program
    } else {
        registry.with(filter_layer).init();
    }

    println!("Performance tracing initialized");
    if config.chrome_output {
        println!("Chrome trace output: {}", config.chrome_path);
    }

    Ok(())
}

/// Initialize performance tracing with default configuration
#[cfg(feature = "perf_trace")]
pub fn init_default_tracing() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing(&TracingConfig::default())
}

/// No-op initialization when tracing is disabled
#[cfg(not(feature = "perf_trace"))]
pub fn init_tracing(_config: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// No-op initialization when tracing is disabled
#[cfg(not(feature = "perf_trace"))]
pub fn init_default_tracing() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Macro for creating performance spans
#[macro_export]
macro_rules! perf_span {
    ($name:expr) => {
        #[cfg(feature = "perf_trace")]
        let _span = tracing::trace_span!($name).entered();
    };
    ($name:expr, $($field:tt)*) => {
        #[cfg(feature = "perf_trace")]
        let _span = tracing::trace_span!($name, $($field)*).entered();
    };
}

/// Macro for creating performance events
#[macro_export]
macro_rules! perf_event {
    ($name:expr) => {
        #[cfg(feature = "perf_trace")]
        tracing::trace!($name);
    };
    ($name:expr, $($field:tt)*) => {
        #[cfg(feature = "perf_trace")]
        tracing::trace!($name, $($field)*);
    };
}

/// Macro for timing code blocks
#[macro_export]
macro_rules! perf_time {
    ($name:expr, $body:expr) => {
        #[cfg(feature = "perf_trace")]
        {
            let _span = tracing::trace_span!($name).entered();
            $body
        }
        #[cfg(not(feature = "perf_trace"))]
        {
            $body
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert!(config.chrome_output);
        assert_eq!(config.chrome_path, "./trace.json");
        assert!(!config.console_output);
        assert!(config.filter_level.contains("amp_engine=trace"));
    }

    #[test]
    fn test_tracing_macros() {
        // Test that macros compile without feature flag
        perf_span!("test_span");
        perf_event!("test_event");
        let result = perf_time!("test_time", 42);
        assert_eq!(result, 42);
    }
}
