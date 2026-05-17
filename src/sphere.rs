use std::f32::consts::PI;

use glam::Vec3A;
use rand_pcg::Pcg64Mcg;

use aabb::AABB;
use basis::OrthonormalBasis;
use results::HitResult;
use materials::MaterialId;
use ray::Ray;
use sampling::{uniform_sample_cone, uniform_sample_sphere};


/// Retrieve the spherical UV coordinates with the given normal
fn get_sphere_uv(normal: &Vec3A) -> (f32, f32) {
    let phi = normal.z.atan2(normal.x);
    let theta = normal.y.asin();
    let u = 1.0 - (phi + PI) / (2.0 * PI);
    let v = (theta + PI / 2.0) / PI;
    (u, v)
}

/// Sphere is a basic UV sphere.
#[derive(Clone)]
pub struct Sphere {
    pub center: Vec3A,
    pub radius: f32,
    pub material_id: MaterialId,
}

impl Sphere {
    /// Create a new sphere to place into the world.
    ///
    /// center is the point in world space where the sphere resides with diameter
    /// of 2 * radius using the material at the material_id index.
    pub fn new(center: Vec3A, radius: f32, material_id: MaterialId) -> Sphere {

        Sphere { center, radius, material_id }
    }

    /// Determine if the given ray intersects with a point on the sphere
    ///
    /// The equation is quadratic in terms of t. We solve for t looking for
    /// a real root. No real roots signifies a miss, one real root signifies
    /// a hit at the boundary of the sphere, and two real roots signify a
    /// ray hitting one point on the sphere and leaving through another point.
    ///
    /// Reference: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection.html
    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitResult> {
        let sphere_center: Vec3A = ray.origin - self.center;
        let a: f32 = ray.direction.dot(ray.direction);
        let b: f32 = sphere_center.dot(ray.direction);
        let c: f32 = sphere_center.dot(sphere_center) - (self.radius * self.radius);
        let discriminant: f32 = b * b - a * c;

        // checking the discriminant is a fast way to determine if the root is real
        if discriminant > 0.0 {
            let sqrt_d = discriminant.sqrt();
            let first_root: f32 = (-b - sqrt_d) / a;
            let second_root: f32 = (-b + sqrt_d) / a;

            let root = if first_root > position_min && first_root < position_max {
                first_root
            } else if second_root > position_min && second_root < position_max {
                second_root
            } else {
                return None;
            };

            let point = ray.point_at_parameter(root);
            let normal = (point - self.center) / self.radius;
            let (u, v) = get_sphere_uv(&normal);
            return Some(HitResult::new(root, u, v, point, normal, normal, self.material_id));
        }
        None
    }

    /// Create a bounding box around the sphere using it's radius
    pub fn bounding_box(&self) -> Option<AABB> {
        let radius = Vec3A::new(self.radius, self.radius, self.radius);
        let min = self.center - radius;
        let max = self.center + radius;

        Some(AABB::from(min, max))
    }

    /// Given a ray, calculate the probability density function (pdf)
    /// of having sampled in the ray's direction.
    pub fn evaluate_sampling_weight(&self, ray: &Ray) -> f32 {
        let center = self.center;
        let to_center = center - ray.origin;
        let distance_squared = to_center.length_squared();

        // if the ray originates from inside the sphere,
        // fallback to uniform sampling.
        if distance_squared <= self.radius * self.radius {
            return 1.0 / (4.0 * PI);
        }

        // compute the cone of directions that can possibly hit the spherical light
        let cos_theta_max = (1.0 - self.radius * self.radius / distance_squared).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        let oc = ray.origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        if b * b - a * c <= 0.0 {
            return 0.0;
        }

        // a smaller, farther light subtends a tighter cone, so it has a smaller
        // solid angle which leads to a higher weight
        1.0 / solid_angle
    }

    /// Sample a direction from the given point to the spherical light.
    ///
    /// origin is the offset point from the ray-primitive intersection test.
    pub fn sample_direction_to_light(&self, origin: Vec3A, rng: &mut Pcg64Mcg) -> Vec3A {
        let center = self.center;
        let to_center = center - origin;
        let distance_squared = to_center.length_squared();
        let distance = to_center.length();

        // fallback to uniform sampling if the origin is inside the sphere
        if distance_squared <= self.radius * self.radius {
            return uniform_sample_sphere(rng);
        }

        // uniformly sample from a cone as it's more efficient and more likely to hit
        // the spherical light
        let cos_theta_max = (1.0 - self.radius * self.radius / distance_squared).sqrt();
        let [cos_theta, sin_theta, phi] = uniform_sample_cone(cos_theta_max, rng).to_array();

        let basis = OrthonormalBasis::new(&to_center);

        distance * basis.local(&Vec3A::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta))
    }
}
