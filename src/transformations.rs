use std::f32;
use std::f32::consts::PI;
use std::sync::Arc;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use nalgebra::Vector3;
use ray::Ray;

pub struct Translate {
    offset: Vector3<f32>,
    hitable: Arc<dyn Hitable>,
}

impl Translate {
    pub fn new<H: Hitable + 'static>(offset: Vector3<f32>, hitable: H) -> Translate {
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
    sin_theta: f32,
    cos_theta: f32,
    hitable: Arc<dyn Hitable>,
}

impl Rotate {
    pub fn new<H: Hitable + 'static>(angle: f32, hitable: H) -> Rotate {
        let hitable = Arc::new(hitable);
        let radians = (PI / 180.0) * angle;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        Rotate { sin_theta,
                 cos_theta,
                 hitable }
    }

    pub fn rotate(&self, vector: &Vector3<f32>) -> Vector3<f32> {
        Vector3::new(self.cos_theta * vector.x - self.sin_theta * vector.z,
                     vector.y,
                     self.sin_theta * vector.x + self.cos_theta * vector.z)
    }

    pub fn rotate_inv(&self, vector: &Vector3<f32>) -> Vector3<f32> {
        Vector3::new(self.cos_theta * vector.x + self.sin_theta * vector.z,
                     vector.y,
                     -self.sin_theta * vector.x + self.cos_theta * vector.z)
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
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
            (0..2).for_each(|i| {
                      (0..2).for_each(|j| {
                                (0..2).for_each(|k| {
                                          let x = i as f32 * bbox.maximum.x
                                                  + (1 - i) as f32 * bbox.minimum.x;
                                          let y = j as f32 * bbox.maximum.y
                                                  + (1 - j) as f32 * bbox.minimum.y;
                                          let z = k as f32 * bbox.maximum.z
                                                  + (1 - k) as f32 * bbox.minimum.z;
                                          let newx = self.cos_theta * x + self.sin_theta * z;
                                          let newz = -self.sin_theta * x + self.cos_theta * z;
                                          let rotation = Vector3::new(newx, y, newz);
                                          (0..3).for_each(|c| {
                                                    max[c] = max[c].max(rotation[c]);
                                                    min[c] = min[c].min(rotation[c]);
                                                });
                                      });
                            });
                  });

            bbox.minimum = min;
            bbox.maximum = max;
            Some(bbox)
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
        let origin = &ray.origin / self.scalar;
        let direction = &ray.direction / self.scalar;

        let scaled_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.hitable.hit(&scaled_ray, t0, t1) {
            hit.point = &hit.point * self.scalar;
            // hit.normal = &hit.normal / self.scalar;
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
