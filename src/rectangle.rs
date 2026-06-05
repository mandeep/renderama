use glam::Vec3A;
use rand_pcg::Pcg64Mcg;

use aabb::AABB;
use results::HitResult;
use primitive::Primitive;
use materials::MaterialId;
use plane::{Axis, Bounds2D, Plane};
use ray::Ray;


#[derive(Clone)]
/// The Rectangle struct allows for easy creation of a rectangular prism,
/// a cuboid with six rectanglular faces.
pub struct Rectangle {
    p0: Vec3A,
    p1: Vec3A,
    primitives: Vec<Primitive>,
}

impl Rectangle {
    /// Create a new rectangular prism from the given vectors and material.
    ///
    /// # Example
    ///
    /// let p0 = Vec3A::new(0.0, 0.0, 0.0);
    /// let p1 = Vec3A::new(100.0, 200.0, 300.0);
    /// let mat_idx = mat!(...);
    ///
    /// let rectangle = Rectangle::new(p0, p1, mat_idx);
    ///
    /// This creates a Rectangle with an XY plane with x in [0.0, 100.0] and y in [0.0, 200.0]
    /// at both z = 0.0 and z = 300.0, an XZ plane with x in [0.0, 300.0] and z in [0.0, 300.0]
    /// at both y = 0.0 and y = 200.0, and a YZ plane with y in [0.0, 200.0] and z in
    /// [0.0, 300.0] at x = 0.0 and x = 100.0.
    pub fn new(p0: Vec3A, p1: Vec3A, material_id: MaterialId) -> Rectangle {
        let mut primitives: Vec<Primitive> = Vec::new();
        let xy_bounds = Bounds2D::new(p0.x..p1.x, p0.y..p1.y);
        let xz_bounds = Bounds2D::new(p0.x..p1.x, p0.z..p1.z);
        let yz_bounds = Bounds2D::new(p0.y..p1.y, p0.z..p1.z);

        primitives.push(Plane::new(Axis::XY, xy_bounds, p1.z, material_id).into_primitive());
        primitives.push(Plane::new(Axis::XY, xy_bounds, p0.z, material_id).into_reversed());
        primitives.push(Plane::new(Axis::XZ, xz_bounds, p1.y, material_id).into_primitive());
        primitives.push(Plane::new(Axis::XZ, xz_bounds, p0.y, material_id).into_reversed());
        primitives.push(Plane::new(Axis::YZ, yz_bounds, p1.x, material_id).into_primitive());
        primitives.push(Plane::new(Axis::YZ, yz_bounds, p0.x, material_id).into_reversed());

        Rectangle { p0, p1, primitives }
    }

    /// Iterate through each of the Plane primitives held in the primitives Vec and
    /// call their hit method. Return the closest hit if it exists.
    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32, rng: &mut Pcg64Mcg) -> Option<HitResult> {
        self.primitives
        .iter()
        .filter_map(|plane| plane.hit(ray, position_min, position_max, rng))
        .filter(|hit| hit.parameter.is_finite())
        .min_by(|a, b| a.parameter.partial_cmp(&b.parameter).unwrap())
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::from(self.p0, self.p1))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_setup() {
        let p0 = Vec3A::new(0.0, 0.0, 0.0);
        let p1 = Vec3A::new(100.0, 200.0, 300.0);
        let mat_idx = MaterialId(0);
        let rectangle = Rectangle::new(p0, p1, mat_idx);

        assert_eq!(rectangle.p0, p0);
        assert_eq!(rectangle.p1, p1);
        assert_eq!(rectangle.primitives.len(), 6);
    }

    #[test]
    fn test_rectangle_bbox() {
        let p0 = Vec3A::new(0.0, 0.0, 0.0);
        let p1 = Vec3A::new(2.0, 3.0, 5.0);
        let mat_idx = MaterialId(0);
        let rectangle = Rectangle::new(p0, p1, mat_idx);

        let aabb = rectangle.bounding_box().unwrap();
        assert_eq!(aabb.minimum, p0);
        assert_eq!(aabb.maximum, p1);
    }

    #[test]
    fn test_rectangle_hit() {
        use rand::SeedableRng;

        let p0 = Vec3A::ZERO;
        let p1 = Vec3A::ONE;
        let mat_idx = MaterialId(0);
        let rectangle = Rectangle::new(p0, p1, mat_idx);
        let mut rng = Pcg64Mcg::seed_from_u64(0);

        let ray = Ray::new(Vec3A::new(0.0, 0.0, 5.0), Vec3A::new(0.0, 0.0, -1.0), 0.0);
        assert!(rectangle.hit(&ray, 0.0, f32::MAX, &mut rng).is_some());
    }

    #[test]
    fn test_rectangle_miss() {
        use rand::SeedableRng;

        let p0 = Vec3A::ZERO;
        let p1 = Vec3A::ONE;
        let mat_idx = MaterialId(0);
        let rectangle = Rectangle::new(p0, p1, mat_idx);
        let mut rng = Pcg64Mcg::seed_from_u64(0);

        let ray = Ray::new(Vec3A::new(0.0, 0.0, 5.0), Vec3A::new(0.0, 0.0, 1.0), 0.0);
        assert!(rectangle.hit(&ray, 0.0, f32::MAX, &mut rng).is_none());
    }
}