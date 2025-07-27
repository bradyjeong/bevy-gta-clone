//! Biome-based content generation for hierarchical world streaming
//!
//! This module implements the actual content generation logic using biome recipes
//! and integrates with the hierarchical world streaming system.

use super::*;
use crate::spawn_budget_policy::{EntityType, SpawnBudgetPolicy, SpawnData, SpawnPriority};
use bevy::prelude::*;
use rand::Rng;

/// Biome-aware content generator
#[derive(Component)]
pub struct BiomeContentGenerator {
    pub position: Vec3,
    pub biome_type: BiomeType,
    pub lod_level: amp_math::spatial::LODLevel,
    pub generation_seed: u64,
    pub generation_radius: f32,
}

impl BiomeContentGenerator {
    pub fn new(
        position: Vec3,
        biome_type: BiomeType,
        lod_level: amp_math::spatial::LODLevel,
        generation_seed: u64,
    ) -> Self {
        let generation_radius = match lod_level {
            amp_math::spatial::LODLevel::Macro => 5000.0,
            amp_math::spatial::LODLevel::Region => 1000.0,
            amp_math::spatial::LODLevel::Local => 200.0,
            amp_math::spatial::LODLevel::Detail => 50.0,
            amp_math::spatial::LODLevel::Micro => 12.5,
        };

        Self {
            position,
            biome_type,
            lod_level,
            generation_seed,
            generation_radius,
        }
    }
}

/// Generated content tracking component
#[derive(Component)]
pub struct GeneratedContent {
    pub biome_type: BiomeType,
    pub generation_seed: u64,
    pub entity_count: usize,
}

/// System for biome-based content generation
pub fn biome_content_generation_system(
    mut commands: Commands,
    generators: Query<(Entity, &BiomeContentGenerator)>,
    recipe_registry: Res<RecipeRegistry>,
    biome_registry: Res<BiomeRegistry>,
    mut spawn_budget: ResMut<SpawnBudgetPolicy>,
    time: Res<Time>,
) {
    const MAX_GENERATIONS_PER_FRAME: usize = 3;

    let mut processed = 0;
    for (entity, generator) in generators.iter() {
        if processed >= MAX_GENERATIONS_PER_FRAME {
            break;
        }

        // Get the generation recipe
        if let Some(recipe) = recipe_registry.get_recipe(generator.biome_type, generator.lod_level)
        {
            let entities =
                generate_biome_content(&mut commands, generator, recipe, &mut spawn_budget, &time);

            // Add tracking component
            commands.entity(entity).insert(GeneratedContent {
                biome_type: generator.biome_type,
                generation_seed: generator.generation_seed,
                entity_count: entities.len(),
            });

            // Remove the generator
            commands.entity(entity).despawn();
            processed += 1;
        }
    }
}

/// Generate content for a biome using the provided recipe
fn generate_biome_content(
    commands: &mut Commands,
    generator: &BiomeContentGenerator,
    recipe: &GenerationRecipe,
    spawn_budget: &mut ResMut<SpawnBudgetPolicy>,
    time: &Res<Time>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut rng = create_biome_rng(generator.position, generator.generation_seed);

    // Generate buildings
    entities.extend(generate_buildings(
        commands,
        generator,
        &recipe.building_rules,
        spawn_budget,
        time,
        &mut rng,
    ));

    // Generate vegetation
    entities.extend(generate_vegetation(
        commands,
        generator,
        &recipe.vegetation_rules,
        spawn_budget,
        time,
        &mut rng,
    ));

    // Generate vehicles (only at detail and micro levels)
    if matches!(
        generator.lod_level,
        amp_math::spatial::LODLevel::Detail | amp_math::spatial::LODLevel::Micro
    ) {
        entities.extend(generate_vehicles(
            commands,
            generator,
            &recipe.vehicle_rules,
            spawn_budget,
            time,
            &mut rng,
        ));
    }

    // Generate NPCs (only at micro level)
    if matches!(generator.lod_level, amp_math::spatial::LODLevel::Micro) {
        entities.extend(generate_npcs(
            commands,
            generator,
            &recipe.npc_rules,
            spawn_budget,
            time,
            &mut rng,
        ));
    }

    entities
}

