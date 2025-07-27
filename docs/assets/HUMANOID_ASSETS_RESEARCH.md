# Humanoid glTF Asset Research & Integration Plan

## Research Summary

### Recommended Asset Sources

#### 1. **Mixamo (Adobe) - PRIMARY RECOMMENDATION**
- **Status**: Free, extensive library
- **Models**: Diverse humanoid characters with proper rigging
- **Animations**: Thousands including idle, walk, run, sprint, jump_start, fall_loop, land, turn
- **Formats**: FBX, glTF/GLB, Collada (DAE), USD
- **Licensing**: Free for commercial and non-commercial use
- **Bevy Compatibility**: ✅ Excellent (glTF native support)
- **Quality**: Professional motion capture animations
- **Workflow**: Upload custom characters for auto-rigging or use provided models

#### 2. **Kay Lousberg (itch.io) - SECONDARY RECOMMENDATION**
- **Status**: Free and paid lowpoly character packs
- **Models**: KayKit Character Packs (Adventurers, Skeletons, etc.)
- **Animations**: Basic set included (shooting, fighting, locomotion)
- **Formats**: FBX and glTF explicitly supported
- **Licensing**: Game asset license (commercial use allowed)
- **Bevy Compatibility**: ✅ Good (designed for game engines)
- **Quality**: Stylized lowpoly art style, consistent across packs

#### 3. **Free3D.com - TERTIARY OPTION**
- **Status**: Mixed free/paid content
- **Models**: Various rigged humanoid models available
- **Animations**: Limited animated models (Nathan Walking, Sophia Idling, Manuel Dancing)
- **Formats**: FBX, OBJ, Blend, others
- **Licensing**: Varies by asset
- **Bevy Compatibility**: ⚠️ Requires conversion to glTF
- **Quality**: Variable

#### 4. **OpenGameArt & Kenney.nl**
- **Status**: Research incomplete - require direct browsing
- **Expected**: Basic game assets, likely limited character selection
- **Recommendation**: Use as fallback sources

## Recommended Asset Integration Plan

### Phase 1: Quick Prototype (Mixamo)
1. **Download Mixamo Assets**:
   - 1-2 basic humanoid characters (different body types)
   - Essential animation set: idle, walk, run, turn_left, turn_right
   - Export as glTF format for direct Bevy compatibility

2. **Asset Structure**:
```
assets/
├── characters/
│   ├── humanoid_basic/
│   │   ├── model.gltf
│   │   ├── animations/
│   │   │   ├── idle.gltf
│   │   │   ├── walk.gltf
│   │   │   ├── run.gltf
│   │   │   └── sprint.gltf
│   │   └── textures/
│   └── humanoid_female/
│       └── [similar structure]
└── README.md (licensing attribution)
```

### Phase 2: Extended Animation Set
Add advanced animations:
- `jump_start.gltf`
- `fall_loop.gltf` 
- `land.gltf`
- `turn_left.gltf`
- `turn_right.gltf`
- `crouch_idle.gltf`
- `crouch_walk.gltf`

### Phase 3: Professional Assets (Optional)
Consider Kay Lousberg packs for consistent art style and game-ready optimization.

## amp_gameplay Integration Approach

### 1. Character Component System
```rust
// In amp_gameplay/src/character/mod.rs
#[derive(Component, Default)]
pub struct HumanoidCharacter {
    pub model_handle: Handle<Scene>,
    pub animation_handles: HashMap<AnimationType, Handle<AnimationClip>>,
    pub current_animation: AnimationType,
    pub animation_player: Entity,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum AnimationType {
    Idle,
    Walk,
    Run,
    Sprint,
    JumpStart,
    FallLoop,
    Land,
    TurnLeft,
    TurnRight,
}
```

### 2. Asset Loading System
```rust
// Character asset loading plugin
pub struct CharacterAssetsPlugin;

impl Plugin for CharacterAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CharacterAssets>()
            .add_systems(PreUpdate, load_character_assets)
            .add_systems(Update, (
                spawn_characters,
                animate_characters,
                transition_animations,
            ));
    }
}
```

### 3. Animation State Machine
```rust
#[derive(Component)]
pub struct CharacterAnimationState {
    pub state_machine: StateMachine<AnimationType>,
    pub transition_timer: Timer,
    pub blend_factor: f32,
}
```

## File Structure Implementation

### Recommended Directory Layout
```
assets/
├── characters/
│   ├── mixamo/
│   │   ├── basic_male/
│   │   │   ├── character.gltf
│   │   │   └── animations/
│   │   │       ├── idle.gltf
│   │   │       ├── walk.gltf
│   │   │       └── [other animations]
│   │   └── basic_female/
│   │       └── [similar structure]
│   ├── kaykit/
│   │   └── adventurers/
│   │       └── [kaykit assets]
│   └── character_manifest.ron  # Asset registry
├── textures/
│   └── characters/
│       ├── skin_variants/
│       └── clothing/
└── ATTRIBUTION.md  # License compliance
```

### Asset Registry Format
```ron
// assets/characters/character_manifest.ron
CharacterManifest(
    characters: {
        "basic_male": CharacterAsset(
            model_path: "characters/mixamo/basic_male/character.gltf",
            animations: {
                Idle: "characters/mixamo/basic_male/animations/idle.gltf",
                Walk: "characters/mixamo/basic_male/animations/walk.gltf",
                Run: "characters/mixamo/basic_male/animations/run.gltf",
            },
            source: "Mixamo",
            license: "Free Commercial Use",
        ),
    }
)
```

## Licensing Considerations

### Mixamo
- ✅ **Free commercial use**
- ✅ **No attribution required** (but recommended)
- ✅ **Modification allowed**
- ⚠️ **Cannot redistribute assets standalone** (only in compiled games)

### Kay Lousberg (KayKit)
- ✅ **Free commercial use** for free packs
- ✅ **Attribution recommended** but not required
- ✅ **Game integration allowed**
- ⚠️ **Check individual pack licenses**

### Compliance Strategy
1. Create `assets/ATTRIBUTION.md` with all asset sources
2. Include license files in version control
3. Document any modification made to original assets
4. Regular license review for new assets

## Next Steps

1. **Immediate**: Download 1-2 Mixamo characters with basic animations
2. **Week 1**: Implement basic character loading in amp_gameplay
3. **Week 2**: Add animation state machine and transitions
4. **Week 3**: Integrate with existing amp_gameplay systems
5. **Future**: Expand asset library and add character customization

## Technical Notes

- **Bevy 0.16.1 Compatibility**: Native glTF support via `bevy_gltf`
- **Animation System**: Use Bevy's built-in `AnimationPlayer` component
- **Performance**: Consider animation compression and LOD for large character counts
- **Fallback**: Ensure graceful degradation if assets fail to load
