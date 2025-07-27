# Mixamo FBX to glTF Conversion Guide

## Overview
Bevy 0.16.1 does not support FBX file format natively. To use your Mixamo characters, you need to convert them to glTF format (.glb files).

## Current Status
- ✅ Mixamo FBX files downloaded and organized in `assets/characters/mixamo/`
- ✅ Character system architecture implemented and working
- ❌ FBX to glTF conversion needed for actual character models
- ✅ Visual fallback system working (multi-part character with animations)

## Quick Conversion Options

### Option 1: Online Converters (Easiest)
1. **GitHub FBX2glTF Online**: Upload your FBX files to online conversion services
2. **Autodesk FBX Converter**: Free tool from Autodesk
3. **Asset conversion services**: Various online tools support FBX → glTF

### Option 2: Blender (Recommended)
1. **Install Blender** (free): https://www.blender.org/download/
2. **Import FBX**: File → Import → FBX (.fbx)
3. **Export glTF**: File → Export → glTF 2.0 (.glb)
4. **Settings**: 
   - Format: glTF Binary (.glb)
   - Include: Selected Objects (or All)
   - Animation: Include if present

### Option 3: Command Line Tools
```bash
# Using Blender command line (after installing Blender)
blender --background --python-expr "
import bpy
bpy.ops.import_scene.fbx(filepath='assets/characters/mixamo/models/Ch33_nonPBR.fbx')
bpy.ops.export_scene.gltf(filepath='assets/characters/mixamo/models/character.glb', export_format='GLB')
"

# Or using FBX2glTF (Facebook's converter)
# Download from: https://github.com/facebookincubator/FBX2glTF/releases
./FBX2glTF assets/characters/mixamo/models/Ch33_nonPBR.fbx
```

## Required Conversions
Convert these files in your `assets/characters/mixamo/` directory:

### Character Models
- `models/Ch33_nonPBR.fbx` → `models/character.glb`

### Animations  
- `animations/idle.fbx` → `animations/idle.glb`
- `animations/walking.fbx` → `animations/walking.glb`
- `animations/running.fbx` → `animations/running.glb`
- `animations/sprinting.fbx` → `animations/sprinting.glb`

## After Conversion

### 1. Update Asset Paths
Edit `assets/characters/mixamo_character.animset.ron` to use `.glb` files:
```ron
AnimationSet(
    animations: {
        Idle: AnimationClip(
            path: "characters/mixamo/animations/idle.glb#Animation0",  // Changed from .fbx
            speed: 1.0,
            loop_mode: true,
            blend_in_duration: 0.3,
            blend_out_duration: 0.3,
        ),
        // ... repeat for other animations
    },
    // ... rest of config
)
```

### 2. Update Character Loading
Re-enable the Mixamo scene loading in the demo and main game:
```rust
// Change this back to load the converted model
let path = "characters/mixamo/models/character.glb#Scene0";
```

### 3. Test the Integration
```bash
# Run the demo to test character loading
cargo run --example simple_visual_character_demo

# Run the main game
cargo run --bin gta_game
```

## Expected Behavior After Conversion
- ✅ Actual 3D Mixamo character model displays instead of capsule/visual parts
- ✅ Character animations work with the humanoid rig
- ✅ Smooth locomotion state transitions (Idle → Walk → Run → Sprint)
- ✅ Character responds to WASD movement and animation blending

## Troubleshooting

### Common Issues
1. **Scene not found**: Use `#Scene0` suffix for the main scene in the glTF file
2. **Animation not found**: Use `#Animation0` suffix for animations
3. **Model too large/small**: Adjust scale in the conversion settings
4. **Wrong orientation**: Check coordinate system settings during conversion

### Verification Commands
```bash
# Check if files exist
ls -la assets/characters/mixamo/models/
ls -la assets/characters/mixamo/animations/

# Test loading with Bevy
cargo run --example simple_visual_character_demo
```

## Next Steps After Conversion
1. **Test all locomotion states**: Walk, run, sprint transitions
2. **Adjust animation speeds**: Modify the `.animset.ron` file as needed
3. **Add more animations**: Jump, fall, landing, turning animations
4. **Integrate physics**: Ensure character controller works with the 3D model
5. **Camera adjustments**: Update camera positioning for the actual character size