/// Generate buildings using biome-specific rules
fn generate_buildings(
    commands: &mut Commands,
    generator: &BiomeContentGenerator,
    rules: &BuildingGenerationRules,
    spawn_budget: &mut ResMut<SpawnBudgetPolicy>,
    time: &Res<Time>,
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    if rules.density_multiplier <= 0.0 {
        return entities;
    }

    let num_buildings = (rules.max_buildings_per_chunk as f32 * rules.density_multiplier) as usize;
    let spawn_biome =
        enhanced_detect_biome_from_position(generator.position, &BiomeDetector::default());

    for _ in 0..num_buildings {
        // Check spawn budget
        if !spawn_budget.can_spawn(EntityType::Building, spawn_biome) {
            // Queue for later if budget exceeded
            let spawn_data = SpawnData::Building {
                position: generator.position,
                building_type: "biome_building".to_string(),
            };
            let _ = spawn_budget.request_spawn(
                EntityType::Building,
                spawn_biome,
                SpawnPriority::Low,
                spawn_data,
                time.elapsed_secs(),
            );
            continue;
        }

        // Select building type based on probabilities
        let building_type = select_weighted_item(&rules.building_types, |bt| bt.probability, rng);
        if let Some(building_type) = building_type {
            // Generate position within generation radius
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let distance = rng.gen::<f32>() * generator.generation_radius;
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let building_pos = generator.position + offset;

            // Vary building height
            let height_variance = 1.0 + (rng.gen::<f32>() - 0.5) * rules.height_variance;
            let building_height = building_type.typical_height * height_variance;

            let entity = commands
                .spawn((
                    Name::new(format!(
                        "BiomeBuilding_{}_{}_{}",
                        generator.biome_type as u8, generator.lod_level as u8, building_type.name
                    )),
                    Transform::from_translation(building_pos + Vec3::Y * building_height * 0.5),
                    GlobalTransform::default(),
                    Visibility::default(),
                    BiomeEntity {
                        biome_type: generator.biome_type,
                        generation_seed: generator.generation_seed,
                    },
                    crate::spawn_budget_integration::BuildingTag {
                        building_type: building_type.name.clone(),
                    },
                ))
                .id();

            entities.push(entity);
            spawn_budget.record_spawn(EntityType::Building);
        }
    }

    entities
}

/// Generate vegetation using biome-specific rules
fn generate_vegetation(
    commands: &mut Commands,
    generator: &BiomeContentGenerator,
    rules: &VegetationGenerationRules,
    spawn_budget: &mut ResMut<SpawnBudgetPolicy>,
    time: &Res<Time>,
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    if rules.density_multiplier <= 0.0 {
        return entities;
    }

    let base_count = (generator.generation_radius / 10.0) as usize;
    let num_vegetation = (base_count as f32 * rules.density_multiplier) as usize;
    let spawn_biome =
        enhanced_detect_biome_from_position(generator.position, &BiomeDetector::default());

    for _ in 0..num_vegetation {
        // Check spawn budget
        if !spawn_budget.can_spawn(EntityType::Tree, spawn_biome) {
            let spawn_data = SpawnData::Tree {
                position: generator.position,
                tree_type: "biome_vegetation".to_string(),
            };
            let _ = spawn_budget.request_spawn(
                EntityType::Tree,
                spawn_biome,
                SpawnPriority::Low,
                spawn_data,
                time.elapsed_secs(),
            );
            continue;
        }

        // Select vegetation type
        let vegetation_type =
            select_weighted_item(&rules.vegetation_types, |vt| vt.probability, rng);
        if let Some(vegetation_type) = vegetation_type {
            // Determine if this should be part of a cluster
            let is_cluster = rng.gen::<f32>() < rules.clustering_probability;
            let cluster_size = if is_cluster {
                rng.gen_range(2..=vegetation_type.clustering_size)
            } else {
                1
            };

            // Generate cluster center
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let distance = rng.gen::<f32>() * generator.generation_radius;
            let cluster_center =
                generator.position + Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            // Generate vegetation in cluster
            for _ in 0..cluster_size {
                if !spawn_budget.can_spawn(EntityType::Tree, spawn_biome) {
                    break;
                }

                let cluster_offset = if cluster_size > 1 {
                    let cluster_angle = rng.gen::<f32>() * std::f32::consts::TAU;
                    let cluster_dist = rng.gen::<f32>() * 5.0; // 5m cluster radius
                    Vec3::new(
                        cluster_angle.cos() * cluster_dist,
                        0.0,
                        cluster_angle.sin() * cluster_dist,
                    )
                } else {
                    Vec3::ZERO
                };

                let vegetation_pos = cluster_center + cluster_offset;
                let size =
                    rng.gen_range(vegetation_type.size_range.0..=vegetation_type.size_range.1);

                let entity = commands
                    .spawn((
                        Name::new(format!(
                            "BiomeVegetation_{}_{}_{}",
                            generator.biome_type as u8,
                            generator.lod_level as u8,
                            vegetation_type.name
                        )),
                        Transform::from_translation(vegetation_pos + Vec3::Y * size * 0.5),
                        GlobalTransform::default(),
                        Visibility::default(),
                        BiomeEntity {
                            biome_type: generator.biome_type,
                            generation_seed: generator.generation_seed,
                        },
                        crate::spawn_budget_integration::TreeTag {
                            tree_type: vegetation_type.name.clone(),
                        },
                    ))
                    .id();

                entities.push(entity);
                spawn_budget.record_spawn(EntityType::Tree);
            }
        }
    }

    entities
}

