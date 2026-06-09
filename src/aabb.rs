use std::f32;

use glam::Vec3A;

use crate::ray::Ray;

#[derive(Clone, Copy)]
/// Axis-aligned bounding boxes are used to subdivide objects in the
/// scene. Since ray-object intersection is the most costly computation
/// in a ray tracer, we use AABBs to optimize that cost. When used in a BVH,
/// the search for objects that a ray hits decreases from O(n) to O(logn).
pub struct AABB {
    pub minimum: Vec3A,
    pub maximum: Vec3A,
}

impl AABB {
    /// Create a new AABB from the minimum and maximum slab vectors
    pub fn from(minimum: Vec3A, maximum: Vec3A) -> AABB {
        AABB { minimum, maximum }
    }

    /// Calculate the surface area of the bounding box
    ///
    /// Total surface area (TSA) of a cuboid is calculated
    /// as 2(lw + wh + lh).
    pub fn surface_area(&self) -> f32 {
        let faces = self.maximum - self.minimum;
        2.0 * (faces.x * faces.y + faces.y * faces.z + faces.z * faces.x)
    }

    /// Find the longest axis of the bounding box
    ///
    /// If all axes are equal, then axis 0 is returned.
    pub fn longest_axis(&self) -> usize {
        let diff = self.maximum - self.minimum;

        diff.as_ref()
            .iter()
            .position(|&e| e == diff.max_element())
            .unwrap()
    }

    /// Perform an intersection test with an AABB
    ///
    /// References:
    /// https://medium.com/@bromanz/another-view-on-the-classic-ray-aabb-intersection-algorithm-for-bvh-traversal-41125138b525
    /// https://jcgt.org/published/0007/03/04/
    pub fn hit(&self, ray: &Ray, _position_min: f32, _position_max: f32) -> bool {
        let t0 = (self.minimum - ray.origin) * ray.inverse_direction;
        let t1 = (self.maximum - ray.origin) * ray.inverse_direction;

        let tmin = t0.min(t1);
        let tmax = t1.max(t0);

        tmin.max_element() <= tmax.min_element()
    }

    /// Create an AABB that encapsulates two volumes
    pub fn surrounding_box(&self, other: &AABB) -> AABB {
        let small = self.minimum.min(other.minimum);
        let big = self.maximum.max(other.maximum);

        AABB::from(small, big)
    }
}


#[cfg(test)]
mod tests {
    use glam::Vec3A;
    use super::{AABB, Ray};

    #[test]
    fn test_from_stores_min_max() {
        let min = Vec3A::new(1.0, 2.0, 3.0);
        let max = Vec3A::new(4.0, 5.0, 6.0);
        let aabb = AABB::from(min, max);
        assert_eq!(aabb.minimum, min);
        assert_eq!(aabb.maximum, max);
    }

    #[test]
    fn test_surface_area() {
        // surface area of box is 2(lb + lh + bh), or 52 in this case
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::new(2.0, 3.0, 4.0));
        assert!((aabb.surface_area() - 52.0).abs() < 1e-4);
    }

    #[test]
    fn test_longest_axis_x() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::new(10.0, 2.0, 3.0));
        assert_eq!(aabb.longest_axis(), 0);
    }

    #[test]
    fn test_longest_axis_y() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::new(1.0, 9.0, 2.0));
        assert_eq!(aabb.longest_axis(), 1);
    }

    #[test]
    fn test_longest_axis_z() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::new(1.0, 2.0, 8.0));
        assert_eq!(aabb.longest_axis(), 2);
    }

    #[test]
    fn test_longest_axis_all_equal_returns_valid() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::ONE);
        let axis = aabb.longest_axis();
        assert_eq!(axis, 0);
    }

    #[test]
    fn test_hit_ray_through_center() {
        let aabb = AABB::from(Vec3A::new(-1.0, -1.0, -1.0), Vec3A::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3A::new(0.0, 0.0, -5.0), Vec3A::new(0.0, 0.0, 1.0), 0.0);
        assert!(aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_hit_ray_along_y_axis() {
        let aabb = AABB::from(Vec3A::new(-1.0, -1.0, -1.0), Vec3A::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3A::new(0.0, -5.0, 0.0), Vec3A::new(0.0, 1.0, 0.0), 0.0);
        assert!(aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_hit_ray_along_z_axis() {
        let aabb = AABB::from(Vec3A::new(-1.0, -1.0, -1.0), Vec3A::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Vec3A::new(0.0, 0.0, -5.0), Vec3A::new(0.0, 0.0, 1.0), 0.0);
        assert!(aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_hit_ray_diagonal() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::new(2.0, 2.0, 2.0));
        let ray = Ray::new(Vec3A::new(-5.0, -5.0, -5.0), Vec3A::new(1.0, 1.0, 1.0), 0.0);
        assert!(aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_missed_ray() {
        let aabb = AABB::from(Vec3A::new(10.0, 10.0, 10.0), Vec3A::new(11.0, 11.0, 11.0));
        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 1.0, 0.0), 0.0);
        assert!(!aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_missed_grazing_ray() {
        let aabb = AABB::from(Vec3A::ZERO, Vec3A::ONE);
        let ray = Ray::new(Vec3A::ZERO, Vec3A::new(0.0, 1.0, 0.0),0.0);
        assert!(!aabb.hit(&ray, 0.0, f32::INFINITY));
    }

    #[test]
    fn test_surrounding_box_basic() {
        let a = AABB::from(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(1.0, 1.0, 1.0));
        let b = AABB::from(Vec3A::new(2.0, 2.0, 2.0), Vec3A::new(3.0, 3.0, 3.0));
        let s = a.surrounding_box(&b);
        assert_eq!(s.minimum, Vec3A::new(0.0, 0.0, 0.0));
        assert_eq!(s.maximum, Vec3A::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_surrounding_box_overlapping() {
        let a = AABB::from(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(2.0, 2.0, 2.0));
        let b = AABB::from(Vec3A::new(1.0, 1.0, 1.0), Vec3A::new(3.0, 3.0, 3.0));
        let s = a.surrounding_box(&b);
        assert_eq!(s.minimum, Vec3A::new(0.0, 0.0, 0.0));
        assert_eq!(s.maximum, Vec3A::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_surrounding_box_one_contains_other() {
        let outer = AABB::from(Vec3A::new(-5.0, -5.0, -5.0), Vec3A::new(5.0, 5.0, 5.0));
        let inner = AABB::from(Vec3A::ZERO, Vec3A::ONE);
        let s = outer.surrounding_box(&inner);
        assert_eq!(s.minimum, outer.minimum);
        assert_eq!(s.maximum, outer.maximum);
    }

    #[test]
    fn test_surrounding_box_identical() {
        let a = AABB::from(Vec3A::ZERO, Vec3A::ONE);
        let s = a.surrounding_box(&a);
        assert_eq!(s.minimum, a.minimum);
        assert_eq!(s.maximum, a.maximum);
    }
}