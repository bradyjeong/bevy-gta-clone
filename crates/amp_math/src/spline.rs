/// High-performance spline mathematics for road system
/// Implements Catmull-Rom splines, linear interpolation, and curve evaluation
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Spline curve representation with control points and evaluation methods
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Spline {
    /// Control points defining the spline curve
    pub control_points: Vec<Vec3>,
    /// Cached length for performance optimization
    cached_length: Option<f32>,
}

impl Spline {
    /// Create a new spline with given control points
    pub fn new(control_points: Vec<Vec3>) -> Self {
        Self {
            control_points,
            cached_length: None,
        }
    }

    /// Create a linear spline between two points
    pub fn linear(start: Vec3, end: Vec3) -> Self {
        Self::new(vec![start, end])
    }

    /// Create a curved spline with a control point
    pub fn curved(start: Vec3, control: Vec3, end: Vec3) -> Self {
        Self::new(vec![start, control, end])
    }

    /// Create a circular spline centered at a point with given radius
    pub fn circle(center: Vec3, radius: f32) -> Self {
        let segments = 16;
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let x = center.x + radius * angle.cos();
            let z = center.z + radius * angle.sin();
            points.push(Vec3::new(x, center.y, z));
        }

        Self::new(points)
    }

    /// Add a control point (invalidates cached length)
    pub fn add_control_point(&mut self, point: Vec3) {
        self.control_points.push(point);
        self.cached_length = None;
    }

    /// Insert a control point at specific index (for curve shaping)
    pub fn insert_control_point(&mut self, index: usize, point: Vec3) {
        if index <= self.control_points.len() {
            self.control_points.insert(index, point);
            self.cached_length = None;
        }
    }

    /// Evaluate spline at parameter t (0.0 to 1.0)
    pub fn evaluate(&self, t: f32) -> Vec3 {
        let t = t.clamp(0.0, 1.0);

        match self.control_points.len() {
            0 => Vec3::ZERO,
            1 => self.control_points[0],
            2 => self.linear_interpolation(t),
            _ => self.catmull_rom_interpolation(t),
        }
    }

    /// Evaluate tangent vector at parameter t
    pub fn evaluate_tangent(&self, t: f32) -> Vec3 {
        let epsilon = 0.001;
        let t1 = (t - epsilon).max(0.0);
        let t2 = (t + epsilon).min(1.0);

        let p1 = self.evaluate(t1);
        let p2 = self.evaluate(t2);

        (p2 - p1).normalize_or_zero()
    }

    /// Get spline length (cached for performance)
    pub fn length(&self) -> f32 {
        if let Some(cached) = self.cached_length {
            return cached;
        }

        self.calculate_length()
    }

    /// Force recalculation of length cache
    pub fn recalculate_length(&mut self) -> f32 {
        let length = self.calculate_length();
        self.cached_length = Some(length);
        length
    }

    /// Linear interpolation between two points
    fn linear_interpolation(&self, t: f32) -> Vec3 {
        self.control_points[0].lerp(self.control_points[1], t)
    }

    /// Catmull-Rom spline interpolation for smooth curves
    fn catmull_rom_interpolation(&self, t: f32) -> Vec3 {
        let points = &self.control_points;
        let n = points.len();

        if n < 4 {
            // Fall back to linear interpolation for insufficient points
            return self.linear_interpolation(t);
        }

        // Find the segment
        let segment_count = n - 3;
        let segment_t = t * segment_count as f32;
        let segment_index = (segment_t.floor() as usize).min(segment_count - 1);
        let local_t = segment_t.fract();

        // Get control points for this segment
        let p0 = points[segment_index];
        let p1 = points[segment_index + 1];
        let p2 = points[segment_index + 2];
        let p3 = points[segment_index + 3];

        self.catmull_rom_segment(p0, p1, p2, p3, local_t)
    }

    /// Catmull-Rom interpolation for a single segment
    fn catmull_rom_segment(&self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;

        // Catmull-Rom formula
        0.5 * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
    }

    /// Calculate total spline length using sampling
    fn calculate_length(&self) -> f32 {
        if self.control_points.len() < 2 {
            return 0.0;
        }

        let samples = 100; // High precision for roads
        let mut length = 0.0;

        for i in 0..samples {
            let t1 = i as f32 / samples as f32;
            let t2 = (i + 1) as f32 / samples as f32;

            let p1 = self.evaluate(t1);
            let p2 = self.evaluate(t2);

            length += p1.distance(p2);
        }

        length
    }

    /// Sample points along the spline at regular intervals
    pub fn sample_points(&self, count: usize) -> Vec<Vec3> {
        if count == 0 {
            return Vec::new();
        }

        let mut points = Vec::with_capacity(count);

        for i in 0..count {
            let t = if count == 1 {
                0.0
            } else {
                i as f32 / (count - 1) as f32
            };
            points.push(self.evaluate(t));
        }

        points
    }

    /// Sample points along the spline at specified distance intervals
    pub fn sample_by_distance(&self, distance: f32) -> Vec<Vec3> {
        if distance <= 0.0 {
            return Vec::new();
        }

        let length = self.length();
        let count = (length / distance).ceil() as usize + 1;

        self.sample_points(count)
    }

    /// Find the closest point on the spline to a given position
    pub fn closest_point(&self, position: Vec3) -> (Vec3, f32) {
        const SAMPLES: usize = 100;

        let mut closest_point = Vec3::ZERO;
        let mut closest_distance = f32::INFINITY;
        let mut closest_t = 0.0;

        for i in 0..=SAMPLES {
            let t = i as f32 / SAMPLES as f32;
            let point = self.evaluate(t);
            let distance = position.distance(point);

            if distance < closest_distance {
                closest_distance = distance;
                closest_point = point;
                closest_t = t;
            }
        }

        (closest_point, closest_t)
    }

    /// Check if a point is within tolerance of the spline
    pub fn contains_point(&self, position: Vec3, tolerance: f32) -> bool {
        let (_, distance) = self.closest_point(position);
        distance <= tolerance
    }
}

