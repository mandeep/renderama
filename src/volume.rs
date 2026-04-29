use std::f32;
use std::sync::Arc;

use glam::Vec3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::{Isotropic};
use ray::Ray;
use texture::Texture;

pub struct Volume {
    density: f32,
    boundary: Arc<dyn Hitable>,
    material_id: u32,
}

impl Volume {
    pub fn new<H: Hitable + 'static>(density: f32, boundary: H, material_id: u32) -> Volume {
        let boundary = Arc::new(boundary);

        Volume { density, boundary, material_id }
    }
}

impl Hitable for Volume {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // Find both intersections of the ray with the volume's boundary.
        // We search the entire ray range (not just [t_min, t_max]) because
        // a ray origin inside the volume would miss the near boundary otherwise.
        // We then clamp against [t_min, t_max] below.
        if let Some(mut hit1) = self.boundary.hit(&ray, f32::NEG_INFINITY, f32::INFINITY) {
            if let Some(mut hit2) =
                self.boundary.hit(&ray, hit1.parameter + 0.0001, f32::INFINITY)
            {
                if hit1.parameter < t_min {
                    hit1.parameter = t_min
                };
                if hit2.parameter > t_max {
                    hit2.parameter = t_max
                };
                if hit1.parameter < hit2.parameter {
                    let distance_inside_boundary =
                        (hit2.parameter - hit1.parameter) * ray.direction.length();
                    let hit_distance = -(1.0 / self.density) * rand::random::<f32>().ln();

                    if hit_distance < distance_inside_boundary {
                        let t = hit1.parameter + hit_distance / ray.direction.length();
                        let point = ray.point_at_parameter(t);
                        let normal = Vec3::new(1.0, 0.0, 0.0);
                        return Some(HitRecord::new(t,
                                                   0.0,
                                                   0.0,
                                                   point,
                                                   normal,
                                                   normal,
                                                   self.material_id));
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
