use std::f64;

use nalgebra::core::Vector3;
use rand;
use rand::distributions::{Distribution, Normal};

use hitable::Hitable;
use world::World;


pub struct Ray {
    pub origin: Vector3<f64>,
    pub direction: Vector3<f64>
}


impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(a: Vector3<f64>, b: Vector3<f64>) -> Ray {
        Ray { origin: a, direction: b }
    }

    pub fn point_at_parameter(&self, parameter: f64) -> Vector3<f64> {
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
pub fn pick_sphere_point(rng: &mut rand::ThreadRng) -> Vector3<f64> {
    let normal_distribution = Normal::new(0.0, 1.0);
    let x = normal_distribution.sample(rng);
    let y = normal_distribution.sample(rng);
    let z = normal_distribution.sample(rng);

    Vector3::new(x, y, z).normalize()
}


pub fn compute_color(ray: &Ray, world: &World, depth: i32, rng: &mut rand::ThreadRng) -> Vector3<f64> {
    match world.hit(ray, 0.001, f64::MAX) {
        Some(hit_record) => {
            if depth < 50 {
                match hit_record.material.scatter(ray, &hit_record, rng) {
                    Some((attenuation, scattered)) => {
                        return attenuation.component_mul(
                            &compute_color(&scattered, world, depth + 1, rng));
                    }
                    None => { return Vector3::zeros(); }
                }
            } else {
                return Vector3::zeros();
            }
        }
        None => {
            let unit_direction: Vector3<f64> = ray.direction.normalize();
            let point: f64 = 0.5 * (unit_direction.y + 1.0);

            return (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0);
        }
    }
}
