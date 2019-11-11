use std::f32;

use glam::Vec3;

use ray::Ray;

#[derive(Clone)]
pub struct AABB {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl AABB {
    /// Create an empty AABB from a zero vector and one vector
    pub fn new() -> AABB {
        AABB { minimum: Vec3::splat(f32::MAX),
               maximum: Vec3::splat(f32::MIN) }
    }

    /// Create a new AABB from the minimum and maximum slab vectors
    pub fn from(minimum: Vec3, maximum: Vec3) -> AABB {
        AABB { minimum, maximum }
    }

    pub fn area(&self) -> f32 {
        let diff = self.maximum - self.minimum;
        2.0 * (diff.x() * diff.y() + diff.y() * diff.z() + diff.z() * diff.x())
    }

    pub fn longest_axis(&self) -> usize {
        let diff = self.maximum - self.minimum;

        if diff.x() == diff.min_element() {
            return 0;
        } else if diff.y() == diff.min_element() {
            return 1;
        } else {
            return 2;
        }
    }

    /// Perform an intersection test with an AABB
    /// Reference: https://medium.com/@bromanz/another-view-on-the-classic-ray
    /// -aabb-intersection-algorithm-for-bvh-traversal-41125138b525
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
