use std::f64;
use nalgebra::core::Vector3;
use hitable::{Hitable, HitRecord};


pub struct Ray {
    pub origin: Vector3<f64>,
    pub direction: Vector3<f64>
}


impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(a: Vector3<f64>, b: Vector3<f64>) -> Ray {
        Ray { origin: a, direction: b }
    }

    pub fn color(self, world: &Hitable) -> Vector3<f64> {
        let mut hit_record = HitRecord::new(f64::MAX);
        if world.hit(&self, 0.0, f64::MAX, &mut hit_record) {
            let normal: Vector3<f64> = hit_record.normal;
            return 0.5 * normal.map(|coordinate| coordinate + 1.0);
        }
        let unit_direction: Vector3<f64> = self.direction.normalize();
        let point: f64 = 0.5 * (unit_direction.y + 1.0);

        (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0)
    }

    pub fn point_at_parameter(&self, point: f64) -> Vector3<f64> {
        self.origin + point * self.direction
    }
}
