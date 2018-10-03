use std::f64;
use nalgebra::core::Vector3;


pub struct Ray {
    pub origin: Vector3<f64>,
    pub direction: Vector3<f64>
}


impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(a: Vector3<f64>, b: Vector3<f64>) -> Ray {
        Ray { origin: a, direction: b }
    }

    pub fn point_at_parameter(&self, parameter: f64) -> Vector3<f64> {
        self.origin + parameter * self.direction
    }
}
