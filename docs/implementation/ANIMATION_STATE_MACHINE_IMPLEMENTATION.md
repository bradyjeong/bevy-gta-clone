# Animation State Machine Implementation

## Overview

Created a comprehensive animation state machine system for humanoid characters in the `amp_gameplay` crate that manages locomotion state transitions and animation blending based on character velocity.

## Implementation Details

### 1. Animation Systems Module Structure
- **Location**: `crates/amp_gameplay/src/character/systems/animation.rs`
- **Integration**: Added to `CharacterPlugin` system set with proper ordering
- **Dependencies**: Uses existing `LocomotionState`, `Velocity`, and `AnimationSet` components

### 2. Core Systems

#### `update_locomotion_state` System
- **Purpose**: Updates character locomotion state based on velocity magnitude
- **Logic**: Maps horizontal velocity (ignoring Y-axis) to appropriate locomotion states
- **Features**:
  - Velocity thresholds: Idle (0.1), Walk (2.0), Run (5.0), Sprint (8.0)
  - **Hysteresis**: 20% buffer to prevent rapid state oscillation
  - Dynamic transition durations based on state change magnitude
  - Proper handling of special states (Jump/Fall/Land/Turn)

#### `apply_animation_transitions` System  
- **Purpose**: Manages animation blending and transition progress
- **Features**:
  - Calculates blend weights for smooth cross-fades
  - Updates transition timers using frame delta time
  - Prepares foundation for actual animation player integration
  - Debug logging for state transitions

### 3. State Machine Logic

#### Velocity Mapping
```rust
// Velocity thresholds with hysteresis
Idle:   0.0 - 0.1 m/s
Walk:   0.1 - 2.0 m/s  
Run:    2.0 - 5.0 m/s
Sprint: 5.0+ m/s
```

#### Hysteresis Implementation
- **Purpose**: Prevents rapid state changes near threshold boundaries
- **Mechanism**: Uses 80% of threshold when transitioning back to lower states
- **Example**: Walk→Idle requires speed ≤ 0.08 (0.1 * 0.8) instead of 0.1

#### Transition Durations
- **Adjacent states**: 0.1-0.2s (e.g., Walk↔Run)
- **Larger jumps**: 0.3-0.4s (e.g., Idle↔Sprint)
- **Special states**: 0.05-0.2s (Jump/Fall/Land)

### 4. Component Integration

#### Existing Components Used
- **`LocomotionState`**: Enhanced with transition timer and progress tracking
- **`Velocity`**: Reads linear velocity for state determination
- **`CharacterAnimations`**: Links to animation assets
- **`AnimationSet`**: Asset containing animation clips per state

#### Component Methods
```rust
// LocomotionState methods
state.transition_to(new_state, duration);
state.transition_progress(); // 0.0-1.0
state.is_transitioning();
state.update(delta_time);
```

### 5. System Integration

#### Plugin Integration
```rust
// Added to CharacterPlugin in Update schedule
.add_systems(
    Update,
    (
        animation::update_locomotion_state,
        animation::apply_animation_transitions,
    )
        .chain()
        .after(movement::update_velocity_steering),
)
```

#### System Ordering
1. **Movement systems** (FixedUpdate): Handle physics and velocity
2. **Velocity steering** (Update): Smooth velocity interpolation  
3. **Animation state** (Update): Update locomotion based on velocity
4. **Animation transitions** (Update): Apply blending and transitions
5. **Camera systems** (Update): Visual updates

### 6. Testing & Validation

#### Unit Tests Included
- **`test_locomotion_state_transitions`**: Validates velocity→state mapping
- **`test_transition_durations`**: Verifies transition timing logic
- **`test_locomotion_state_component`**: Tests component state management

#### Test Coverage
- State transition logic with hysteresis
- Transition duration calculations
- Component timer management
- Progress tracking accuracy

## Architecture Design

### Foundation for Extension
The implementation provides a solid foundation that can be extended with:

1. **Animation Player Integration**: Ready for Bevy's animation system
2. **Blend Trees**: Infrastructure for complex animation blending
3. **Special States**: Easy addition of Jump/Fall/Land logic
4. **Directional Movement**: Support for strafing and turning animations
5. **Layer System**: Multiple animation layers (upper/lower body)

### Performance Considerations
- **Efficient Queries**: Systems use targeted queries for relevant entities
- **Minimal Allocations**: State calculations use stack-based operations
- **Delta Time Aware**: Proper frame-rate independent timing
- **Debug Mode**: Logging can be disabled in release builds

### Bevy 0.16.1 Compatibility
- Uses `time.delta_secs()` (correct Bevy 0.16.1 API)
- Follows Bevy's ECS patterns and system ordering
- Compatible with existing character movement systems
- Ready for integration with Bevy's animation framework

## Future Extensions

### Phase 1: Animation Player Integration
- Connect to Bevy's `AnimationPlayer` component
- Implement actual animation clip playback
- Add cross-fade blending between states

### Phase 2: Advanced States
- Implement Jump/Fall/Land state logic with ground detection
- Add directional movement states (strafe, turn)
- Support for crouching and other poses

### Phase 3: Complex Blending
- Layer-based animation system (upper/lower body)
- Additive animations for emotions and reactions
- IK integration for foot placement and look-at

## Integration Status

✅ **Core State Machine**: Complete and tested  
✅ **Velocity-Based Transitions**: Working with hysteresis  
✅ **Component Integration**: Integrated with existing character systems  
✅ **Plugin Registration**: Added to CharacterPlugin  
✅ **System Ordering**: Proper execution order established  
⏳ **Animation Player**: Ready for implementation  
⏳ **Special States**: Framework ready for Jump/Fall/Land  

The animation state machine provides a professional foundation for character animation that follows AAA game development practices with smooth transitions, hysteresis-based stability, and extensible architecture.
