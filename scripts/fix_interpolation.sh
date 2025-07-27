#!/bin/bash

# Fix velocity type conflicts in bundles
sed -i '' 's/pub velocity: Velocity,/pub velocity: CharacterVelocity,/g' crates/amp_gameplay/src/character/bundles.rs
sed -i '' 's/velocity: Velocity::default()/velocity: CharacterVelocity::default()/g' crates/amp_gameplay/src/character/bundles.rs

# Fix velocity conflicts in movement systems  
sed -i '' 's/use crate::character::components::\*/use crate::character::components::\*;\nuse crate::character::components::Velocity as CharacterVelocity;/g' crates/amp_gameplay/src/character/systems/movement.rs
sed -i '' 's/Query<\&mut Velocity>/Query<\&mut CharacterVelocity>/g' crates/amp_gameplay/src/character/systems/movement.rs
sed -i '' 's/\&mut Velocity,/\&mut CharacterVelocity,/g' crates/amp_gameplay/src/character/systems/movement.rs

echo "Applied velocity type fixes"