/// Generate vehicles using biome-specific rules  
fn generate_vehicles(
    commands: &mut Commands,
    generator: &BiomeContentGenerator,
    rules: &VehicleGenerationRules,
    spawn_budget: &mut ResMut<SpawnBudgetPolicy>,
    time: &Res<Time>,
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    if rules.density_multiplier <= 0.0 {
        return entities;
    }

    let base_count = (generator.generation_radius / 25.0) as usize; // Vehicles are less dense
    let num_vehicles = (base_count as f32 * rules.density_multiplier) as usize;
    let spawn_biome =
        enhanced_detect_biome_from_position(generator.position, &BiomeDetector::default());

    for _ in 0..num_vehicles {
        if !spawn_budget.can_spawn(EntityType::Vehicle, spawn_biome) {
            let spawn_data = SpawnData::Vehicle {
                position: generator.position,
                vehicle_type: "biome_vehicle".to_string(),
            };
            let _ = spawn_budget.request_spawn(
                EntityType::Vehicle,
                spawn_biome,
                SpawnPriority::Medium,
                spawn_data,
                time.elapsed_secs(),
            );
            continue;
        }

        let vehicle_type = select_weighted_item(&rules.vehicle_types, |vt| vt.probability, rng);
        if let Some(vehicle_type) = vehicle_type {
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let distance = rng.gen::<f32>() * generator.generation_radius;
            let vehicle_pos =
                generator.position + Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let entity = commands
                .spawn((
                    Name::new(format!(
                        "BiomeVehicle_{}_{}_{}",
                        generator.biome_type as u8, generator.lod_level as u8, vehicle_type.name
                    )),
                    Transform::from_translation(vehicle_pos),
                    GlobalTransform::default(),
                    Visibility::default(),
                    BiomeEntity {
                        biome_type: generator.biome_type,
                        generation_seed: generator.generation_seed,
                    },
                    crate::spawn_budget_integration::VehicleTag {
                        vehicle_type: vehicle_type.name.clone(),
                    },
                ))
                .id();

            entities.push(entity);
            spawn_budget.record_spawn(EntityType::Vehicle);
        }
    }

    entities
}

/// Generate NPCs using biome-specific rules
fn generate_npcs(
    commands: &mut Commands,
    generator: &BiomeContentGenerator,
    rules: &NpcGenerationRules,
    spawn_budget: &mut ResMut<SpawnBudgetPolicy>,
    time: &Res<Time>,
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    if rules.density_multiplier <= 0.0 {
        return entities;
    }

    let base_count = (generator.generation_radius / 50.0) as usize; // NPCs are sparse
    let num_npcs = (base_count as f32 * rules.density_multiplier) as usize;
    let spawn_biome =
        enhanced_detect_biome_from_position(generator.position, &BiomeDetector::default());

    for _ in 0..num_npcs {
        if !spawn_budget.can_spawn(EntityType::Npc, spawn_biome) {
            let spawn_data = SpawnData::Npc {
                position: generator.position,
                npc_type: "biome_npc".to_string(),
            };
            let _ = spawn_budget.request_spawn(
                EntityType::Npc,
                spawn_biome,
                SpawnPriority::Medium,
                spawn_data,
                time.elapsed_secs(),
            );
            continue;
        }

        let npc_type = select_weighted_item(&rules.npc_types, |nt| nt.probability, rng);
        if let Some(npc_type) = npc_type {
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let distance = rng.gen::<f32>() * generator.generation_radius;
            let npc_pos = generator.position
                + Vec3::new(
                    angle.cos() * distance,
                    1.8, // NPC height
                    angle.sin() * distance,
                );

            let entity = commands
                .spawn((
                    Name::new(format!(
                        "BiomeNPC_{}_{}_{}",
                        generator.biome_type as u8, generator.lod_level as u8, npc_type.name
                    )),
                    Transform::from_translation(npc_pos),
                    GlobalTransform::default(),
                    Visibility::default(),
                    BiomeEntity {
                        biome_type: generator.biome_type,
                        generation_seed: generator.generation_seed,
                    },
                    crate::spawn_budget_integration::NpcTag {
                        npc_type: npc_type.name.clone(),
                    },
                ))
                .id();

            entities.push(entity);
            spawn_budget.record_spawn(EntityType::Npc);
        }
    }

    entities
}

