use std::f32::consts::PI;

use glam::Vec3;
use rand::rngs::ThreadRng;

use aabb::AABB;
use events::HitEvent;
use materials::MaterialId;
use ray::Ray;
use sampling::{uniform_sample_cone, uniform_sample_sphere};


fn get_sphere_uv(p: &Vec3) -> (f32, f32) {
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();
    let u = 1.0 - (phi + PI) / (2.0 * PI);
    let v = (theta + PI / 2.0) / PI;
    (u, v)
}

#[derive(Clone)]
pub struct Sphere {
    pub start_center: Vec3,
    pub end_center: Vec3,
    pub radius: f32,
    pub material_id: MaterialId,
    pub start_time: f32,
    pub end_time: f32,
}

impl Sphere {
    /// Create a new sphere to place into the world
    ///
    /// We use the 'static lifetime so that we can create a Arc material
    /// within the function rather than having to pass a Arc material
    /// as an input parameter.
    pub fn new(start_center: Vec3,
                                      end_center: Vec3,
                                      radius: f32,
                                      material_id: MaterialId,
                                      start_time: f32,
                                      end_time: f32)
                                      -> Sphere {

        Sphere { start_center,
                 end_center,
                 radius,
                 material_id,
                 start_time,
                 end_time }
    }

    pub fn center(&self, time: f32) -> Vec3 {
        self.start_center
        + ((time - self.start_time) / (self.end_time - self.start_time))
          * (self.end_center - self.start_center)
    }

    /// Determine if the given ray intersects with a point on the sphere
    ///
    /// The equation is quadratic in terms of t. We solve for t looking for
    /// a real root. No real roots signifies a miss, one real root signifies
    /// a hit at the boundary of the sphere, and two real roots signify a
    /// ray hitting one point on the sphere and leaving through another point.
    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitEvent> {
        let sphere_center: Vec3 = ray.origin - self.center(ray.time);
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
            let normal = (point - self.center(ray.time)) / self.radius;
            let (u, v) = get_sphere_uv(&normal);
            return Some(HitEvent::new(root, u, v, point, normal, normal, self.material_id));
        }
        None
    }

    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        let radius = Vec3::new(self.radius, self.radius, self.radius);
        let min0 = self.center(t0) - radius;
        let max0 = self.center(t0) + radius;
        let min1 = self.center(t1) - radius;
        let max1 = self.center(t1) + radius;

        let small = AABB::from(min0, max0);
        let big = AABB::from(min1, max1);

        Some(small.surrounding_box(&big))
    }

    pub fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f32 {
        let center = self.center(0.0);
        let to_center = center - origin;
        let distance_squared = to_center.length_squared();

        // Origin inside the sphere — fall back to uniform sphere sampling
        // (cone subtends the full 4π steradians).
        if distance_squared <= self.radius * self.radius {
            return 1.0 / (4.0 * PI);
        }

        // Half-angle of the cone subtending the sphere from `origin`.
        //   sin²θ_max = r² / d²    →    cos θ_max = sqrt(1 - r²/d²)
        let cos_theta_max = (1.0 - self.radius * self.radius / distance_squared).sqrt();

        // Solid angle of the cone: Ω = 2π(1 − cos θ_max).
        // Uniform sampling over the cone gives p(ω) = 1 / Ω.
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        // Use the same discriminant test as sphere::hit so that pdf_value returns
        // a positive PDF for exactly the same directions that hit() accepts.  The
        // earlier dot-product cone check and hit()'s discriminant can disagree near
        // the tangent due to floating-point rounding, which made power_heuristic
        // assign weight=1 to near-tangent samples and created a bright ring artifact.
        let oc = origin - center;
        let a = direction.dot(direction);
        let b = oc.dot(direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        if b * b - a * c <= 0.0 {
            return 0.0;
        }

        1.0 / solid_angle
    }

    pub fn pdf_random(&self, origin: Vec3, rng: &mut ThreadRng) -> Vec3 {
        let center = self.center(0.0);
        let to_center = center - origin;
        let distance_squared = to_center.length_squared();

        // Inside the sphere: just return a random direction on the unit sphere.
        if distance_squared <= self.radius * self.radius {
            return uniform_sample_sphere(rng);
        }

        let cos_theta_max = (1.0 - self.radius * self.radius / distance_squared).sqrt();

        // Sample (cos θ, φ) uniformly in the cone.
        let [cos_theta, sin_theta, phi] = uniform_sample_cone(cos_theta_max, rng).to_array();

        // Local frame around the axis from origin → center.
        let w = to_center.normalize();
        let a = if w.x.abs() > 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
        let v = w.cross(a).normalize();
        let u = w.cross(v);

        let dist = distance_squared.sqrt();
        dist * (phi.cos() * sin_theta * u + phi.sin() * sin_theta * v + cos_theta * w)
    }
}
