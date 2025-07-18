// Batch processing macros for complex optimizations

/// Macro for creating batch job handlers
#[macro_export]
macro_rules! batch_job_handler {
    ($name:ident, $input:ty, $output:ty, $handler:expr) => {
        pub fn $name(input: $input) -> $output {
            $handler(input)
        }
    };
}

/// Macro for performance monitoring
#[macro_export]
macro_rules! monitor_performance {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::debug!("Performance: {} took {:?}", $name, duration);
        result
    }};
}

pub use {batch_job_handler, monitor_performance};
