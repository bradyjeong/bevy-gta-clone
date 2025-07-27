//! Demonstration of facade crate migration
//!
//! This example shows how to use amp_foundation instead of individual crates.

use amp_foundation::prelude::{Error, InterpolatedTransform, PhysicsSets, Player, Result};
use bevy::prelude::*;

fn main() {
    println!("Facade migration demo");

    // These imports now work through amp_foundation::prelude
    let _error: Error = Error::internal("Test error");
    let _result: Result<()> = Ok(());

    println!("✅ Successfully using facade imports!");
    println!("✅ Error type: available");
    println!("✅ Result type: available");
    println!("✅ InterpolatedTransform: available");
    println!("✅ PhysicsSets: available");
    println!("✅ Player component: available");
}
