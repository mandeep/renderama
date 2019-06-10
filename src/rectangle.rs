use nalgebra::core::Vector3;

use hitable::{Hitable, HitRecord};
use materials::Material;
use ray::Ray;


pub struct Rectangle {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k:  f64,
    material: Box<dyn Material>
}


impl Rectangle {

    pub fn new<M: Material + 'static>(x0: f64, x1: f64, y0: f64, y1: f64, k: f64,
                                  material: M) -> Rectangle {
        let material = Box::new(material);
        Rectangle { x0: x0, x1: x1, y0: y0, y1: y1, k: k, material: material }
    }
}


impl Hitable for Rectangle {

    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord> {
        let t = (self.k - ray.origin.z) / ray.direction.z;
        if t < position_min || t > position_max {
            return None;
        }

        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let record = HitRecord::new(t,
                                    (x - self.x0) / (self.x1 - self.x0),
                                    (y - self.y0) / (self.y1 - self.y0),
                                    ray.point_at_parameter(t),
                                    Vector3::new(0.0, 0.0, 1.0),
                                    self.material.box_clone());

        Some(record)
    }
}
