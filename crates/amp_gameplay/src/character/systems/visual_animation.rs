//! Visual character animation systems
//!
//! Systems for animating the multi-part visual character based on locomotion state.

use crate::character::components::{Locomotion, LocomotionState, Speed, Velocity};
use crate::character::visual::*;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

/// System to update body part animations based on character locomotion state
pub fn update_body_part_animations(
    time: Res<Time>,
    mut body_part_query: Query<
        (&mut BodyPartAnimation, &mut Transform),
        (
            Or<(
                With<CharacterHead>,
                With<CharacterTorso>,
                With<CharacterLeftArm>,
                With<CharacterRightArm>,
                With<CharacterLeftLeg>,
                With<CharacterRightLeg>,
            )>,
            Without<VisualCharacter>,
        ),
    >,
    character_query: Query<
        (&LocomotionState, &Velocity, &Speed, &BodyParts),
        With<VisualCharacter>,
    >,
) {
    for (locomotion, velocity, speed, body_parts) in character_query.iter() {
        let speed_factor = match locomotion.current {
            Locomotion::Idle => 0.0,
            Locomotion::Walk => 1.0,
            Locomotion::Run => 1.5,
            Locomotion::Sprint => 2.0,
            _ => 1.0,
        };

        let movement_speed = velocity.linear.length();
        let normalized_speed = (movement_speed / speed.walk).clamp(0.0, 3.0);

        // Update each body part based on its type
        let body_part_entities = [
            body_parts.head,
            body_parts.torso,
            body_parts.left_arm,
            body_parts.right_arm,
            body_parts.left_leg,
            body_parts.right_leg,
        ];

        for entity in body_part_entities {
            if let Ok((mut animation, mut transform)) = body_part_query.get_mut(entity) {
                // Update animation phase based on movement speed
                animation.phase += time.delta_secs() * 2.0 * speed_factor * normalized_speed;

                // Apply different animations based on body part type
                // We check which entity this is by comparing with the BodyParts
                if entity == body_parts.head {
                    animate_head(
                        &mut animation,
                        &mut transform,
                        locomotion.current,
                        normalized_speed,
                    );
                } else if entity == body_parts.torso {
                    animate_torso(
                        &mut animation,
                        &mut transform,
                        locomotion.current,
                        normalized_speed,
                    );
                } else if entity == body_parts.left_arm || entity == body_parts.right_arm {
                    animate_arm(
                        &mut animation,
                        &mut transform,
                        locomotion.current,
                        normalized_speed,
                    );
                } else if entity == body_parts.left_leg || entity == body_parts.right_leg {
                    animate_leg(
                        &mut animation,
                        &mut transform,
                        locomotion.current,
                        normalized_speed,
                    );
                }
            }
        }
    }
}

fn animate_head(
    animation: &mut BodyPartAnimation,
    transform: &mut Transform,
    locomotion: Locomotion,
    speed: f32,
) {
    let base_y = 1.6; // Default head height

    match locomotion {
        Locomotion::Idle => {
            // Subtle idle bobbing
            let bob = (animation.phase * 0.5).sin() * 0.01;
            transform.translation.y = base_y + bob;
        }
        Locomotion::Walk | Locomotion::Run | Locomotion::Sprint => {
            // Head bobbing while moving
            let bob_intensity = 0.03 * speed;
            let bob = (animation.phase * 2.0).sin() * bob_intensity;
            transform.translation.y = base_y + bob;

            // Slight forward lean when sprinting
            if locomotion == Locomotion::Sprint {
                let lean_angle = 5.0_f32.to_radians() * speed;
                transform.rotation = Quat::from_rotation_x(lean_angle);
            } else {
                transform.rotation = Quat::IDENTITY;
            }
        }
        _ => {
            transform.translation.y = base_y;
            transform.rotation = Quat::IDENTITY;
        }
    }
}

fn animate_torso(
    animation: &mut BodyPartAnimation,
    transform: &mut Transform,
    locomotion: Locomotion,
    speed: f32,
) {
    match locomotion {
        Locomotion::Idle => {
            // Subtle idle breathing
            let breath = (animation.phase * 0.3).sin() * 0.005;
            let scale = 1.0 + breath;
            transform.scale = Vec3::new(1.0, scale, 1.0 + breath * 0.5);
            transform.rotation = Quat::IDENTITY;
        }
        Locomotion::Walk | Locomotion::Run => {
            // Slight rotation for natural movement
            let sway_angle = (animation.phase).sin() * 2.0_f32.to_radians() * speed * 0.5;
            transform.rotation = Quat::from_rotation_y(sway_angle);
            transform.scale = Vec3::ONE;
        }
        Locomotion::Sprint => {
            // Forward lean and more sway when sprinting
            let lean_angle = 8.0_f32.to_radians() * speed;
            let sway_angle = (animation.phase).sin() * 3.0_f32.to_radians() * speed;
            transform.rotation =
                Quat::from_rotation_x(lean_angle) * Quat::from_rotation_y(sway_angle);
            transform.scale = Vec3::ONE;
        }
        _ => {
            transform.rotation = Quat::IDENTITY;
            transform.scale = Vec3::ONE;
        }
    }
}

