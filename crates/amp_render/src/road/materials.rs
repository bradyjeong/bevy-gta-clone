/// Road material creation and management for different surface types
use bevy::prelude::*;

/// Road surface types with their material properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoadSurfaceType {
    /// New asphalt surface
    Asphalt,
    /// Older concrete surface
    Concrete,
    /// Dirt or gravel road
    Dirt,
    /// Cobblestone (historical areas)
    Cobblestone,
    /// Wet asphalt with reflective properties
    WetAsphalt,
}

impl Default for RoadSurfaceType {
    fn default() -> Self {
        Self::Asphalt
    }
}

impl RoadSurfaceType {
    /// Get the base color for this surface type
    pub fn base_color(&self) -> Color {
        match self {
            RoadSurfaceType::Asphalt => Color::srgb(0.35, 0.35, 0.4),
            RoadSurfaceType::Concrete => Color::srgb(0.5, 0.5, 0.55),
            RoadSurfaceType::Dirt => Color::srgb(0.6, 0.4, 0.3),
            RoadSurfaceType::Cobblestone => Color::srgb(0.4, 0.4, 0.45),
            RoadSurfaceType::WetAsphalt => Color::srgb(0.2, 0.2, 0.25),
        }
    }

    /// Get the roughness value for PBR materials
    pub fn roughness(&self) -> f32 {
        match self {
            RoadSurfaceType::Asphalt => 0.8,
            RoadSurfaceType::Concrete => 0.7,
            RoadSurfaceType::Dirt => 0.9,
            RoadSurfaceType::Cobblestone => 0.6,
            RoadSurfaceType::WetAsphalt => 0.2, // Much smoother when wet
        }
    }

    /// Get the metallic value for PBR materials
    pub fn metallic(&self) -> f32 {
        match self {
            RoadSurfaceType::Asphalt => 0.0,
            RoadSurfaceType::Concrete => 0.0,
            RoadSurfaceType::Dirt => 0.0,
            RoadSurfaceType::Cobblestone => 0.0,
            RoadSurfaceType::WetAsphalt => 0.1, // Slight metallic for wet reflection
        }
    }

    /// Get the reflectance value for PBR materials
    pub fn reflectance(&self) -> f32 {
        match self {
            RoadSurfaceType::Asphalt => 0.2,
            RoadSurfaceType::Concrete => 0.25,
            RoadSurfaceType::Dirt => 0.1,
            RoadSurfaceType::Cobblestone => 0.3,
            RoadSurfaceType::WetAsphalt => 0.8, // High reflectance when wet
        }
    }

    /// Get emissive color for special effects
    pub fn emissive(&self) -> LinearRgba {
        match self {
            RoadSurfaceType::WetAsphalt => LinearRgba::new(0.05, 0.05, 0.1, 1.0), // Slight blue tint
            _ => LinearRgba::BLACK,
        }
    }

    /// Get the friction coefficient for physics integration
    pub fn friction_coefficient(&self) -> f32 {
        match self {
            RoadSurfaceType::Asphalt => 0.9,
            RoadSurfaceType::Concrete => 0.8,
            RoadSurfaceType::Dirt => 0.6,
            RoadSurfaceType::Cobblestone => 0.7,
            RoadSurfaceType::WetAsphalt => 0.4, // Much lower friction when wet
        }
    }
}

/// Road marking types with their visual properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkingType {
    /// Standard white lane markings
    WhiteLine,
    /// Yellow center line (no passing zones)
    YellowLine,
    /// Blue handicap markings
    BlueMarking,
    /// Red stop line markings
    RedStopLine,
    /// Reflective lane markings for night visibility
    ReflectiveLine,
}

impl Default for MarkingType {
    fn default() -> Self {
        Self::WhiteLine
    }
}

impl MarkingType {
    /// Get the base color for this marking type
    pub fn base_color(&self) -> Color {
        match self {
            MarkingType::WhiteLine => Color::srgb(0.95, 0.95, 0.95),
            MarkingType::YellowLine => Color::srgb(1.0, 0.9, 0.0),
            MarkingType::BlueMarking => Color::srgb(0.0, 0.4, 0.8),
            MarkingType::RedStopLine => Color::srgb(0.9, 0.1, 0.1),
            MarkingType::ReflectiveLine => Color::srgb(1.0, 1.0, 1.0),
        }
    }

