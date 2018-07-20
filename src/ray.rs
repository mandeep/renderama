extern crate nalgebra;

use nalgebra::core::Vector3;


pub struct Ray {
    pub origin: Vector3<f64>,
    pub direction: Vector3<f64>
}


impl Ray {
    pub fn new(origin: Vector3<f64>, direction: Vector3<f64>) -> Ray {
        Ray { origin: origin, direction: direction }
    }

    pub fn point_at_perimeter(&self, point: f64) -> Vector3<f64> {
        self.origin + point * self.direction
    }
}
