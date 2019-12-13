use std::f32;
use std::sync::Arc;

use glam::Vec3;
use nalgebra::Vector3;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use bvh::BVH;
use hitable::Hitable;
use pdf::PDF;
use plane::Plane;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: f32,
    pub inverse_direction: Vec3,
}

impl Ray {
    /// Create a new Ray with origin at `a` and direction towards `b`
    pub fn new(origin: Vec3, direction: Vec3, time: f32) -> Ray {
        let direction = direction.normalize();
        let inverse_direction = direction.reciprocal();
        Ray { origin: origin,
              direction: direction,
              time: time,
              inverse_direction: inverse_direction }
    }

    /// Find the point on the ray given the parameter of the direction vector
    pub fn point_at_parameter(&self, parameter: f32) -> Vec3 {
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
pub fn pick_sphere_point(rng: &mut ThreadRng) -> Vec3 {
    let normal_distribution = Normal::new(0.0, 1.0).unwrap();
    let x = normal_distribution.sample(rng) as f32;
    let y = normal_distribution.sample(rng) as f32;
    let z = normal_distribution.sample(rng) as f32;

    Vec3::new(x, y, z).normalize()
}

/// Find the offset ray given the ray origin and geometric normal of the shape
///
/// Reference:
/// Carsten Wächter, Nikolaus Binder
/// A Fast and Robust Method for Avoiding Self-Intersection
/// Ray Tracing Gems, Chapter 6
pub fn find_offset_point(point: Vec3, geometric_normal: Vec3) -> Vec3 {
    let origin: f32 = 1.0 / 32.0;
    let float_scale: f32 = 1.0 / 65536.0;
    let int_scale: f32 = 256.0;

    let offset_int: Vector3<u32> = Vector3::new((int_scale * geometric_normal.x()) as u32,
                                                (int_scale * geometric_normal.y()) as u32,
                                                (int_scale * geometric_normal.z()) as u32);

    let mut point_int = Vec3::zero();

    if point.x() < 0.0 {
        point_int.set_x(f32::from_bits(f32::to_bits(point.x()).wrapping_sub(offset_int.x)));
    } else {
        point_int.set_x(f32::from_bits(f32::to_bits(point.x()).wrapping_add(offset_int.x)));
    }
    if point.y() < 0.0 {
        point_int.set_y(f32::from_bits(f32::to_bits(point.y()).wrapping_sub(offset_int.y)));
    } else {
        point_int.set_y(f32::from_bits(f32::to_bits(point.y()).wrapping_add(offset_int.y)));
    }

    if point.z() < 0.0 {
        point_int.set_z(f32::from_bits(f32::to_bits(point.z()).wrapping_sub(offset_int.z)));
    } else {
        point_int.set_z(f32::from_bits(f32::to_bits(point.z()).wrapping_add(offset_int.z)));
    }

    let mut new_offset: Vec3 = point_int.clone();

    if point.x().abs() < origin {
        new_offset.set_x(point_int.x() + float_scale * geometric_normal.x());
    }
    if point.y().abs() < origin {
        new_offset.set_y(point_int.y() + float_scale * geometric_normal.y());
    }
    if point.z().abs() < origin {
        new_offset.set_z(point_int.z() + float_scale * geometric_normal.z());
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
                     -> Vec3 {
    let mut color = Vec3::zero();
    let mut throughput = Vec3::one();

    for bounce in 0..=bounces {
        if let Some(hit_record) = world.hit(&ray, 1e-2, f32::MAX) {
            let emitted = hit_record.material.emitted(&ray, &hit_record);
            color += throughput * emitted;

            if let Some(scatter_record) = hit_record.material.scatter(&ray, &hit_record, rng) {
                if scatter_record.specular {
                    throughput *= scatter_record.attenuation;
                    ray = scatter_record.specular_ray;
                } else {
                    let hitable_pdf = PDF::HitablePDF { origin: hit_record.point,
                                                        hitable: Arc::new(light_source.clone()) };
                    let mixture_pdf = PDF::MixturePDF { cosine_pdf: &scatter_record.pdf,
                                                        hitable_pdf: &hitable_pdf };

                    let mut offset_point = hit_record.point;
                    if hit_record.geometric_normal != hit_record.shading_normal {
                        offset_point =
                            find_offset_point(hit_record.point, hit_record.geometric_normal);
                        offset_point += pick_sphere_point(rng);
                    }
                    let scattered = Ray::new(offset_point, mixture_pdf.generate(rng), ray.time);
                    let pdf = mixture_pdf.value(scattered.direction.normalize());
                    let scattering_pdf = hit_record.material
                                                   .scattering_pdf(&ray, &hit_record, &scattered);

                    throughput *= (scattering_pdf * scatter_record.attenuation) / pdf;

                    ray = scattered;
                }
            } else {
                break;
            }
        } else {
            if atmosphere {
                let unit_direction: Vec3 = ray.direction.normalize();
                let point: f32 = 0.5 * (unit_direction.y() + 1.0);
                let lerp = (1.0 - point) * Vec3::splat(1.0) + point * Vec3::new(0.5, 0.7, 1.0);
                color = throughput * lerp;
            } else {
                color = Vec3::zero();
            }
        }

        if bounce > 3 {
            let roulette_factor = (1.0 - throughput.max_element()).max(0.05);
            if rng.gen::<f32>() < roulette_factor {
                break;
            }
            throughput /= 1.0 - roulette_factor;
        }
    }
    return color;
}
