use std::f32;
use std::sync::Arc;

use glam::Vec3A;
use rand::{Rng, RngExt};

use crate::aabb::AABB;
use crate::primitive::Primitive;
use crate::materials::MaterialId;
use crate::ray::Ray;
use crate::results::HitResult;

#[derive(Clone)]
pub struct Volume {
    density: f32,
    boundary: Arc<Primitive>,
    material_id: MaterialId,
}

impl Volume {
    pub fn new(density: f32, boundary: impl Into<Primitive>, material_id: MaterialId) -> Volume {
        let boundary = Arc::new(boundary.into());
        Volume { density, boundary, material_id }
    }

    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut impl Rng) -> Option<HitResult> {
        // Find both intersections of the ray with the volume's boundary.
        // We search the entire ray range (not just [start_distance, end_distance]) because
        // a ray origin inside the volume would miss the near boundary otherwise.
        // We then clamp against [start_distance, end_distance] below.
        let mut entry_hit = self.boundary.hit(ray, f32::NEG_INFINITY, f32::INFINITY, rng)?;
        // a volume can be hit anywhere inside it, not just the surface like other geometry
        let mut exit_hit = self.boundary.hit(ray, entry_hit.parameter + 1e-4, f32::INFINITY, rng)?;

        if entry_hit.parameter < start_distance { entry_hit.parameter = start_distance };
        if exit_hit.parameter > end_distance { exit_hit.parameter = end_distance };

        if entry_hit.parameter < exit_hit.parameter {
            let distance_inside_boundary = (exit_hit.parameter - entry_hit.parameter) * ray.direction.length();
            let hit_distance = -(1.0 / self.density) * rng.random::<f32>().ln();

            if hit_distance < distance_inside_boundary {
                let parameter = entry_hit.parameter + hit_distance / ray.direction.length();
                let point = ray.point_at_parameter(parameter);
                 // arbitrary normal is used since light is scattered equally in all directions
                 // regardless of surface orientation
                let normal = Vec3A::new(1.0, 0.0, 0.0);
                return Some(
                    HitResult::new(parameter, 0.0, 0.0, point, normal, normal, self.material_id)
                );
            }
        }

        None
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        self.boundary.bounding_box()
    }
}
