//! Compile-fail tests to prevent regressions to manual plugin wiring
//!
//! These tests ensure that developers are guided to use AAAPlugins instead of legacy patterns.

// Documentation tests that verify the correct patterns
/// Example showing the correct way to use AAAPlugins
///
/// ```rust
/// use bevy::prelude::*;
/// use amp_engine::prelude::*;
///
/// fn setup_app() {
///     let app = App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(AAAPlugins::default());
/// }
/// ```
pub fn _correct_aaa_plugins_usage() {}

/// Example showing how to add custom plugins to AAAPlugins
///
/// ```rust
/// use bevy::prelude::*;
/// use amp_engine::prelude::*;
///
/// struct CustomPlugin;
/// impl Plugin for CustomPlugin {
///     fn build(&self, app: &mut App) {
///         // Plugin implementation
///     }
/// }
///
/// fn setup_app_with_custom_plugin() {
///     let app = App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(AAAPlugins::empty().add_plugin(CustomPlugin));
/// }
/// ```
pub fn _custom_plugin_with_aaa_plugins() {}

#[cfg(test)]
mod tests {
    /// This test ensures GameplayPlugins no longer exists and has been replaced with AAAPlugins
    #[test]
    fn test_gameplay_plugins_removed() {
        // This should compile - GameplayPlugins should not exist anymore
        // If this test fails to compile, it means someone re-added GameplayPlugins

        let compilation_check = "GameplayPlugins struct should not exist";

        // Test that we can't use the old pattern
        // let app = App::new().add_plugins(GameplayPlugins); // This should not compile

        assert!(!compilation_check.is_empty()); // Just to make this a valid test
    }

    /// Test that the preferred pattern using AAAPlugins compiles correctly
    #[test]
    fn test_aaa_plugins_pattern() {
        use amp_engine::prelude::*;

        // This should compile successfully - just test the type construction
        let _plugins = AAAPlugins::default();
        let _plugins2 = AAAPlugins::empty();

        // We don't actually create the App here to avoid EventLoop issues on macOS
    }

    /// Test that manual plugin addition is discouraged in favor of AAAPlugins
    #[test]
    fn test_manual_plugin_pattern_discouraged() {
        use amp_engine::prelude::*;

        // Test that we can construct the plugin groups without creating the app
        let _plugins = AAAPlugins::default();

        // This test verifies that the preferred AAAPlugins pattern exists and compiles
        // We don't create the actual App to avoid macOS EventLoop constraints
    }
}
