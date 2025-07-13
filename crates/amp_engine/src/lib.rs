pub mod prelude;
pub mod spatial;
pub mod world;

#[cfg(feature = "bevy16")]
pub mod plugins;

#[cfg(feature = "bevy16")]
pub mod gpu;

#[cfg(feature = "bevy16")]
pub mod assets;

#[cfg(all(test, feature = "bevy16"))]
pub mod test_utils;
