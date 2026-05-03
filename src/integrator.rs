use std::f32;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::StandardNormal;

use geometry::Geometry;
use pdf::{HybridPDF, MaterialPDF};
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
pub fn render_path_integrator(mut ray: Ray, scene: &Scene, bounces: u32, rng: &mut ThreadRng) -> Vec3 {
    let mut color = Vec3::ZERO;
    let mut throughput = Vec3::ONE;

    for bounce in 0..=bounces {
        if let Some(hit_event) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
            let material = &scene.materials[hit_event.material_id.index()];
            let emitted = material.emitted(&ray, &hit_event);
            color += throughput * emitted;

            if let Some(scatter_event) = material.scatter(&ray, &hit_event, rng) {
                if scatter_event.specular {
                    throughput *= scatter_event.attenuation;
                    ray = scatter_event.specular_ray;
                } else {
                    let importance_pdf = scene.light_source.as_ref().map(|light| {
                        MaterialPDF::Importance { origin: hit_event.point, geometry: Geometry::Plane(light.clone())}
                    });
                    let importance_ref = importance_pdf.as_ref().unwrap_or(&scatter_event.pdf);
                    let hybrid_pdf = HybridPDF::new(&scatter_event.pdf, importance_ref);

                    let scattered_direction = hybrid_pdf.generate(rng);
                    let offset_normal = if scattered_direction.dot(hit_event.geometric_normal) > 0.0 {
                        hit_event.geometric_normal
                    } else {
                        -hit_event.geometric_normal
                    };
                    let offset_point = find_offset_point(hit_event.point, offset_normal);
                    let scattered = Ray::new(offset_point, scattered_direction, ray.time);
                    let pdf_value = hybrid_pdf.value(scattered.direction);
                    let scattering_pdf = material.scattering_pdf(&ray, &hit_event, &scattered);

                    throughput *= (scattering_pdf * scatter_event.attenuation) / pdf_value;

                    ray = scattered;
                }
            } else {
                break;
            }
        } else {
            if let Some(environment) = &scene.environment {
                // u and v not needed for the enviroment map so we just pass dummy arguments
                color += throughput * environment.value(0.0, 0.0, &ray.direction);
                break;
            } else if scene.atmosphere {
                let point: f32 = 0.5 * (ray.direction.y + 1.0);
                let lerp = (1.0 - point) * Vec3::splat(1.0) + point * Vec3::new(0.5, 0.7, 1.0);
                color += throughput * lerp;
                break;
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
    color
}

pub fn render_normals(ray: Ray, scene: &Scene) -> Vec3 {
    if let Some(hit) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
        let normal = hit.shading_normal;
        0.5 * Vec3::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0)
    } else {
        let point = 0.5 * (ray.direction.normalize().y + 1.0);
        (1.0 - point) * Vec3::new(1.0, 1.0, 1.0) + point * Vec3::new(0.5, 0.7, 1.0)
    }
}

fn visibility(scene: &Scene, ray: &Ray, target_dist: f32) -> f32 {
    // We use a slightly larger epsilon (1e-3) for the start 
    // and a shorter max distance to avoid hitting the light itself
    if let Some(_) = scene.accelerator.hit(ray, 0.001, target_dist - 0.01) {
        return 0.0; // Blocked by wall, floor, or the surface itself
    }
    1.0 // Clear path
}

pub fn render_nee_integrator(mut ray: Ray, scene: &Scene, bounces: u32, rng: &mut ThreadRng) -> Vec3 {
    let mut color = Vec3::ZERO;
    let mut throughput = Vec3::ONE;
    let mut last_specular = true;

    for bounce in 0..=bounces {
        if let Some(hit_event) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
            let material = &scene.materials[hit_event.material_id.index()];
            
            // 1. Direct Emission
            if last_specular {
                color += throughput * material.emitted(&ray, &hit_event);
            }

            if let Some(scatter_event) = material.scatter(&ray, &hit_event, rng) {
                if scatter_event.specular {
                    throughput *= scatter_event.attenuation;
                    ray = scatter_event.specular_ray;
                    last_specular = true;
                } else {
                    last_specular = false;

                    // 2. Next Event Estimation (Direct Lighting)
                    if let Some(light_geom) = &scene.light_source {
                        // Sample direction toward light using Geometry::pdf_random
                        let light_dir_vec = light_geom.pdf_random(hit_event.point, rng);
                        let light_dist = light_dir_vec.length();
                        let light_dir = light_dir_vec.normalize();

                        // OFFSET: Move origin along the normal to prevent self-intersection[cite: 2]
                        let shadow_origin = hit_event.point + hit_event.geometric_normal * 0.001;
                        let shadow_ray = Ray::new(shadow_origin, light_dir, ray.time);
                        
                        let v = visibility(scene, &shadow_ray, light_dist);
                        
                        if v > 0.0 {
                            let p_val = light_geom.pdf_value(hit_event.point, light_dir);
                            
                            // Only Planes currently return a non-zero pdf_value[cite: 1]
                            if p_val > 1e-7 {
                                let s_pdf = material.scattering_pdf(&ray, &hit_event, &shadow_ray);
                                
                                // Get actual emission by hitting the light source
                                if let Some(sh_hit) = scene.accelerator.hit(&shadow_ray, 0.001, light_dist + 0.01) {
                                    let sh_mat = &scene.materials[sh_hit.material_id.index()];
                                    let emission = sh_mat.emitted(&shadow_ray, &sh_hit);
                                    
                                    color += (throughput * emission * scatter_event.attenuation * s_pdf) / p_val;
                                }
                            }
                        }
                    }

                    // 3. Indirect Lighting
                    let scattered_dir = scatter_event.pdf.generate(rng);
                    let pdf_val = scatter_event.pdf.value(scattered_dir);
                    if pdf_val <= 0.0 { break; }

                    // Ensure next ray starts outside the surface
                    let next_origin = if scattered_dir.dot(hit_event.geometric_normal) > 0.0 {
                        hit_event.point + hit_event.geometric_normal * 0.001
                    } else {
                        hit_event.point - hit_event.geometric_normal * 0.001
                    };

                    ray = Ray::new(next_origin, scattered_dir, ray.time);
                    let s_pdf = material.scattering_pdf(&ray, &hit_event, &ray);
                    
                    throughput *= (s_pdf * scatter_event.attenuation) / pdf_val;
                }
            } else { break; }
        } else {
            if let Some(env) = &scene.environment {
                color += throughput * env.value(0.0, 0.0, &ray.direction);
            }
            break;
        }

        // Russian Roulette
        if bounce > 3 {
            let q = (1.0 - throughput.max_element()).max(0.05);
            if rng.gen::<f32>() < q { break; }
            throughput /= 1.0 - q;
        }
    }
    color
}
