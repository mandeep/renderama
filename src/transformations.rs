use std::f32;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use nalgebra::Vector3;
use ray::Ray;

#[derive(Clone)]
pub struct Translate {
    offset: Vector3<f32>,
    hitable: Box<dyn Hitable>,
}

impl Translate {
    pub fn new<H: Hitable + 'static>(offset: Vector3<f32>, hitable: H) -> Translate {
        let hitable = Box::new(hitable);
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

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct Rotate {
    sin_theta: f32,
    cos_theta: f32,
    hitable: Box<dyn Hitable>,
    bbox: AABB,
}

impl Rotate {
    pub fn new<H: Hitable + 'static>(angle: f32, hitable: H) -> Rotate {
        let hitable = Box::new(hitable);

        let radians = (std::f32::consts::PI / 180.0) * angle;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        if let Some(bbox) = hitable.bounding_box(0.0, 1.0) {
            let mut minimum = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut maximum = Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX);

            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f32 * bbox.maximum.x + (1.0 - i as f32) * bbox.minimum.x;
                        let y = j as f32 * bbox.maximum.y + (1.0 - j as f32) * bbox.minimum.y;
                        let z = k as f32 * bbox.maximum.z + (1.0 - k as f32) * bbox.minimum.z;
                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vector3::new(newx, y, newz);

                        for c in 0..3 {
                            if tester[c] > maximum[c] {
                                maximum[c] = tester[c];
                            }
                            if tester[c] < minimum[c] {
                                minimum[c] = tester[c];
                            }
                        }
                    }
                }
            }
            Rotate { sin_theta,
                     cos_theta,
                     hitable,
                     bbox }
        } else {
            let minimum = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
            let maximum = Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX);
            let bbox = AABB::new(minimum, maximum);
            Rotate { sin_theta,
                     cos_theta,
                     hitable,
                     bbox }
        }
    }
}

impl Hitable for Rotate {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;

        origin[0] = self.cos_theta * ray.origin[0] - self.sin_theta * ray.origin[2];
        origin[2] = self.sin_theta * ray.origin[0] + self.cos_theta * ray.origin[2];

        direction[0] = self.cos_theta * ray.direction[0] - self.sin_theta * ray.direction[2];
        direction[2] = self.sin_theta * ray.direction[0] + self.cos_theta * ray.direction[2];

        let rotated_ray = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.hitable.hit(&rotated_ray, position_min, position_max) {
            hit.point[0] = self.cos_theta * hit.point[0] + self.sin_theta * hit.point[2];
            hit.point[2] = -self.sin_theta * hit.point[0] + self.cos_theta * hit.point[2];
            hit.normal[0] = self.cos_theta * hit.normal[0] + self.sin_theta * hit.normal[2];
            hit.normal[2] = -self.sin_theta * hit.normal[0] + self.cos_theta * hit.normal[2];
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        Some(self.bbox.clone())
    }

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(self.clone())
    }
}
