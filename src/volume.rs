use std::f32;

use glam::Vec3A;

use aabb::AABB;
use events::HitEvent;
use primitive::Primitive;
use materials::MaterialId;
use ray::Ray;

#[derive(Clone)]
pub struct Volume {
    density: f32,
    boundary: Box<Primitive>,
    material_id: MaterialId,
}

impl Volume {
    pub fn new(density: f32, boundary: Primitive, material_id: MaterialId) -> Volume {
        let boundary = Box::new(boundary);
        Volume { density, boundary, material_id }
    }

    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32) -> Option<HitEvent> {
        // Find both intersections of the ray with the volume's boundary.
        // We search the entire ray range (not just [start_distance, end_distance]) because
        // a ray origin inside the volume would miss the near boundary otherwise.
        // We then clamp against [start_distance, end_distance] below.
        if let Some(mut hit1) = self.boundary.hit(ray, f32::NEG_INFINITY, f32::INFINITY) {
            if let Some(mut hit2) =
                self.boundary.hit(ray, hit1.parameter + 0.0001, f32::INFINITY)
            {
                if hit1.parameter < start_distance {
                    hit1.parameter = start_distance
                };
                if hit2.parameter > end_distance {
                    hit2.parameter = end_distance
                };
                if hit1.parameter < hit2.parameter {
                    let distance_inside_boundary =
                        (hit2.parameter - hit1.parameter) * ray.direction.length();
                    let hit_distance = -(1.0 / self.density) * rand::random::<f32>().ln();

                    if hit_distance < distance_inside_boundary {
                        let t = hit1.parameter + hit_distance / ray.direction.length();
                        let point = ray.point_at_parameter(t);
                        let normal = Vec3A::new(1.0, 0.0, 0.0);
                        return Some(HitEvent::new(t,
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

    pub fn bounding_box(&self) -> Option<AABB> {
        self.boundary.bounding_box()
    }
}
