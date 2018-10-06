use hitable::Hitable;
use nalgebra::core::Vector3;
use rand;
use std::f64;
use world::World;


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


pub fn random_point_in_sphere() -> Vector3<f64> {
    let x = rand::random::<f64>();
    let y = rand::random::<f64>();
    let z = rand::random::<f64>();

    let distribution = 1.0 / (x * x + y * y + z * z).sqrt();
    let random_unit_sphere_point = distribution * Vector3::new(x, y, z);

    random_unit_sphere_point
}

                return (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0);
            }
        }
}
