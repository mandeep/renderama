use std::f32;
use std::sync::Arc;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use bvh::BVH;
use hitable::Hitable;
use pdf::PDF;
use plane::Plane;
use ray::{find_offset_point, Ray};

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
                    let pdf = mixture_pdf.value(scattered.direction);
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
                let point: f32 = 0.5 * (ray.direction.y() + 1.0);
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
