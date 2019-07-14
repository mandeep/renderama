use nalgebra::core::Vector3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;

#[derive(Clone)]
pub enum Plane {
    XY,
    YZ,
    XZ,
}

#[derive(Clone)]
pub struct Rectangle {
    plane: Plane,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
    material: Box<dyn Material>,
}

impl Rectangle {
    pub fn new<M: Material + 'static>(plane: Plane,
                                      x0: f32,
                                      x1: f32,
                                      y0: f32,
                                      y1: f32,
                                      k: f32,
                                      material: M)
                                      -> Rectangle {
        let material = Box::new(material);
        Rectangle { plane,
                    x0,
                    x1,
                    y0,
                    y1,
                    k,
                    material }
    }
}

impl Hitable for Rectangle {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let mut t = 0.0;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut normal: Vector3<f32> = Vector3::zeros();
        match self.plane {
            Plane::XY => {
                t = (self.k - ray.origin.z) / ray.direction.z;
                x = ray.origin.x + t * ray.direction.x;
                y = ray.origin.y + t * ray.direction.y;
                normal = Vector3::new(0.0, 0.0, 1.0);
            }
            Plane::YZ => {
                t = (self.k - ray.origin.x) / ray.direction.x;
                x = ray.origin.y + t * ray.direction.y;
                y = ray.origin.z + t * ray.direction.z;
                normal = Vector3::new(1.0, 0.0, 0.0);
            }
            Plane::XZ => {
                t = (self.k - ray.origin.y) / ray.direction.y;
                x = ray.origin.x + t * ray.direction.x;
                y = ray.origin.z + t * ray.direction.z;
                normal = Vector3::new(0.0, 1.0, 0.0);
            }
        }

        if t < position_min || t > position_max {
            return None;
        }

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let record = HitRecord::new(t,
                                    (x - self.x0) / (self.x1 - self.x0),
                                    (y - self.y0) / (self.y1 - self.y0),
                                    ray.point_at_parameter(t),
                                    normal,
                                    self.material.box_clone());

        Some(record)
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        let minimum = Vector3::new(self.x0, self.y0, self.k - 0.0001);
        let maximum = Vector3::new(self.x1, self.y1, self.k + 0.0001);
        Some(AABB::new(minimum, maximum))
    }

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(self.clone())
    }
}
