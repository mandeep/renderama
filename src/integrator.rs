use std::f32;

use glam::Vec3A;
use rand::rngs::ThreadRng;
use rand::RngExt;
use rand_distr::StandardNormal;

use pdf::power_heuristic;
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
pub fn pick_sphere_point(rng: &mut ThreadRng) -> Vec3A {
    let x: f32 = rng.sample(StandardNormal);
    let y: f32 = rng.sample(StandardNormal);
    let z: f32 = rng.sample(StandardNormal);

    Vec3A::new(x, y, z).normalize()
}

pub fn render_normals(ray: Ray, scene: &Scene) -> Vec3A {
    if let Some(hit) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
        let normal = hit.shading_normal;
        0.5 * Vec3A::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0)
    } else {
        let point = 0.5 * (ray.direction.y + 1.0);
        (1.0 - point) * Vec3A::new(1.0, 1.0, 1.0) + point * Vec3A::new(0.5, 0.7, 1.0)
    }
}

pub fn render_nee_integrator(mut ray: Ray, scene: &Scene, rng: &mut ThreadRng) -> (Vec3A, Vec3A, Vec3A) {
    let mut color = Vec3A::ZERO;
    let mut throughput = Vec3A::ONE;
    let mut should_weight_contribution = false; // flag for weighing contributions from specific materials like specular
    let mut previous_material_weight = 0.0;
    let mut first_albedo = Vec3A::ZERO;
    let mut first_normal = Vec3A::ZERO;

    let bounces = 10;

    for bounce in 0..=bounces {
        if let Some(hit_event) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) {
            let material = &scene.materials[hit_event.material_id.index()];

            let emission = material.evaluate_emission(&ray, &hit_event);
            if emission.length_squared() > 0.0 {
                if !should_weight_contribution {
                    color += throughput * emission;
                } else {
                    let light_weight: f32 = scene.lights.iter()
                        .map(|light| light.evaluate_sampling_weight(ray.origin, ray.direction))
                        .sum();
                    let weight = power_heuristic(previous_material_weight, light_weight);
                    color += throughput * weight * emission;
                }
            }

            if let Some(scatter_event) = material.generate_response(&ray, &hit_event, rng) {
                if bounce == 0 {
                    first_albedo = scatter_event.attenuation;
                    first_normal = hit_event.shading_normal;
                }
                if scatter_event.specular {
                    throughput *= scatter_event.attenuation;
                    ray = scatter_event.specular_ray;
                    should_weight_contribution = false;
                } else {
                    should_weight_contribution = true;

                    let mut direct_light = Vec3A::ZERO;

                    // using a manual offset instead of the find_offset_point for now as it
                    // gives better results on shadow rays
                    let shadow_origin = hit_event.point + hit_event.geometric_normal * 1e-3;

                    for light_source in &scene.lights {
                        let light_direction_vector = light_source.sample_direction_to_light(shadow_origin, rng);
                        let light_distance = light_direction_vector.length();
                        let light_direction = light_direction_vector.normalize();

                        let shadow_ray = Ray::new(shadow_origin, light_direction);
                        let end_distance = light_source.calculate_distance_from(light_distance);

                        if !scene.accelerator.hits_anything(&shadow_ray, 1e-3, end_distance) {
                            let light_weight = light_source.evaluate_sampling_weight(shadow_origin, light_direction);
                            if light_weight > 1e-7 {
                                let reflectance = material.compute_reflectance(&ray, &hit_event, &shadow_ray);
                                let material_weight = scatter_event.sampling_strategy.calculate_probability(light_direction);
                                let weight = power_heuristic(light_weight, material_weight);
                                direct_light += (weight * throughput * light_source.emission * scatter_event.attenuation * reflectance) / light_weight;
                            }
                        }
                    }

                    // environment map NEE samples toward bright regions via luminance CDF
                    // this prevents the high number of fireflies we saw in high contrast
                    // environment maps with luminance values as high as 100,000
                    if let Some(environment) = &scene.environment {
                        if let Some(environment_direction) = environment.sample_direction_to_light(rng) {
                            let environment_weight = environment.evaluate_sampling_weight(&environment_direction).unwrap_or(0.0);
                            if environment_weight > 1e-7 {
                                let shadow_origin = hit_event.point + hit_event.geometric_normal * 1e-3;
                                let environment_shadow_ray = Ray::new(shadow_origin, environment_direction);
                                if scene.accelerator.hit(&environment_shadow_ray, 1e-3, f32::MAX).is_none() {
                                    let environment_value = environment.generate_response(0.0, 0.0, &environment_direction);
                                    let material_weight = scatter_event.sampling_strategy.calculate_probability(environment_direction);
                                    let reflectance = material.compute_reflectance(&ray, &hit_event, &environment_shadow_ray);
                                    let weight = power_heuristic(environment_weight, material_weight);
                                    direct_light += (weight * throughput * environment_value * scatter_event.attenuation * reflectance) / environment_weight;
                                }
                            }
                        }
                    }

                    color += direct_light;

                    let scattered_direction = scatter_event.sampling_strategy.pick_direction(rng);
                    let material_weight = scatter_event.sampling_strategy.calculate_probability(scattered_direction);
                    if material_weight <= 0.0 { break; }

                    let offset_point = find_offset_point(hit_event.point, hit_event.geometric_normal);
                    let scattered_ray = Ray::new(offset_point, scattered_direction);
                    let reflectance = material.compute_reflectance(&ray, &hit_event, &scattered_ray);

                    // if we're using a material with pre-weighted ggx vndf
                    // then no need to compute the reflactance and weight
                    if scatter_event.pre_weighted {
                        throughput *= scatter_event.attenuation;
                    } else {
                        throughput *= (reflectance * scatter_event.attenuation) / material_weight;
                    }
                    ray = scattered_ray;
                     // store the material so that we don't need to perform a best estimate
                     // hit which would cost another traversal
                    previous_material_weight = material_weight;
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
                    first_albedo = env.generate_response(0.0, 0.0, &ray.direction).clamp(Vec3A::ZERO, Vec3A::ONE);
                } else if scene.atmosphere {
                    let point: f32 = 0.5 * (ray.direction.y + 1.0);
                    first_albedo = (1.0 - point) * Vec3A::splat(1.0) + point * Vec3A::new(0.5, 0.7, 1.0);
                }
            }
            if let Some(environment) = &scene.environment {
                let environment_response = environment.generate_response(0.0, 0.0, &ray.direction);
                // apply MIS if this is an importance-sampled environment map and
                // the ray arrived via a material-sampled direction (not specular).
                let contribution = match environment.evaluate_sampling_weight(&ray.direction) {
                    Some(environment_weight) if should_weight_contribution && environment_weight > 0.0 => {
                        let weight = power_heuristic(previous_material_weight, environment_weight);
                        throughput * weight * environment_response
                    }
                    _ => throughput * environment_response,
                };
                color += contribution;
            } else if scene.atmosphere {
                let point: f32 = 0.5 * (ray.direction.y + 1.0);
                let lerp = (1.0 - point) * Vec3A::splat(1.0) + point * Vec3A::new(0.5, 0.7, 1.0);
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