/// Helper function to select an item based on weighted probabilities
fn select_weighted_item<'a, T, F>(items: &'a [T], weight_fn: F, rng: &mut impl Rng) -> Option<&'a T>
where
    F: Fn(&T) -> f32,
{
    if items.is_empty() {
        return None;
    }

    let total_weight: f32 = items.iter().map(&weight_fn).sum();
    if total_weight <= 0.0 {
        return None;
    }

    let mut random_weight = rng.gen::<f32>() * total_weight;

    for item in items {
        random_weight -= weight_fn(item);
        if random_weight <= 0.0 {
            return Some(item);
        }
    }

    // Fallback to last item if floating point precision issues
    items.last()
}

/// System to update biome detection based on position changes
pub fn update_biome_detection_system(
    mut query: Query<(&Transform, &mut BiomeEntity), Changed<Transform>>,
    detector: Res<BiomeDetector>,
) {
    for (transform, mut biome_entity) in query.iter_mut() {
        let new_biome = detector.detect_biome(transform.translation);
        if new_biome != biome_entity.biome_type {
            biome_entity.biome_type = new_biome;
        }
    }
}

/// System to create biome content generators for world chunks
/// Note: This should be integrated with amp_engine's world streaming system
#[cfg(feature = "unstable_hierarchical_world")]
pub fn create_biome_generators_for_position(
    mut commands: Commands,
    position: Vec3,
    lod_level: amp_math::spatial::LODLevel,
    generation_seed: u64,
    detector: &BiomeDetector,
) {
    let biome_type = detector.detect_biome(position);

    let generator = BiomeContentGenerator::new(position, biome_type, lod_level, generation_seed);

    commands.spawn(generator);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_content_generator_creation() {
        let generator = BiomeContentGenerator::new(
            Vec3::new(100.0, 0.0, 200.0),
            BiomeType::Urban,
            amp_math::spatial::LODLevel::Local,
            12345,
        );

        assert_eq!(generator.position, Vec3::new(100.0, 0.0, 200.0));
        assert_eq!(generator.biome_type, BiomeType::Urban);
        assert_eq!(generator.lod_level, amp_math::spatial::LODLevel::Local);
        assert_eq!(generator.generation_seed, 12345);
    }

    #[test]
    fn test_weighted_item_selection() {
        struct TestItem {
            name: String,
            weight: f32,
        }

        let items = vec![
            TestItem {
                name: "A".to_string(),
                weight: 0.5,
            },
            TestItem {
                name: "B".to_string(),
                weight: 0.3,
            },
            TestItem {
                name: "C".to_string(),
                weight: 0.2,
            },
        ];

        let mut rng = create_biome_rng(Vec3::ZERO, 0);
        let selected = select_weighted_item(&items, |item| item.weight, &mut rng);

        assert!(selected.is_some());
        assert!(["A", "B", "C"].contains(&selected.unwrap().name.as_str()));
    }

    #[test]
    fn test_generation_radius_scaling() {
        let micro_gen = BiomeContentGenerator::new(
            Vec3::ZERO,
            BiomeType::Urban,
            amp_math::spatial::LODLevel::Micro,
            0,
        );

        let macro_gen = BiomeContentGenerator::new(
            Vec3::ZERO,
            BiomeType::Urban,
            amp_math::spatial::LODLevel::Macro,
            0,
        );

        // Macro should have larger generation radius
        assert!(macro_gen.generation_radius > micro_gen.generation_radius);
    }
}
