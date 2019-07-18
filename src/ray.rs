use std::f32;

use nalgebra::core::Vector3;
use rand::distributions::{Distribution, Normal};

use bvh::BVH;
use hitable::Hitable;

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32,
    pub inverse_direction: Vector3<f32>,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, time: f32) -> Ray {
        let inverse_direction = b.map(|component| 1.0 / component);
        Ray { origin: a,
              direction: b,
              time: time,
              inverse_direction: inverse_direction }
    }

    /// Find the point on the ray given the parameter of the direction vector
    pub fn point_at_parameter(&self, parameter: f32) -> Vector3<f32> {
        self.origin + parameter * self.direction
    }
}

/// Pick a random point on the unit sphere
///
/// We can use a Gaussian distribution to uniformly generate points
/// on the unit sphere. If a uniform distribution were used instead,
/// the points would tend to aggregate to the poles of the sphere.
/// A vector is created from the sample points taken for each coordinate
/// axis and the unit vector of this newly created vector is returned.
///
/// Reference: http://mathworld.wolfram.com/SpherePointPicking.html
///
pub fn pick_sphere_point(rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
    let normal_distribution = Normal::new(0.0, 1.0);
    let x = normal_distribution.sample(rng) as f32;
    let y = normal_distribution.sample(rng) as f32;
    let z = normal_distribution.sample(rng) as f32;

    Vector3::new(x, y, z).normalize()
}

/// Compute the color of the surface that the ray has collided with
///
/// If the ray hits an object in the world, the object is colored in relation
/// to the object's material. If the ray does not record a hit, then we compute
/// the color of the atmosphere. We recursively call compute_color to sample
/// the color at the ray's hit point. The depth has been set to an arbitrary
/// limit of 50 which can lead to bias rendering.
///
/// Instead of returning a Vector3 of zeros, the following code can be used
/// to simulate an atmosphere:
/// ```
///     let unit_direction: Vector3<f32> = ray.direction.normalize();
///     let point: f32 = 0.5 * (unit_direction.y + 1.0);
///     return (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0);
/// ```
///
pub fn compute_color(ray: &Ray,
                     world: &BVH,
                     depth: i32,
                     rng: &mut rand::rngs::ThreadRng)
                     -> Vector3<f32> {
    if let Some(hit_record) = world.hit(ray, 0.001, f32::MAX) {
        let emitted = hit_record.material
                                .emitted(hit_record.u, hit_record.v, &hit_record.point);
        if depth < 50 {
            if let Some((attenuation, scattered)) =
                hit_record.material.scatter(ray, &hit_record, rng)
            {
                return emitted
                       + attenuation.component_mul(&compute_color(&scattered,
                                                                  world,
                                                                  depth + 1,
                                                                  rng));
            }
        }
        emitted
    } else {
        Vector3::zeros()
    }
}
