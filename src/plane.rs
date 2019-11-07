use std::f32;
use std::sync::Arc;

use nalgebra::core::Vector3;
use rand::rngs::ThreadRng;
use rand::Rng;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;

#[derive(Clone)]
pub enum Axis {
    XY,
    YZ,
    XZ,
}

#[derive(Clone)]
pub struct Plane {
    axis: Axis,
    r0: f32,
    r1: f32,
    s0: f32,
    s1: f32,
    k: f32,
    material: Arc<dyn Material>,
}

impl Plane {
    pub fn new<M: Material + 'static>(axis: Axis,
                                      r0: f32,
                                      r1: f32,
                                      s0: f32,
                                      s1: f32,
                                      k: f32,
                                      material: M)
                                      -> Plane {
        let material = Arc::new(material);
        Plane { axis,
                r0,
                r1,
                s0,
                s1,
                k,
                material }
    }

    pub fn from_box(axis: Axis,
                    r0: f32,
                    r1: f32,
                    s0: f32,
                    s1: f32,
                    k: f32,
                    material: Arc<dyn Material>)
                    -> Plane {
        Plane { axis,
                r0,
                r1,
                s0,
                s1,
                k,
                material }
    }
}

impl Hitable for Plane {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        match self.axis {
            Axis::XY => {
                let t = (self.k - ray.origin.z) / ray.direction.z;

                if t < position_min || t > position_max {
                    return None;
                }

                let x = ray.origin.x + t * ray.direction.x;
                let y = ray.origin.y + t * ray.direction.y;

                if x < self.r0 || x > self.r1 || y < self.s0 || y > self.s1 {
                    return None;
                }

                let normal = Vector3::new(0.0, 0.0, 1.0);

                let record = HitRecord::new(t,
                                            (x - self.r0) / (self.r1 - self.r0),
                                            (y - self.s0) / (self.s1 - self.s0),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material.clone());

                Some(record)
            }
            Axis::YZ => {
                let t = (self.k - ray.origin.x) / ray.direction.x;

                if t < position_min || t > position_max {
                    return None;
                }

                let y = ray.origin.y + t * ray.direction.y;
                let z = ray.origin.z + t * ray.direction.z;

                if y < self.r0 || y > self.r1 || z < self.s0 || z > self.s1 {
                    return None;
                }

                let normal = Vector3::new(1.0, 0.0, 0.0);

                let record = HitRecord::new(t,
                                            (y - self.r0) / (self.r1 - self.r0),
                                            (z - self.s0) / (self.s1 - self.s0),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material.clone());

                Some(record)
            }
            Axis::XZ => {
                let t = (self.k - ray.origin.y) / ray.direction.y;

                if t < position_min || t > position_max {
                    return None;
                }

                let x = ray.origin.x + t * ray.direction.x;
                let z = ray.origin.z + t * ray.direction.z;

                if x < self.r0 || x > self.r1 || z < self.s0 || z > self.s1 {
                    return None;
                }

                let normal = Vector3::new(0.0, 1.0, 0.0);

                let record = HitRecord::new(t,
                                            (x - self.r0) / (self.r1 - self.r0),
                                            (z - self.s0) / (self.s1 - self.s0),
                                            ray.point_at_parameter(t),
                                            normal,
                                            normal,
                                            self.material.clone());

                Some(record)
            }
        }
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        match self.axis {
            Axis::XY => {
                let minimum = Vector3::new(self.r0, self.s0, self.k - 0.0001);
                let maximum = Vector3::new(self.r1, self.s1, self.k + 0.0001);
                Some(AABB::from(minimum, maximum))
            }
            Axis::YZ => {
                let minimum = Vector3::new(self.k - 0.0001, self.r0, self.s0);
                let maximum = Vector3::new(self.k + 0.0001, self.r1, self.s1);
                Some(AABB::from(minimum, maximum))
            }
            Axis::XZ => {
                let minimum = Vector3::new(self.r0, self.k - 0.0001, self.s0);
                let maximum = Vector3::new(self.r1, self.k + 0.0001, self.s1);
                Some(AABB::from(minimum, maximum))
            }
        }
    }

    fn pdf_value(&self, origin: &Vector3<f32>, direction: &Vector3<f32>) -> f32 {
        if let Some(hit) = self.hit(&Ray::new(*origin, *direction, 0.0), 0.001, f32::MAX) {
            let area = (self.r1 - self.r0) * (self.s1 - self.s0);
            let distance_squared = hit.parameter * hit.parameter * direction.norm_squared();
            let cosine = direction.dot(&hit.shading_normal).abs() / direction.norm();
            distance_squared / (cosine * area)
        } else {
            0.0
        }
    }

    fn pdf_random(&self, origin: &Vector3<f32>, rng: &mut ThreadRng) -> Vector3<f32> {
        let random_point = Vector3::new(self.r0 + rng.gen::<f32>() * (self.r1 - self.r0),
                                        self.k,
                                        self.s0 + rng.gen::<f32>() * (self.s1 - self.s0));
        random_point - origin
    }
}
