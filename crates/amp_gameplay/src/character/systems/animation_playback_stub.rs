// Temporary stub for animation playback system
// This will be replaced with full Bevy 0.16.1 AnimationGraph integration

use crate::character::components::{CharacterAnimations, HumanoidRig, LocomotionState};
use bevy::prelude::*;

/// Stub system for animation playback until full Bevy 0.16.1 integration
pub fn apply_animation_playback_stub(
    query: Query<(Entity, &LocomotionState, &CharacterAnimations, &HumanoidRig)>,
) {
    for (entity, locomotion_state, _animations, _rig) in query.iter() {
        debug!(
            "Animation playback stub for entity {:?}: locomotion_state={:?}",
            entity, locomotion_state.current
        );
    }
}