    /// Get emissive properties for visibility
    pub fn emissive(&self) -> LinearRgba {
        match self {
            MarkingType::WhiteLine => LinearRgba::new(0.2, 0.2, 0.2, 1.0),
            MarkingType::YellowLine => LinearRgba::new(0.3, 0.2, 0.0, 1.0),
            MarkingType::BlueMarking => LinearRgba::new(0.0, 0.1, 0.2, 1.0),
            MarkingType::RedStopLine => LinearRgba::new(0.2, 0.0, 0.0, 1.0),
            MarkingType::ReflectiveLine => LinearRgba::new(0.4, 0.4, 0.4, 1.0), // High emissive for reflective
        }
    }

    /// Get roughness for marking materials
    pub fn roughness(&self) -> f32 {
        match self {
            MarkingType::ReflectiveLine => 0.3, // Smoother for better reflection
            _ => 0.6,
        }
    }

    /// Get reflectance for marking materials
    pub fn reflectance(&self) -> f32 {
        match self {
            MarkingType::ReflectiveLine => 0.9, // High reflectance
            _ => 0.5,
        }
    }
}

/// Factory functions for creating road materials
pub struct RoadMaterialFactory;

impl RoadMaterialFactory {
    /// Create a standard road surface material
    pub fn create_road_material(
        surface_type: RoadSurfaceType,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: surface_type.base_color(),
            perceptual_roughness: surface_type.roughness(),
            metallic: surface_type.metallic(),
            reflectance: surface_type.reflectance(),
            emissive: surface_type.emissive(),
            ..default()
        })
    }

    /// Create a road surface material with custom texture
    pub fn create_textured_road_material(
        surface_type: RoadSurfaceType,
        texture: Handle<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            base_color: surface_type.base_color(),
            perceptual_roughness: surface_type.roughness(),
            metallic: surface_type.metallic(),
            reflectance: surface_type.reflectance(),
            emissive: surface_type.emissive(),
            ..default()
        })
    }

    /// Create a lane marking material
    pub fn create_marking_material(
        marking_type: MarkingType,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: marking_type.base_color(),
            emissive: marking_type.emissive(),
            perceptual_roughness: marking_type.roughness(),
            metallic: 0.0,
            reflectance: marking_type.reflectance(),
            ..default()
        })
    }

    /// Create a road material optimized for different weather conditions
    pub fn create_weather_adjusted_material(
        surface_type: RoadSurfaceType,
        weather_factor: f32, // 0.0 = dry, 1.0 = very wet
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let base_roughness = surface_type.roughness();
        let wet_roughness = base_roughness * (1.0 - weather_factor * 0.7); // Reduce roughness when wet

        let base_reflectance = surface_type.reflectance();
        let wet_reflectance = base_reflectance + weather_factor * 0.5; // Increase reflectance when wet

        let base_color = surface_type.base_color();
        let wet_color = Color::srgb(
            base_color.to_srgba().red * (1.0 - weather_factor * 0.3),
            base_color.to_srgba().green * (1.0 - weather_factor * 0.3),
            base_color.to_srgba().blue * (1.0 - weather_factor * 0.2), // Less blue reduction for wet look
        );

        materials.add(StandardMaterial {
            base_color: wet_color,
            perceptual_roughness: wet_roughness,
            metallic: surface_type.metallic() + weather_factor * 0.1,
            reflectance: wet_reflectance.min(1.0),
            emissive: surface_type.emissive(),
            ..default()
        })
    }

    /// Create intersection material with special properties
    pub fn create_intersection_material(
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.45), // Slightly different from regular asphalt
            perceptual_roughness: 0.75,
            metallic: 0.0,
            reflectance: 0.25,
            emissive: LinearRgba::BLACK,
            ..default()
        })
    }
}

/// Resource containing pre-created road materials for performance
#[derive(Resource)]
pub struct RoadMaterialLibrary {
    /// Standard surface materials
    pub surfaces: std::collections::HashMap<RoadSurfaceType, Handle<StandardMaterial>>,
    /// Standard marking materials
    pub markings: std::collections::HashMap<MarkingType, Handle<StandardMaterial>>,
    /// Weather-adjusted materials cache
    pub weather_materials:
        std::collections::HashMap<(RoadSurfaceType, u8), Handle<StandardMaterial>>,
    /// Intersection material
    pub intersection: Handle<StandardMaterial>,
}

