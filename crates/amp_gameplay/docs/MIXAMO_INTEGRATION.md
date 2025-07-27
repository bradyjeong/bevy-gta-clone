# Mixamo Character Asset Integration

This document describes the asset integration foundation for Mixamo characters in the amp_gameplay character system.

## Overview

The character system provides a complete asset loading pipeline for Mixamo characters, including:

- Automatic glTF skeleton processing
- RON-based animation configuration
- Easy character spawning with bundles
- Physics integration
- Scale and orientation handling

## Asset Pipeline Architecture

### 1. RigBuilder System

The `RigBuilder` system automatically processes imported glTF models:

```rust
use amp_gameplay::character::{
    systems::asset_loading::LoadCharacterAsset,
    bundles::CharacterBundle,
};

// Request character loading
commands.spawn((
    LoadCharacterAsset::new("models/character.glb", "player"),
    // ... other components
));
```

**Features:**
- Scans glTF skeleton hierarchy
- Maps bone names to `HumanoidBone` enum
- Populates `HumanoidRig` component automatically
- Handles Mixamo naming conventions (`mixamorig:*`)
- Attaches `AnimationPlayer` component

### 2. AnimationSet Asset Loading

Animation sets are defined in RON files and loaded as assets:

```ron
// animations/character.animset.ron
(
    character_type: "mixamo",
    clips: {
        "idle": "animations/idle.glb#Animation0",
        "walk": "animations/walking.glb#Animation0",
        "run": "animations/running.glb#Animation0",
        // ... more animations
    },
    blend_weights: Some({
        "idle": 1.0,
        "walk": 1.0,
        // ... weights for blending
    }),
    speeds: Some({
        "idle": 1.0,
        "walk": 1.2,
        "run": 1.5,
        // ... playback speeds
    }),
    transitions: Some({
        "idle": 0.3,
        "walk": 0.2,
        // ... transition durations
    }),
)
```

### 3. PlayerBundle for Easy Spawning

Complete character spawning with minimal code:

```rust
use amp_gameplay::character::bundles::CharacterBundle;

// Method 1: Basic spawning
let character = CharacterBundle::spawn_player(
    &mut commands,
    "models/mixamo_character.glb",
    animation_set_handle,
);

// Method 2: With custom scale (Mixamo models are often in cm)
let character = CharacterBundle::spawn_player_with_scale(
    &mut commands,
    "models/mixamo_character.glb", 
    animation_set_handle,
    0.01, // Convert cm to meters
);

// Method 3: Physics-enabled character
#[cfg(feature = "rapier3d_030")]
let physics_character = PhysicsCharacterBundle::spawn_physics_player(
    &mut commands,
    "models/mixamo_character.glb",
    animation_set_handle,
);
```

## Workflow for Importing Mixamo Characters

### Step 1: Export from Mixamo

1. Download character model as FBX
2. Download animations as FBX (same character)
3. Convert to glTF using Blender or other tools
4. Place in `assets/models/` and `assets/animations/` directories

### Step 2: Create Animation Configuration

Create a `.animset.ron` file:

```ron
(
    character_type: "my_character",
    clips: {
        "idle": "animations/my_character/idle.glb#Animation0",
        "walk": "animations/my_character/walk.glb#Animation0",
        // ... add all your animations
    },
    blend_weights: Some({
        "idle": 1.0,
        "walk": 1.0,
        // ... adjust blend weights as needed
    }),
    speeds: Some({
        "idle": 1.0,
        "walk": 1.2,
        // ... adjust playback speeds
    }),
)
```

### Step 3: Spawn Character in Code

```rust
fn spawn_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load animation set
    let animation_set: Handle<AnimationSet> = asset_server.load("animations/my_character.animset.ron");
    
    // Spawn character
    let character = CharacterBundle::spawn_player_with_scale(
        &mut commands,
        "models/my_character.glb",
        animation_set,
        0.01, // Scale from cm to m if needed
    );
}
```

## Advanced Usage

### Custom Animation Sets

