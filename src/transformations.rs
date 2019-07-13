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

    fn bounding_box(&self, t0: f32, t1:f32) -> Option<AABB> {
        if let Some(mut bbox) = self.hitable.bounding_box(t0, t1) {
            bbox.minimum += self.offset;
            bbox.maximum += self.offset;
            return Some(bbox);
        }

        None
    }

    fn box_clone(&self) -> Box<dyn Hitable> {
        Box::new(*self).clone()
    }
}

pub struct Rotate {
    sin_theta: f32,
    cos_theta: f32,
    hitable: Box<dyn Hitable>,
}

impl Rotate {
    pub fn new<H: Hitable + 'static>(angle: f32, hitable: H) -> Rotate {
        let hitable = Box::new(hitable);

        let radians = (std::f32::consts::PI / 180.0) * angle;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        Rotate { sin_theta,
                 cos_theta,
                 hitable }
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
}