impl RoadMaterialLibrary {
    /// Create a new material library with all standard materials
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut surfaces = std::collections::HashMap::new();
        let mut markings = std::collections::HashMap::new();

        // Create all surface materials
        surfaces.insert(
            RoadSurfaceType::Asphalt,
            RoadMaterialFactory::create_road_material(RoadSurfaceType::Asphalt, materials),
        );
        surfaces.insert(
            RoadSurfaceType::Concrete,
            RoadMaterialFactory::create_road_material(RoadSurfaceType::Concrete, materials),
        );
        surfaces.insert(
            RoadSurfaceType::Dirt,
            RoadMaterialFactory::create_road_material(RoadSurfaceType::Dirt, materials),
        );
        surfaces.insert(
            RoadSurfaceType::Cobblestone,
            RoadMaterialFactory::create_road_material(RoadSurfaceType::Cobblestone, materials),
        );
        surfaces.insert(
            RoadSurfaceType::WetAsphalt,
            RoadMaterialFactory::create_road_material(RoadSurfaceType::WetAsphalt, materials),
        );

        // Create all marking materials
        markings.insert(
            MarkingType::WhiteLine,
            RoadMaterialFactory::create_marking_material(MarkingType::WhiteLine, materials),
        );
        markings.insert(
            MarkingType::YellowLine,
            RoadMaterialFactory::create_marking_material(MarkingType::YellowLine, materials),
        );
        markings.insert(
            MarkingType::BlueMarking,
            RoadMaterialFactory::create_marking_material(MarkingType::BlueMarking, materials),
        );
        markings.insert(
            MarkingType::RedStopLine,
            RoadMaterialFactory::create_marking_material(MarkingType::RedStopLine, materials),
        );
        markings.insert(
            MarkingType::ReflectiveLine,
            RoadMaterialFactory::create_marking_material(MarkingType::ReflectiveLine, materials),
        );

        let intersection = RoadMaterialFactory::create_intersection_material(materials);

        Self {
            surfaces,
            markings,
            weather_materials: std::collections::HashMap::new(),
            intersection,
        }
    }

    /// Get a surface material by type
    pub fn get_surface_material(&self, surface_type: RoadSurfaceType) -> Handle<StandardMaterial> {
        self.surfaces
            .get(&surface_type)
            .unwrap_or_else(|| self.surfaces.get(&RoadSurfaceType::Asphalt).unwrap())
            .clone()
    }

    /// Get a marking material by type
    pub fn get_marking_material(&self, marking_type: MarkingType) -> Handle<StandardMaterial> {
        self.markings
            .get(&marking_type)
            .unwrap_or_else(|| self.markings.get(&MarkingType::WhiteLine).unwrap())
            .clone()
    }

    /// Get intersection material
    pub fn get_intersection_material(&self) -> Handle<StandardMaterial> {
        self.intersection.clone()
    }

    /// Get or create weather-adjusted material
    pub fn get_weather_material(
        &mut self,
        surface_type: RoadSurfaceType,
        weather_factor: f32,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let weather_key = (surface_type, (weather_factor * 100.0) as u8);

        if let Some(material) = self.weather_materials.get(&weather_key) {
            material.clone()
        } else {
            let material = RoadMaterialFactory::create_weather_adjusted_material(
                surface_type,
                weather_factor,
                materials,
            );
            self.weather_materials.insert(weather_key, material.clone());
            material
        }
    }
}

/// Helper trait for road material operations
pub trait RoadMaterialExt {
    /// Apply wear effect to a road material
    fn apply_wear(&mut self, wear_factor: f32);

    /// Apply weather effects to a road material
    fn apply_weather(&mut self, wetness: f32);
}

