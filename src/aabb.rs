use nalgebra::Vector3;

use ray::Ray;

#[derive(Clone)]
pub struct AABB {
    pub minimum: Vector3<f32>,
    pub maximum: Vector3<f32>,
}

impl AABB {
    pub fn new(minimum: Vector3<f32>, maximum: Vector3<f32>) -> AABB {
        AABB { minimum, maximum }
    }

    /// Perform an intersection test with an AABB
    /// Reference: https://medium.com/@bromanz/another-view-on-the-classic-ray
    /// -aabb-intersection-algorithm-for-bvh-traversal-41125138b525
    pub fn hit(&self, ray: &Ray, _position_min: f32, _position_max: f32) -> bool {
        let t0 = (self.minimum - ray.origin).component_mul(&ray.inverse_direction);
        let t1 = (self.maximum - ray.origin).component_mul(&ray.inverse_direction);

        let tmin = t0.zip_map(&t1, |a, b| a.min(b));
        let tmax = t0.zip_map(&t1, |a, b| a.max(b));

        tmin.max() <= tmax.min()
    }
}

pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
    let small = box0.minimum.zip_map(&box1.minimum, |a, b| a.min(b));
    let big = box0.maximum.zip_map(&box1.maximum, |a, b| a.max(b));

    AABB::new(small, big)
}
