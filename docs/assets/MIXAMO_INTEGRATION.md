# Mixamo Character Integration Guide

This guide walks you through the complete workflow of adding Mixamo characters to the game, from download to in-game animation.

## Overview

The humanoid character system supports Mixamo characters with a standardized bone mapping and animation state machine. Characters are configured through RON asset files that define animation clips and transition parameters.

## Workflow: From Mixamo to In-Game Character

### Step 1: Download Mixamo Character

1. Go to [Adobe Mixamo](https://www.mixamo.com)
2. Select a character (recommend humanoid T-pose models)
3. Download in FBX format with these settings:
   - Format: FBX Binary (.fbx)
   - Skin: With Skin
   - Keyframe Reduction: None
   - Frame Rate: 30 FPS
   - Pose: T-Pose

### Step 2: Download Animations

Download the following core animations for basic locomotion:
- **Idle**: "Breathing Idle" or "Standing Idle"
- **Walk**: "Walking" or "Catwalk Walk"  
- **Run**: "Running" or "Fast Run"
- **Sprint**: "Sprint" or "Run Forward"
- **Jump**: "Jump" or "Standing Jump"
- **Fall**: "Falling Idle" or "Falling"
- **Land**: "Hard Landing" or "Falling To Landing"
- **Turn**: "Left Turn" or "Turn 90 Left"

**Download Settings:**
- Format: FBX Binary (.fbx)
- Skin: Without Skin
- Keyframe Reduction: Uniform (for smaller file sizes)
- Frame Rate: 30 FPS
- Root Motion: In Place (for stationary animations)

### Step 3: Convert to glTF (Recommended)

For better performance and compatibility with Bevy:

```bash
# Install glTF converter (if not already installed)
npm install -g gltf-pipeline

# Convert FBX files to glTF
gltf-pipeline -i character.fbx -o character.glb
gltf-pipeline -i idle.fbx -o idle.glb
gltf-pipeline -i walking.fbx -o walking.glb
# ... convert all animations
```

### Step 4: Organize Assets

Place files in the assets directory:

```
assets/
├── characters/
│   ├── models/
│   │   └── mixamo_character.glb      # Character mesh + rig
│   ├── animations/
│   │   ├── idle.glb                  # Animation clips
│   │   ├── walking.glb
│   │   ├── running.glb
│   │   ├── sprinting.glb
│   │   ├── jumping.glb
│   │   ├── falling.glb
│   │   ├── landing.glb
│   │   └── turning.glb
│   └── mixamo_character.animset.ron  # Animation configuration
```

### Step 5: Create AnimationSet Configuration

Create a new `.animset.ron` file for your character:

```ron
// Example: assets/characters/mixamo_character.animset.ron
AnimationSet(
    animations: {
        Idle: AnimationClip(
            path: "characters/animations/idle.glb#Animation0",
            speed: 1.0,
            loop_mode: true,
            blend_in_duration: 0.3,
            blend_out_duration: 0.3,
        ),
        Walk: AnimationClip(
            path: "characters/animations/walking.glb#Animation0",
            speed: 1.0,
            loop_mode: true,
            blend_in_duration: 0.2,
            blend_out_duration: 0.2,
        ),
        // ... add other animations
    },
    walk_threshold: 0.1,
    run_threshold: 3.0,
    sprint_threshold: 6.0,
    velocity_hysteresis: 0.2,
    state_change_cooldown: 0.1,
)
```

### Step 6: Spawn Character in Code

```rust
use amp_gameplay::character::{bundles::PlayerBundle, components::*};

fn spawn_mixamo_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load character model and animations
    let character_scene: Handle<Scene> = asset_server.load("characters/models/mixamo_character.glb#Scene0");
    let animation_set: Handle<AnimationSet> = asset_server.load("characters/mixamo_character.animset.ron");

    // Spawn complete character
    commands.spawn((
        PlayerBundle {
            character: CharacterBundle {
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                },
                animation_set,
                ..default()
            },
            player: Player,
        },
        SceneBundle {
            scene: character_scene,
            ..default()
        },
        Name::new("Mixamo Player"),
    ));
}
```

## Bone Mapping

The system automatically maps Mixamo bone names to the `HumanoidBone` enum:

### Core Bones (Required)
- **Hips** → HumanoidBone::Hips (root bone)
- **Spine**, **Spine1**, **Spine2** → HumanoidBone::Spine variants
- **Neck**, **Head** → HumanoidBone::Neck, HumanoidBone::Head

### Limbs (Required for full IK)
- **LeftShoulder**, **LeftArm**, **LeftForeArm**, **LeftHand**
- **RightShoulder**, **RightArm**, **RightForeArm**, **RightHand**  
- **LeftUpLeg**, **LeftLeg**, **LeftFoot**, **LeftToeBase**
- **RightUpLeg**, **RightLeg**, **RightFoot**, **RightToeBase**

### Fingers (Optional)
- **LeftHandThumb1-3**, **LeftHandIndex1-3**, etc.
- **RightHandThumb1-3**, **RightHandIndex1-3**, etc.

## Animation State Machine

The system includes an automatic state machine based on character velocity:

### States
1. **Idle** (0 - 0.1 m/s): Standing still
2. **Walk** (0.1 - 3.0 m/s): Slow movement
3. **Run** (3.0 - 6.0 m/s): Medium movement
4. **Sprint** (6.0+ m/s): Fast movement
5. **Jump**: Triggered by input
6. **Fall**: When not grounded
7. **Land**: Transition from fall to ground

### Customization
Adjust thresholds in the AnimationSet:
```ron
walk_threshold: 0.1,        // Start walking at this speed
run_threshold: 3.0,         // Start running at this speed  
sprint_threshold: 6.0,      // Start sprinting at this speed
velocity_hysteresis: 0.2,   // Prevents state oscillation
state_change_cooldown: 0.1, // Minimum time between transitions
```

## Performance Optimization

### File Size Optimization
- Use keyframe reduction in Mixamo downloads
- Convert to glTF/GLB for smaller files
- Use texture compression for character materials

### Runtime Optimization
- The bone mapping system uses O(1) HashMap lookups
- Animation blending is optimized for minimal CPU usage
- State machine prevents unnecessary animation switches

## Troubleshooting

### Character Not Animating
1. Check that animation paths in `.animset.ron` are correct
2. Verify animations contain `#Animation0` or correct animation index
3. Ensure character has `AnimationPlayer` component (added automatically)

### Bone Mapping Issues  
1. Check console for bone mapping warnings
2. Verify Mixamo character uses standard bone names
3. Some custom rigs may need manual bone name mapping

### Performance Issues
1. Reduce animation clip length if not looping
2. Use LOD system for distant characters
3. Limit number of active characters with complex animations

### Animation Glitches
1. Check blend durations aren't too short (<0.1s)
2. Verify animation loops correctly for looping states
3. Adjust velocity thresholds to prevent rapid state changes

## Advanced Features

### Multiple Character Types
Create different `.animset.ron` files for different character archetypes:
- `player_character.animset.ron` - Player with full movement
- `npc_civilian.animset.ron` - NPCs with limited animations
- `npc_guard.animset.ron` - NPCs with combat animations

### Custom Animations
Add custom states by extending the `LocomotionState` enum and animation system:
```rust
// In your custom character plugin
pub enum CustomLocomotionState {
    Combat,
    Climbing,
    Swimming,
    // ... etc
}
```

### IK and Procedural Animation
The bone mapping system supports future IK systems:
- Foot IK for uneven terrain
- Hand IK for object interaction
- Look-at targeting for head tracking

## Integration with Existing Systems

The humanoid character system integrates with:
- **Physics**: Characters use Rapier3D collision detection
- **Input**: Standard input system for movement and actions
- **Camera**: Third-person camera follows player characters
- **Audio**: Footstep and voice audio triggered by animation events
- **Game Logic**: State machine can trigger gameplay events

## Next Steps

1. **Download your first Mixamo character** and follow this workflow
2. **Test with the demo**: Run `cargo run --example humanoid_character_demo`
3. **Customize animations**: Adjust blend times and thresholds for your game feel
4. **Add custom characters**: Create multiple character types for variety
5. **Extend the system**: Add combat, interaction, or other custom states

For additional help, check the example code in `examples/humanoid_character_demo.rs` or review the character system source in `crates/amp_gameplay/src/character/`.