impl RoadMaterialExt for StandardMaterial {
    fn apply_wear(&mut self, wear_factor: f32) {
        // Increase roughness and reduce reflectance based on wear
        self.perceptual_roughness = (self.perceptual_roughness + wear_factor * 0.3).min(1.0);
        self.reflectance = (self.reflectance - wear_factor * 0.2).max(0.0);

        // Darken the surface slightly
        let current_color = self.base_color.to_srgba();
        self.base_color = Color::srgb(
            current_color.red * (1.0 - wear_factor * 0.1),
            current_color.green * (1.0 - wear_factor * 0.1),
            current_color.blue * (1.0 - wear_factor * 0.1),
        );
    }

    fn apply_weather(&mut self, wetness: f32) {
        // Reduce roughness and increase reflectance when wet
        let base_roughness = self.perceptual_roughness;
        self.perceptual_roughness = base_roughness * (1.0 - wetness * 0.7);

        let base_reflectance = self.reflectance;
        self.reflectance = (base_reflectance + wetness * 0.5).min(1.0);

        // Add slight metallic property when very wet
        self.metallic = wetness * 0.1;

        // Darken the surface when wet
        let current_color = self.base_color.to_srgba();
        self.base_color = Color::srgb(
            current_color.red * (1.0 - wetness * 0.3),
            current_color.green * (1.0 - wetness * 0.3),
            current_color.blue * (1.0 - wetness * 0.2),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::Assets;

    #[test]
    fn test_road_surface_properties() {
        assert_eq!(RoadSurfaceType::Asphalt.roughness(), 0.8);
        assert_eq!(RoadSurfaceType::WetAsphalt.roughness(), 0.2);
        assert!(RoadSurfaceType::WetAsphalt.reflectance() > RoadSurfaceType::Asphalt.reflectance());
        assert!(
            RoadSurfaceType::WetAsphalt.friction_coefficient()
                < RoadSurfaceType::Asphalt.friction_coefficient()
        );
    }

    #[test]
    fn test_marking_properties() {
        assert_eq!(
            MarkingType::WhiteLine.base_color(),
            Color::srgb(0.95, 0.95, 0.95)
        );
        assert_eq!(
            MarkingType::YellowLine.base_color(),
            Color::srgb(1.0, 0.9, 0.0)
        );
        assert!(MarkingType::ReflectiveLine.reflectance() > MarkingType::WhiteLine.reflectance());
    }

    #[test]
    fn test_material_factory() {
        let mut materials = Assets::default();

        let asphalt_material =
            RoadMaterialFactory::create_road_material(RoadSurfaceType::Asphalt, &mut materials);

        let white_line_material =
            RoadMaterialFactory::create_marking_material(MarkingType::WhiteLine, &mut materials);

        assert!(!asphalt_material.is_weak());
        assert!(!white_line_material.is_weak());
        assert_ne!(asphalt_material.id(), white_line_material.id());
    }

    #[test]
    fn test_weather_adjusted_material() {
        let mut materials = Assets::default();

        let dry_material = RoadMaterialFactory::create_weather_adjusted_material(
            RoadSurfaceType::Asphalt,
            0.0,
            &mut materials,
        );

        let wet_material = RoadMaterialFactory::create_weather_adjusted_material(
            RoadSurfaceType::Asphalt,
            1.0,
            &mut materials,
        );

        assert_ne!(dry_material.id(), wet_material.id());
    }

    #[test]
    fn test_material_library_creation() {
        let mut materials = Assets::default();
        let library = RoadMaterialLibrary::new(&mut materials);

        assert!(library.surfaces.contains_key(&RoadSurfaceType::Asphalt));
        assert!(library.surfaces.contains_key(&RoadSurfaceType::Concrete));
        assert!(library.markings.contains_key(&MarkingType::WhiteLine));
        assert!(library.markings.contains_key(&MarkingType::YellowLine));
    }

    #[test]
    fn test_material_extension_trait() {
        let mut material = StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.5,
            reflectance: 0.2,
            ..default()
        };

        let original_roughness = material.perceptual_roughness;
        let original_reflectance = material.reflectance;

        material.apply_wear(0.5);

        assert!(material.perceptual_roughness > original_roughness);
        assert!(material.reflectance < original_reflectance);

        material.apply_weather(0.8);

        assert!(material.perceptual_roughness < original_roughness); // Wet reduces roughness
        assert!(material.metallic > 0.0); // Wet adds metallic property
    }
}