/// Utilities for spline mathematics
pub mod utils {
    use super::*;

    /// Create a smooth curve through multiple waypoints
    pub fn create_smooth_path(waypoints: &[Vec3]) -> Spline {
        if waypoints.len() < 2 {
            return Spline::default();
        }

        if waypoints.len() == 2 {
            return Spline::linear(waypoints[0], waypoints[1]);
        }

        // Create control points for smooth interpolation
        let mut control_points = Vec::with_capacity(waypoints.len() + 2);

        // Add extended start point for better tangent calculation
        let start_tangent = (waypoints[1] - waypoints[0]).normalize();
        control_points.push(waypoints[0] - start_tangent * 10.0);

        // Add all waypoints
        control_points.extend_from_slice(waypoints);

        // Add extended end point for better tangent calculation
        let end_idx = waypoints.len() - 1;
        let end_tangent = (waypoints[end_idx] - waypoints[end_idx - 1]).normalize();
        control_points.push(waypoints[end_idx] + end_tangent * 10.0);

        Spline::new(control_points)
    }

    /// Calculate the curvature at a point on the spline
    pub fn calculate_curvature(spline: &Spline, t: f32) -> f32 {
        let epsilon = 0.001;
        let t1 = (t - epsilon).max(0.0);
        let t2 = (t + epsilon).min(1.0);

        let tangent1 = spline.evaluate_tangent(t1);
        let tangent2 = spline.evaluate_tangent(t2);

        let angular_change = tangent1.angle_between(tangent2);
        let arc_length = spline.evaluate(t1).distance(spline.evaluate(t2));

        if arc_length > 0.0 {
            angular_change / arc_length
        } else {
            0.0
        }
    }

    /// Calculate banking angle for a road based on curvature
    pub fn calculate_banking_angle(curvature: f32, speed: f32) -> f32 {
        const MAX_BANKING: f32 = 0.2; // ~11 degrees
        const GRAVITY: f32 = 9.81;

        if curvature > 0.0 {
            let radius = 1.0 / curvature;
            let banking = (speed * speed / (GRAVITY * radius)).atan();
            banking.min(MAX_BANKING)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_spline() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(10.0, 0.0, 0.0);
        let spline = Spline::linear(start, end);

        assert_eq!(spline.evaluate(0.0), start);
        assert_eq!(spline.evaluate(1.0), end);
        assert_eq!(spline.evaluate(0.5), Vec3::new(5.0, 0.0, 0.0));
        assert!((spline.length() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_curved_spline() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let control = Vec3::new(5.0, 5.0, 0.0);
        let end = Vec3::new(10.0, 0.0, 0.0);
        let spline = Spline::curved(start, control, end);

        assert_eq!(spline.evaluate(0.0), start);
        assert_eq!(spline.evaluate(1.0), end);
        assert!(spline.length() > 10.0); // Curved path is longer
    }

    #[test]
    fn test_tangent_calculation() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(10.0, 0.0, 0.0);
        let spline = Spline::linear(start, end);

        let tangent = spline.evaluate_tangent(0.5);
        assert!((tangent - Vec3::X).length() < 0.01);
    }

    #[test]
    fn test_closest_point() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(10.0, 0.0, 0.0);
        let spline = Spline::linear(start, end);

        let test_point = Vec3::new(5.0, 2.0, 0.0);
        let (closest, _) = spline.closest_point(test_point);

        assert!((closest - Vec3::new(5.0, 0.0, 0.0)).length() < 0.01);
    }

    #[test]
    fn test_sample_points() {
        let spline = Spline::linear(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0));
        let points = spline.sample_points(5);

        assert_eq!(points.len(), 5);
        assert_eq!(points[0], Vec3::ZERO);
        assert_eq!(points[4], Vec3::new(10.0, 0.0, 0.0));
    }
}
