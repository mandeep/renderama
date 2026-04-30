use std::f32;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::StandardNormal;

use basis::OrthonormalBasis;
use bvh::BVH;
use geometry::Geometry;
use pdf::PDF;
use ray::{find_offset_point, Ray};
use scene::Scene;

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
    let x: f32 = rng.sample(StandardNormal);
    let y: f32 = rng.sample(StandardNormal);
    let z: f32 = rng.sample(StandardNormal);

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
pub fn render_path_integrator(mut ray: Ray,
                     scene: &Scene,
                     bounces: u32,
                     light_source: &Option<Plane>,
                     atmosphere: bool,
                     rng: &mut ThreadRng)
                     -> Vec3 {
    let mut color = Vec3::ZERO;
    let mut throughput = Vec3::ONE;

    for bounce in 0..=bounces {
        if let Some(hit_record) = scene.world.hit(&ray, 1e-4, f32::MAX) {
            let material = &scene.materials[hit_record.material_id as usize];
            let emitted = material.emitted(&ray, &hit_record);
            color += throughput * emitted;

            if let Some(scatter_record) = material.scatter(&ray, &hit_record, rng) {
                if scatter_record.specular {
                    throughput *= scatter_record.attenuation;
                    ray = scatter_record.specular_ray;
                } else {
                    let fallback_pdf = PDF::FallbackPDF { uvw: OrthonormalBasis::new(&hit_record.shading_normal) };
                    let importance_pdf = light_source.as_ref().map(|light| {
                        PDF::ImportancePDF { origin: hit_record.point, geometry: Geometry::Plane(light.clone())}
                    });
                    let hybrid_pdf = match &importance_pdf {
                        Some(sample_target_pdf) => PDF::HybridPDF {
                            material_pdf: &scatter_record.pdf,
                            importance_pdf: sample_target_pdf,
                        },
                        None => PDF::HybridPDF {
                            material_pdf: &scatter_record.pdf,
                            importance_pdf: &fallback_pdf,
                        },
                    };

                    let scattered_direction = hybrid_pdf.generate(rng);
                    let offset_normal = if scattered_direction.dot(hit_record.geometric_normal) > 0.0 {
                        hit_record.geometric_normal
                    } else {
                        -hit_record.geometric_normal
                    };
                    let offset_point = find_offset_point(hit_record.point, offset_normal);
                    let scattered = Ray::new(offset_point, scattered_direction, ray.time);
                    let pdf_value = hybrid_pdf.value(scattered.direction);
                    let scattering_pdf = material.scattering_pdf(&ray, &hit_record, &scattered);

                    throughput *= (scattering_pdf * scatter_record.attenuation) / pdf_value;

                    ray = scattered;
                }
            } else {
                break;
            }
        } else {
            if atmosphere {
                let point: f32 = 0.5 * (ray.direction.y + 1.0);
                let lerp = (1.0 - point) * Vec3::splat(1.0) + point * Vec3::new(0.5, 0.7, 1.0);
                color += throughput * lerp;
            }
            break;
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

pub fn render_normals(ray: Ray, world: &BVH) -> Vec3 {
    if let Some(hit) = world.hit(&ray, 1e-4, f32::MAX) {
        let normal = hit.shading_normal;
        0.5 * Vec3::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0)
    } else {
        let point = 0.5 * (ray.direction.normalize().y + 1.0);
        (1.0 - point) * Vec3::new(1.0, 1.0, 1.0) + point * Vec3::new(0.5, 0.7, 1.0)
    }
}
