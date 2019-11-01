use std::f32;
use std::sync::Arc;

use nalgebra::core::Vector3;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use basis::OrthonormalBase;
use bvh::BVH;
use hitable::Hitable;
use pdf::PDF;
use plane::Plane;

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32,
    pub inverse_direction: Vector3<f32>,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>, time: f32) -> Ray {
        let inverse_direction = direction.map(|component| 1.0 / component);
        Ray { origin: origin,
              direction: direction,
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
pub fn pick_sphere_point(rng: &mut ThreadRng) -> Vector3<f32> {
    let normal_distribution = Normal::new(0.0, 1.0).unwrap();
    let x = normal_distribution.sample(rng) as f32;
    let y = normal_distribution.sample(rng) as f32;
    let z = normal_distribution.sample(rng) as f32;

    Vector3::new(x, y, z).normalize()
}

/// Find the offset ray given the ray origin and geometric normal of the shape
///
/// Reference:
/// Carsten Wächter, Nikolaus Binder
/// A Fast and Robust Method for Avoiding Self-Intersection
/// Ray Tracing Gems, Chapter 6
pub fn find_offset_point(point: Vector3<f32>, geometric_normal: Vector3<f32>) -> Vector3<f32> {
    let origin: f32 = 1.0 / 32.0;
    let float_scale: f32 = 1.0 / 65536.0;
    let int_scale: f32 = 256.0;

    let mut offset_int: Vector3<u32> = Vector3::zeros();

    for k in 0..3 {
        offset_int[k] = (int_scale * geometric_normal[k]) as u32;
    }

    let mut point_int: Vector3<f32> = Vector3::zeros();

    for j in 0..3 {
        if point[j] < 0.0 {
            point_int[j] = f32::from_bits(f32::to_bits(point[j]).wrapping_sub(offset_int[j]));
        } else {
            point_int[j] = f32::from_bits(f32::to_bits(point[j]).wrapping_add(offset_int[j]));
        }
    }

    let mut new_offset: Vector3<f32> = point_int.clone();

    for i in 0..3 {
        if point[i].abs() < origin {
            new_offset[i] = point_int[i] + float_scale * geometric_normal[i];
        }
    }

    new_offset
}

/// Compute the color of the surface that the ray has collided with
///
/// If the ray hits an object in the world, the object is colored in relation
/// to the object's material. If the ray does not record a hit, then we compute
/// the color of the atmosphere. We recursively call compute_color to sample
/// the color at the ray's hit point. The depth has been set to an arbitrary
/// limit of 50 which can lead to bias rendering.
///
pub fn compute_color(mut ray: Ray,
                     world: &BVH,
                     bounces: u32,
                     light_source: &Plane,
                     atmosphere: bool,
                     rng: &mut ThreadRng)
                     -> Vector3<f32> {
    let mut color = Vector3::zeros();
    let mut throughput = Vector3::new(1.0, 1.0, 1.0);

    for bounce in 0..=bounces {
        if let Some(hit_record) = world.hit(&ray, 1e-2, f32::MAX) {
            let emitted = hit_record.material.emitted(&ray, &hit_record);
            color += throughput.component_mul(&emitted);

            if let Some((attenuation, _, _)) = hit_record.material.scatter(&ray, &hit_record, rng) {
                let cosine_pdf =
                    PDF::CosinePDF { uvw: OrthonormalBase::new(&hit_record.shading_normal
                                                                          .normalize()) };
                let hitable_pdf = PDF::HitablePDF { origin: hit_record.point,
                                                    hitable: Arc::new(light_source.clone()) };
                let mixture_pdf = PDF::MixturePDF { cosine_pdf: &cosine_pdf,
                                                    hitable_pdf: &hitable_pdf };

                let mut offset_point = hit_record.point;
                if hit_record.geometric_normal != hit_record.shading_normal {
                    offset_point = find_offset_point(hit_record.point, hit_record.geometric_normal);
                    offset_point += pick_sphere_point(rng);
                }
                let scattered = Ray::new(offset_point, mixture_pdf.generate(rng), ray.time);
                let pdf = mixture_pdf.value(&scattered.direction.normalize());
                let scattering_pdf = hit_record.material
                                               .scattering_pdf(&ray, &hit_record, &scattered);

                throughput = throughput.component_mul(&(scattering_pdf * attenuation)) / pdf;

                ray = scattered;
            } else {
                break;
            }
        } else {
            if atmosphere {
                let unit_direction: Vector3<f32> = ray.direction.normalize();
                let point: f32 = 0.5 * (unit_direction.y + 1.0);
                let lerp =
                    (1.0 - point) * Vector3::repeat(1.0) + point * Vector3::new(0.5, 0.7, 1.0);
                color = throughput.component_mul(&lerp);
            } else {
                color = Vector3::zeros();
            }
        }

        if bounce > 3 {
            let roulette_factor = (1.0 - throughput.max()).max(0.05);
            if rng.gen::<f32>() < roulette_factor {
                break;
            }
            throughput /= 1.0 - roulette_factor;
        }
    }
    return color;
}