Create animation sets programmatically:

```rust
fn create_custom_animation_set(
    mut animation_sets: ResMut<Assets<AnimationSet>>,
    asset_server: Res<AssetServer>,
) -> Handle<AnimationSet> {
    let mut set = AnimationSet::new("custom");
    
    set.add_clip(
        Locomotion::Idle,
        asset_server.load("animations/custom_idle.glb#Animation0"),
        1.0,
    );
    
    animation_sets.add(set)
}
```

### Asset Registry

Track loaded assets by character type:

```rust
fn use_asset_registry(
    registry: Res<CharacterAssetRegistry>,
    mut commands: Commands,
) {
    if let Some(animation_set) = registry.get_animation_set("mixamo") {
        // Use the loaded animation set
        let character = CharacterBundle::spawn_player(
            &mut commands,
            "models/character.glb",
            animation_set.clone(),
        );
    }
}
```

### Loading State Monitoring

Monitor asset loading progress:

```rust
fn monitor_loading(
    loading_query: Query<&CharacterAssetLoading>,
) {
    for loading in loading_query.iter() {
        match loading.stage {
            LoadingStage::LoadingGltf => info!("Loading glTF..."),
            LoadingStage::ProcessingSkeleton => info!("Processing skeleton..."),
            LoadingStage::SettingUpAnimation => info!("Setting up animations..."),
            LoadingStage::Complete => info!("Loading complete!"),
        }
    }
}
```

## Technical Details

### Bone Mapping

The system automatically maps Mixamo bone names to the `HumanoidBone` enum:

- `mixamorig:Hips` → `HumanoidBone::Hips`
- `mixamorig:LeftArm` → `HumanoidBone::LeftArm`
- `mixamorig:RightLeg` → `HumanoidBone::RightLeg`
- etc.

### Scale Handling

Mixamo models are often exported in centimeters. The system provides scale correction:

```rust
// Automatic scale application during rig building
let rig = HumanoidRig::from_skeleton(skeleton_entity, &bone_names);
rig.scale = 0.01; // Convert cm to meters

// Scale is applied to the transform automatically
```

### Physics Integration

When using the physics features:

```rust
#[cfg(feature = "rapier3d_030")]
let physics_character = PhysicsCharacterBundle::spawn_physics_player(
    &mut commands,
    "models/character.glb",
    animation_set,
);
```

This automatically sets up:
- Kinematic rigid body
- Capsule collider
- Character controller
- Collision groups

## Example Project Structure

```
assets/
├── models/
│   ├── mixamo_character.glb
│   └── mixamo_character_scaled.glb
├── animations/
│   ├── mixamo/
│   │   ├── idle.glb
│   │   ├── walking.glb
│   │   ├── running.glb
│   │   └── jump.glb
│   └── mixamo_default.animset.ron
└── ...

src/
├── main.rs
└── ...
```

## Performance Considerations

- Asset loading is asynchronous and non-blocking
- Bone mapping is done once during loading
- Animation sets are shared between characters of the same type
- Scale corrections are applied efficiently during rig building

## Troubleshooting

### Common Issues

1. **Character not appearing**: Check glTF file path and ensure it's in the assets directory
2. **Wrong scale**: Adjust the scale parameter, Mixamo models often need 0.01 scale
3. **Animations not playing**: Verify animation clip paths in the .animset.ron file
4. **Bone mapping issues**: Check bone names in the glTF file match Mixamo conventions

### Debugging

Enable debug logging:

```rust
// Add to main function
app.insert_resource(bevy::log::LogSettings {
    level: bevy::log::Level::DEBUG,
    ..default()
});
```

Check loading progress:

```rust
fn debug_loading(
    loading_query: Query<&CharacterAssetLoading>,
) {
    for loading in loading_query.iter() {
        debug!("Character loading stage: {:?}", loading.stage);
    }
}
```

This foundation provides a robust, extensible system for integrating Mixamo characters with minimal boilerplate code while maintaining flexibility for advanced use cases.
