use std::sync::Arc;

use nalgebra::core::Vector3;

use aabb;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;

use std::f32;

pub struct Sphere {
    pub start_center: Vector3<f32>,
    pub end_center: Vector3<f32>,
    pub radius: f32,
    pub material: Arc<dyn Material>,
    pub start_time: f32,
    pub end_time: f32,
}

impl Sphere {
    /// Create a new sphere to place into the world
    ///
    /// We use the 'static lifetime so that we can create a Arc material
    /// within the function rather than having to pass a Arc material
    /// as an input parameter.
    pub fn new<M: Material + 'static>(start_center: Vector3<f32>,
                                      end_center: Vector3<f32>,
                                      radius: f32,
                                      material: M,
                                      start_time: f32,
                                      end_time: f32)
                                      -> Sphere {
        let material = Arc::new(material);
        Sphere { start_center,
                 end_center,
                 radius,
                 material,
                 start_time,
                 end_time }
    }

    pub fn center(&self, time: f32) -> Vector3<f32> {
        self.start_center
        + ((time - self.start_time) / (self.end_time - self.start_time))
          * (self.end_center - self.start_center)
    }
}

fn get_sphere_uv(p: &Vector3<f32>) -> (f32, f32) {
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();
    let u = 1.0 - (phi + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    let v = (theta + std::f32::consts::PI / 2.0) / std::f32::consts::PI;
    (u, v)
}

impl Hitable for Sphere {
    /// Determine if the given ray intersects with a point on the sphere
    ///
    /// The equation is quadratic in terms of t. We solve for t looking for
    /// a real root. No real roots signifies a miss, one real root signifies
    /// a hit at the boundary of the sphere, and two real roots signify a
    /// ray hitting one point on the sphere and leaving through another point.
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let sphere_center: Vector3<f32> = ray.origin - self.center(ray.time);
        let a: f32 = ray.direction.dot(&ray.direction);
        let b: f32 = sphere_center.dot(&ray.direction);
        let c: f32 = sphere_center.dot(&sphere_center) - (self.radius * self.radius);
        let discriminant: f32 = b * b - a * c;

        // checking the discriminant is a fast way to determine if the root is real
        if discriminant >= 0.0 {
            let first_root: f32 = (-b - (b * b - a * c).sqrt()) / a;
            let second_root: f32 = (-b + (b * b - a * c).sqrt()) / a;
            let mut roots = vec![first_root, second_root];

            // if we have two positive roots, we want the smaller one as
            // it is the first hit point of the sphere
            roots.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // negative roots mean the intersection point is not in our view
            // so we can safely disregard these hits
            roots.retain(|&root| root > 0.0);

            for root in roots {
                if root > position_min && root < position_max {
                    let point = ray.point_at_parameter(root);
                    let normal = (point - self.center(ray.time)) / self.radius;
                    let (u, v) = get_sphere_uv(&normal);
                    return Some(HitRecord::new(root, u, v, point, normal, self.material.clone()));
                }
            }
        }
        None
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<aabb::AABB> {
        let radius = Vector3::new(self.radius, self.radius, self.radius);
        let min0 = self.center(t0) - radius;
        let max0 = self.center(t0) + radius;
        let min1 = self.center(t1) - radius;
        let max1 = self.center(t1) + radius;

        let small = aabb::AABB::new(min0, max0);
        let big = aabb::AABB::new(min1, max1);

        Some(small.surrounding_box(&big))
    }
}
