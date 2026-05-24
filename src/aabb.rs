use std::f32;

use glam::Vec3A;

use ray::Ray;

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
    pub fn surface_area(&self) -> f32 {
        let diff = self.maximum - self.minimum;
        2.0 * (diff.x * diff.y + diff.y * diff.z + diff.z * diff.x)
    }

    /// Find the longest axis of the bounding box
    pub fn longest_axis(&self) -> usize {
        let diff = self.maximum - self.minimum;

        diff.as_ref()
            .iter()
            .position(|&e| e == diff.max_element())
            .unwrap()
    }

    /// Perform an intersection test with an AABB
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
