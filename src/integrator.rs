use std::f32;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::RngExt;
use rand_distr::StandardNormal;

use pdf::{HybridPDF, MaterialPDF, power_heuristic};
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
                    let number_of_lights = scene.lights.len();
                    let importance_pdf = if number_of_lights > 0 {
                        // sampling towards a random light source is good enough for this integrator
                        let i = (rng.random::<f32>() * number_of_lights as f32) as usize % number_of_lights;
                        Some(MaterialPDF::Importance { origin: hit_event.point, geometry: scene.lights[i].to_geometry() })
                    } else {
                        None
                    };
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
            if rng.random::<f32>() < roulette_factor {
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
        let point = 0.5 * (ray.direction.y + 1.0);
        (1.0 - point) * Vec3::new(1.0, 1.0, 1.0) + point * Vec3::new(0.5, 0.7, 1.0)
    }
}

pub fn render_nee_integrator(mut ray: Ray, scene: &Scene, bounces: u32, rng: &mut ThreadRng) -> (Vec3, Vec3, Vec3) {
    let mut color = Vec3::ZERO;
    let mut throughput = Vec3::ONE;
    let mut last_specular = true;
    let mut last_material_pdf = 0.0;
    let mut first_albedo = Vec3::ZERO;
    let mut first_normal = Vec3::ZERO;

    for bounce in 0..=bounces {
        if let Some(hit_event) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
            let material = &scene.materials[hit_event.material_id.index()];

            let emission = material.emitted(&ray, &hit_event);
            if emission.length_squared() > 0.0 {
                if last_specular {
                    color += throughput * emission;
                } else {
                    let light_pdf: f32 = scene.lights.iter()
                        .map(|light| light.pdf_value(ray.origin, ray.direction))
                        .sum();
                    let weight = power_heuristic(last_material_pdf, light_pdf);
                    color += throughput * weight * emission;
                }
            }

            if let Some(scatter_event) = material.scatter(&ray, &hit_event, rng) {
                if bounce == 0 {
                    first_albedo = scatter_event.attenuation;
                    first_normal = hit_event.shading_normal;
                }
                if scatter_event.specular {
                    throughput *= scatter_event.attenuation;
                    ray = scatter_event.specular_ray;
                    last_specular = true;
                } else {
                    last_specular = false;

                    let mut direct_light = Vec3::ZERO;

                    // using a manual offset instead of the find_offset_point for now as it
                    // gives better results on shadow rays
                    let shadow_origin = hit_event.point + hit_event.geometric_normal * 1e-3;

                    for light_source in &scene.lights {
                        let light_direction_vector = light_source.pdf_random(shadow_origin, rng);
                        let light_distance = light_direction_vector.length();
                        let light_direction = light_direction_vector.normalize();

                        let shadow_ray = Ray::new(shadow_origin, light_direction, ray.time);
                        let t_max = light_source.occlusion_t_max(light_distance);

                        if !scene.accelerator.any_hit(&shadow_ray, 1e-3, t_max) {
                            let light_pdf = light_source.pdf_value(shadow_origin, light_direction);
                            if light_pdf > 1e-7 {
                                let scattering_pdf = material.scattering_pdf(&ray, &hit_event, &shadow_ray);
                                let material_pdf = scatter_event.pdf.value(light_direction);
                                let weight = power_heuristic(light_pdf, material_pdf);
                                direct_light += (weight * throughput * light_source.emission * scatter_event.attenuation * scattering_pdf) / light_pdf;
                            }
                        }
                    }

                    // environment map NEE samples toward bright regions via luminance CDF
                    // this prevents the high number of fireflies we saw in high contrast
                    // environment maps with luminance values as high as 100,000
                    if let Some(env) = &scene.environment {
                        if let Some(env_dir) = env.env_pdf_random(rng) {
                            let env_pdf = env.env_pdf_value(&env_dir).unwrap_or(0.0);
                            if env_pdf > 1e-7 {
                                let shadow_origin = hit_event.point + hit_event.geometric_normal * 1e-3;
                                let env_shadow_ray = Ray::new(shadow_origin, env_dir, ray.time);
                                if scene.accelerator.hit(&env_shadow_ray, 1e-3, f32::MAX).is_none() {
                                    let env_value = env.value(0.0, 0.0, &env_dir);
                                    let material_pdf = scatter_event.pdf.value(env_dir);
                                    let scattering_pdf = material.scattering_pdf(&ray, &hit_event, &env_shadow_ray);
                                    let weight = power_heuristic(env_pdf, material_pdf);
                                    direct_light += (weight * throughput * env_value * scatter_event.attenuation * scattering_pdf) / env_pdf;
                                }
                            }
                        }
                    }

                    color += direct_light;

                    let scattered_direction = scatter_event.pdf.generate(rng);
                    let material_pdf = scatter_event.pdf.value(scattered_direction);
                    if material_pdf <= 0.0 { break; }

                    let offset_point = find_offset_point(hit_event.point, hit_event.geometric_normal);
                    let scattered_ray = Ray::new(offset_point, scattered_direction, ray.time);
                    let scattering_pdf = material.scattering_pdf(&ray, &hit_event, &scattered_ray);

                    throughput *= (scattering_pdf * scatter_event.attenuation) / material_pdf;
                    ray = scattered_ray;
                     // store the material so that we don't need to perform a best estimate
                     // hit which would cost another traversal
                    last_material_pdf = material_pdf;
                }
            } else {
                if bounce == 0 {
                    first_albedo = emission;
                    first_normal = hit_event.shading_normal;
                }
                break;
            }
        } else {
            if bounce == 0 {
                if let Some(env) = &scene.environment {
                    first_albedo = env.value(0.0, 0.0, &ray.direction).clamp(Vec3::ZERO, Vec3::ONE);
                } else if scene.atmosphere {
                    let point: f32 = 0.5 * (ray.direction.y + 1.0);
                    first_albedo = (1.0 - point) * Vec3::splat(1.0) + point * Vec3::new(0.5, 0.7, 1.0);
                }
            }
            if let Some(env) = &scene.environment {
                let env_value = env.value(0.0, 0.0, &ray.direction);
                // apply MIS if this is an importance-sampled environment map and
                // the ray arrived via a material-sampled direction (not specular).
                let contribution = match env.env_pdf_value(&ray.direction) {
                    Some(env_pdf) if !last_specular && env_pdf > 0.0 => {
                        let weight = power_heuristic(last_material_pdf, env_pdf);
                        throughput * weight * env_value
                    }
                    _ => throughput * env_value,
                };
                color += contribution;
            } else if scene.atmosphere {
                let point: f32 = 0.5 * (ray.direction.y + 1.0);
                let lerp = (1.0 - point) * Vec3::splat(1.0) + point * Vec3::new(0.5, 0.7, 1.0);
                color += throughput * lerp;
            }
            break;
        }

        if bounce > 3 {
            let roulette_factor = (1.0 - throughput.max_element()).max(0.05);
            if rng.random::<f32>() < roulette_factor {
                break;
            }
            throughput /= 1.0 - roulette_factor;
        }
    }

    (color, first_albedo, first_normal)
}