fn animate_arm(
    animation: &mut BodyPartAnimation,
    transform: &mut Transform,
    locomotion: Locomotion,
    speed: f32,
) {
    let is_left_arm = transform.translation.x < 0.0;

    match locomotion {
        Locomotion::Idle => {
            // Arms hang naturally
            transform.rotation = Quat::IDENTITY;
        }
        Locomotion::Walk | Locomotion::Run | Locomotion::Sprint => {
            // Arm swinging - opposite to legs
            let phase_offset = if is_left_arm {
                0.0
            } else {
                std::f32::consts::PI
            };
            let swing_phase = animation.phase + phase_offset;

            let swing_intensity = match locomotion {
                Locomotion::Walk => 0.3,
                Locomotion::Run => 0.5,
                Locomotion::Sprint => 0.7,
                _ => 0.0,
            };

            let swing_angle =
                swing_phase.sin() * animation.max_swing_angle * swing_intensity * speed;

            // Swing arms forward and back (rotation around X axis)
            transform.rotation = Quat::from_rotation_x(swing_angle);
        }
        Locomotion::Jump => {
            // Arms up when jumping
            let raise_angle = -45.0_f32.to_radians();
            transform.rotation = Quat::from_rotation_x(raise_angle);
        }
        _ => {
            transform.rotation = Quat::IDENTITY;
        }
    }
}

fn animate_leg(
    animation: &mut BodyPartAnimation,
    transform: &mut Transform,
    locomotion: Locomotion,
    speed: f32,
) {
    let is_left_leg = transform.translation.x < 0.0;

    match locomotion {
        Locomotion::Idle => {
            // Legs in neutral position
            transform.rotation = Quat::IDENTITY;
        }
        Locomotion::Walk | Locomotion::Run | Locomotion::Sprint => {
            // Leg movement for walking/running
            let phase_offset = if is_left_leg {
                0.0
            } else {
                std::f32::consts::PI
            };
            let walk_phase = animation.phase + phase_offset;

            let step_intensity = match locomotion {
                Locomotion::Walk => 0.4,
                Locomotion::Run => 0.6,
                Locomotion::Sprint => 0.8,
                _ => 0.0,
            };

            let step_angle = walk_phase.sin() * animation.max_swing_angle * step_intensity * speed;

            // Legs swing forward and back (rotation around X axis)
            transform.rotation = Quat::from_rotation_x(step_angle);
        }
        Locomotion::Jump => {
            // Legs bent when jumping
            let bend_angle = 30.0_f32.to_radians();
            transform.rotation = Quat::from_rotation_x(-bend_angle);
        }
        _ => {
            transform.rotation = Quat::IDENTITY;
        }
    }
}

/// System to update visual character state indicators based on locomotion
pub fn update_character_visual_state(
    mut material_query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    character_query: Query<(&LocomotionState, &BodyParts), With<VisualCharacter>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (locomotion, body_parts) in character_query.iter() {
        // Change torso color based on current state for visual feedback
        if let Ok(material_component) = material_query.get_mut(body_parts.torso) {
            if let Some(material) = materials.get_mut(&material_component.0) {
                material.base_color = match locomotion.current {
                    Locomotion::Idle => Color::srgb(0.2, 0.4, 0.8), // Blue
                    Locomotion::Walk => Color::srgb(0.2, 0.8, 0.4), // Green
                    Locomotion::Run => Color::srgb(0.8, 0.8, 0.2),  // Yellow
                    Locomotion::Sprint => Color::srgb(0.8, 0.2, 0.2), // Red
                    Locomotion::Jump => Color::srgb(0.8, 0.4, 0.8), // Purple
                    _ => Color::srgb(0.5, 0.5, 0.5),                // Gray
                };
            }
        }
    }
}

/// System to make body parts follow their parent character
pub fn update_body_part_transforms(
    character_query: Query<(&Transform, &BodyParts), (With<VisualCharacter>, Changed<Transform>)>,
    mut body_part_query: Query<&mut Transform, (Without<VisualCharacter>, Without<BodyParts>)>,
) {
    for (character_transform, body_parts) in character_query.iter() {
        let body_part_entities = [
            body_parts.head,
            body_parts.torso,
            body_parts.left_arm,
            body_parts.right_arm,
            body_parts.left_leg,
            body_parts.right_leg,
        ];

        // Update each body part's global position based on character transform
        for entity in body_part_entities {
            if let Ok(mut body_part_transform) = body_part_query.get_mut(entity) {
                // Apply character's rotation to the body part's base position
                let base_translation = body_part_transform.translation;
                let rotated_translation = character_transform.rotation * base_translation;

                // Update position relative to character
                body_part_transform.translation =
                    character_transform.translation + rotated_translation;
            }
        }
    }
}
