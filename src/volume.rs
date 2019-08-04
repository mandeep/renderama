use std::f32;
use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::{Isotropic, Material};
use ray::Ray;
use texture::Texture;

pub struct Volume {
    density: f32,
    boundary: Arc<dyn Hitable>,
    material: Arc<dyn Material>
}

impl Volume {
    pub fn new<H: Hitable + 'static, T: Texture + 'static>(density: f32, boundary: H, texture: T) -> Volume {
        let boundary = Arc::new(boundary);
        let material = Arc::new(Isotropic::new(texture));
        Volume { density, boundary, material }
    }
}

impl Hitable for Volume {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some(mut hit1) = self.boundary.hit(&ray, f32::MIN, f32::MAX) {
            if let Some(mut hit2) = self.boundary.hit(&ray, hit1.parameter + 0.0001, f32::MAX) {
                if hit1.parameter < t_min { hit1.parameter = t_min };
                if hit2.parameter > t_max { hit2.parameter = t_max };
                if hit1.parameter < hit2.parameter {
                    let distance_inside_boundary = (hit2.parameter - hit1.parameter) * ray.direction.norm();
                    let hit_distance = -(1.0 / self.density) * rand::random::<f32>().ln();

                    if hit_distance < distance_inside_boundary {
                        let t = hit1.parameter + hit_distance / ray.direction.norm();
                        let point = ray.point_at_parameter(t);
                        let normal = Vector3::new(1.0, 0.0, 0.0);
                        return Some(HitRecord::new(t, 0.0, 0.0, point, normal, self.material.clone()));
                    }
                }
            }
        }
        None
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        self.boundary.bounding_box(t0, t1)
    }
}
