use nalgebra::core::Vector3;
use ray::Ray;


pub struct Camera {
    pub lower_left_corner: Vector3<f64>,
    pub horizontal: Vector3<f64>,
    pub vertical: Vector3<f64>,
    pub origin: Vector3<f64>,
}


impl Camera {
    pub fn new(lower_left_corner: Vector3<f64>,
               horizontal: Vector3<f64>,
               vertical: Vector3<f64>,
               origin: Vector3<f64>) -> Camera {

        Camera { lower_left_corner: lower_left_corner,
                 horizontal: horizontal,
                 vertical: vertical,
                 origin: origin }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray { origin: self.origin,
              direction: self.lower_left_corner + u * self.horizontal + v * self.vertical -
                         self.origin }
    }
}
