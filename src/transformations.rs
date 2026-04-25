use std::f32;
use std::sync::Arc;

use glam::Vec3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use ray::Ray;

pub struct Translate {
    offset: Vec3,
    hitable: Arc<dyn Hitable>,
}

impl Translate {
    pub fn new<H: Hitable + 'static>(offset: Vec3, hitable: H) -> Translate {
        let hitable = Arc::new(hitable);
        Translate { offset, hitable }
    }
}

impl Hitable for Translate {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        if let Some(mut hit) = self.hitable.hit(&moved_ray, position_min, position_max) {
            hit.point += self.offset;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            bbox.minimum += self.offset;
            bbox.maximum += self.offset;
            Some(bbox)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Rotate {
    cos_theta_x: f32, sin_theta_x: f32,
    cos_theta_y: f32, sin_theta_y: f32,
    cos_theta_z: f32, sin_theta_z: f32,
    hitable: Arc<dyn Hitable>,
}

impl Rotate {
    pub fn new<H: Hitable + 'static>(theta_x: f32, theta_y: f32, theta_z: f32, hitable: H) -> Rotate {
        let (tx, ty, tz) = (theta_x.to_radians(), theta_y.to_radians(), theta_z.to_radians());
        Rotate {
            cos_theta_x: tx.cos(), sin_theta_x: tx.sin(),
            cos_theta_y: ty.cos(), sin_theta_y: ty.sin(),
            cos_theta_z: tz.cos(), sin_theta_z: tz.sin(),
            hitable: Arc::new(hitable),
        }
    }

    /// Forward rotation: applies X, then Y, then Z (extrinsic order)
    fn rotate(&self, v: &Vec3) -> Vec3 {
        // Rotate around X
        let v = Vec3::new(
            v.x(),
            self.cos_theta_x * v.y() - self.sin_theta_x * v.z(),
            self.sin_theta_x * v.y() + self.cos_theta_x * v.z(),
        );
        // Rotate around Y (matches original Rotate sign convention)
        let v = Vec3::new(
            self.cos_theta_y * v.x() - self.sin_theta_y * v.z(),
            v.y(),
            self.sin_theta_y * v.x() + self.cos_theta_y * v.z(),
        );
        // Rotate around Z
        Vec3::new(
            self.cos_theta_z * v.x() - self.sin_theta_z * v.y(),
            self.sin_theta_z * v.x() + self.cos_theta_z * v.y(),
            v.z(),
        )
    }

    /// Inverse rotation: applies Z⁻¹, then Y⁻¹, then X⁻¹
    fn rotate_inv(&self, v: &Vec3) -> Vec3 {
        // Inverse Z
        let v = Vec3::new(
            self.cos_theta_z * v.x() + self.sin_theta_z * v.y(),
            -self.sin_theta_z * v.x() + self.cos_theta_z * v.y(),
            v.z(),
        );
        // Inverse Y
        let v = Vec3::new(
            self.cos_theta_y * v.x() + self.sin_theta_y * v.z(),
            v.y(),
            -self.sin_theta_y * v.x() + self.cos_theta_y * v.z(),
        );
        // Inverse X
        Vec3::new(
            v.x(),
            self.cos_theta_x * v.y() + self.sin_theta_x * v.z(),
            -self.sin_theta_x * v.y() + self.cos_theta_x * v.z(),
        )
    }
}

impl Hitable for Rotate {
    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = self.rotate(&ray.origin);
        let direction = self.rotate(&ray.direction);
        let rotated_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.hitable.hit(&rotated_ray, t0, t1) {
            hit.point = self.rotate_inv(&hit.point);
            hit.shading_normal = self.rotate_inv(&hit.shading_normal);
            hit.geometric_normal = self.rotate_inv(&hit.geometric_normal);
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(bbox) = self.hitable.bounding_box(t0, t1) {
            let mut min = Vec3::splat(f32::MAX);
            let mut max = Vec3::splat(f32::MIN);
            for i in 0..8 {
                let x = if i & 1 != 0 { bbox.maximum.x() } else { bbox.minimum.x() };
                let y = if i & 2 != 0 { bbox.maximum.y() } else { bbox.minimum.y() };
                let z = if i & 4 != 0 { bbox.maximum.z() } else { bbox.minimum.z() };
                let corner = self.rotate_inv(&Vec3::new(x, y, z));
                min = min.min(corner);
                max = max.max(corner);
            }
            Some(AABB::from(min, max))
        } else {
            None
        }
    }
}

pub struct Scale {
    scalar: f32,
    hitable: Arc<dyn Hitable>,
}

impl Scale {
    pub fn new<H: Hitable + 'static>(scalar: f32, hitable: H) -> Scale {
        let hitable = Arc::new(hitable);
        Scale { scalar, hitable }
    }
}

impl Hitable for Scale {
    /// Reference: http://woo4.me/raytracer/translations/
    fn hit(&self, ray: &Ray, t0: f32, t1: f32) -> Option<HitRecord> {
        let origin = ray.origin / self.scalar;
        let direction = (ray.direction / self.scalar).normalize();

        let scaled_ray = Ray::new(origin, direction, ray.time);

        // The inner hitable works in scaled-local space. Distances there
        // are 1/scalar times world distances, so scale t bounds accordingly
        // for correct BVH pruning, and scale the returned t back to world
        // space so the outer BVH's depth comparison works correctly.
        let scaled_t0 = t0 / self.scalar;
        let scaled_t1 = t1 / self.scalar;

        if let Some(mut hit) = self.hitable.hit(&scaled_ray, scaled_t0, scaled_t1) {
            hit.point = hit.point * self.scalar;
            hit.shading_normal = (hit.shading_normal / self.scalar).normalize();
            hit.parameter = hit.parameter * self.scalar;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            bbox.minimum *= self.scalar;
            bbox.maximum *= self.scalar;
            Some(bbox)
        } else {
            None
        }
    }
}
