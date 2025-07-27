# Oracle Asset Integration System Fixes

This document details the fixes implemented for the Oracle-identified issues in the asset integration system.

## Issues Fixed

### 1. Scene Spawning Race Condition
**Problem**: Asset loading system was trying to process skeleton immediately after scene spawning without waiting for scene instantiation.

**Solution**: 
- Added `WaitingForScene` loading stage between `LoadingGltf` and `ProcessingSkeleton`
- Added scene instance tracking in `CharacterAssetLoading` component
- Implemented proper scene instantiation checking using `SceneInstance` and `SceneSpawner.instance_is_ready()`
- Only proceed to skeleton processing after scene is fully instantiated

**Files Modified**: `crates/amp_gameplay/src/character/systems/asset_loading.rs`

### 2. Bone Mapping Performance Optimization  
**Problem**: O(n²) bone scanning due to nested loops over bone names and bone variants.

**Solution**:
- Added `HumanoidRig::from_skeleton_optimized()` method using O(n) HashMap lookups
- Pre-build HashMap with stripped bone name variants for flexible matching
- Single pass through bone variants with O(1) HashMap lookups
- Performance improvement from O(n²) to O(n) complexity

**Files Modified**: `crates/amp_gameplay/src/character/components.rs`

### 3. AnimationPlayer Placement Fix
**Problem**: AnimationPlayer was being added to character root entity instead of skeleton entity.

**Solution**:
- Store skeleton entity reference in `CharacterAssetLoading` component
- Place AnimationPlayer on skeleton entity during `SettingUpAnimation` stage
- Ensures animations target the correct entity hierarchy

**Files Modified**: `crates/amp_gameplay/src/character/systems/asset_loading.rs`

### 4. AnimationSet Performance Improvements
**Problem**: AnimationSet used HashMap for locomotion state lookups, causing heap allocations.

**Solution**:
- Replaced `HashMap<Locomotion, Handle<AnimationClip>>` with fixed-size arrays
- Added explicit enum indices to `Locomotion` enum (0-7)
- Array-based lookups: `clips[state as usize]` for O(1) performance
- Added transition speeds table and validation functionality
- Used `const { None }` blocks for array initialization of non-Copy types

**Files Modified**: `crates/amp_gameplay/src/character/components.rs`

### 5. Scaling System Improvements
**Problem**: Scale was applied to root entity, affecting both visual and skeleton hierarchies.

**Solution**:
- Target only visual model entities, skip skeleton/armature entities
- Recursive scaling application to complex model hierarchies
- Name-based filtering to exclude `Armature`, `Skeleton`, and `mixamorig` entities
- Proper hierarchy traversal with visual mesh identification

**Files Modified**: `crates/amp_gameplay/src/character/systems/asset_loading.rs`

## Technical Details

### New Loading Flow
```
LoadingGltf → WaitingForScene → ProcessingSkeleton → SettingUpAnimation → Complete
```

### Performance Improvements
- **Bone Mapping**: O(n²) → O(n) using HashMap pre-processing
- **Animation Lookups**: HashMap → Array indexing (cache-friendly)
- **Scene Loading**: Proper async handling prevents race conditions

### Entity Hierarchy
```
Character Entity
├── SceneRoot (visual model)
│   ├── Mesh entities (scaled)
│   └── Armature/Skeleton (unscaled)
│       └── Bone hierarchy (AnimationPlayer here)
```

## Validation

All fixes maintain backward compatibility and follow Bevy 0.16.1 best practices. The system now properly handles:

1. ✅ Asynchronous scene instantiation
2. ✅ Optimized bone mapping performance  
3. ✅ Correct AnimationPlayer placement
4. ✅ Array-based animation lookups
5. ✅ Proper visual model scaling

## Testing

Run `cargo check --workspace` to verify compilation. The fixes address all Oracle-identified performance and correctness issues while maintaining the existing API surface.
