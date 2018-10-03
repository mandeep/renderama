extern crate nalgebra;

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

    pub fn color(&self) -> Vector3<f64> {
        let point: f64 = self.hit_sphere(&Vector3::new(0.0, 0.0, -1.0), 0.5);
        if point > 0.0 {
            let normal = (self.point_at_perimeter(point) - Vector3::new(0.0, 0.0, -1.0)).normalize();
            return 0.5 * normal.map(|coordinate| coordinate + 1.0);
        }

        let unit_direction: Vector3<f64> = self.direction.normalize();
        let point: f64 = 0.5 * (unit_direction.y + 1.0);

        (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0)
    }

    pub fn point_at_perimeter(&self, point: f64) -> Vector3<f64> {
        self.origin + point * self.direction
    }

    pub fn hit_sphere(&self, center: &Vector3<f64>, radius: f64) -> f64 {
        let sphere_center: Vector3<f64> = self.origin - center;
        let a: f64 = self.direction.dot(&self.direction);
        let b: f64 = 2.0 * sphere_center.dot(&self.direction);
        let c: f64 = sphere_center.dot(&sphere_center) - (radius * radius);
        let discriminant: f64 = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return -1.0;
        } else {
            return (-b - discriminant.sqrt()) / (2.0 * a);
        }
    }
}
