use nalgebra::Vector3;

use hitable::Hitable;
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

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> bool {
        for i in 0..3 {
            let inverse_direction = 1.0 / ray.direction[i];
            let mut t0 = (self.minimum[i] - ray.origin[i]) * inverse_direction;
            let mut t1 = (self.maximum[i] - ray.origin[i]) * inverse_direction;

            if inverse_direction < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            let tmin = t0.max(position_min);
            let tmax = t1.min(position_max);

            if tmax <= tmin {
                return false;
            }
        }
        return true;
    }
}

pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
    let small = Vector3::new(box0.minimum.x.min(box1.minimum.x),
                             box0.minimum.y.min(box1.minimum.y),
                             box0.minimum.z.min(box1.minimum.z),);
    let big = Vector3::new(box0.maximum.x.max(box1.maximum.x),
                           box0.maximum.y.max(box1.maximum.y),
                           box0.maximum.z.max(box1.maximum.z),);

    AABB::new(small, big)
}
