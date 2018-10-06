use hitable::{Hitable, HitRecord};
use materials::Material;
use nalgebra::core::Vector3;
use ray::Ray;


pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material: Box<dyn Material>
}


impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64, material: Box<dyn Material>) -> Sphere {
        Sphere { center: center, radius: radius, material: material }
    }
}


impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, position_min: f64, position_max: f64) -> Option<HitRecord> {
        let sphere_center: Vector3<f64> = ray.origin - self.center;
        let a: f64 = ray.direction.dot(&ray.direction);
        let b: f64 = sphere_center.dot(&ray.direction);
        let c: f64 = sphere_center.dot(&sphere_center) - (self.radius * self.radius);
        let discriminant: f64 = b * b * a * c;

        if discriminant > 0.0 {
            let first_root: f64 = (-b - (b * b - a * c).sqrt()) / a;
            let second_root: f64 = (-b + (b * b - a * c).sqrt()) / a;
            let roots = vec![first_root, second_root];

            for root in roots {
                if root < position_max && root > position_min {
                    let point = ray.point_at_parameter(root);
                    let normal = (point - self.center) / self.radius;
                    return Some(HitRecord::new(root, point, normal, self.material.box_clone()));
                }
            }

        }
        None
    }
}
