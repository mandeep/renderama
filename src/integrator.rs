use std::f32;
use std::str::FromStr;

use clap::ValueEnum;
use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use events::{HitEvent, ScatterEvent};
use lights::Light;
use materials::Material;
use pdf::power_heuristic;
use ray::{find_offset_point, Ray};
use scene::Scene;

#[derive(Copy, Clone, ValueEnum)]
pub enum Integrator {
    Beauty,
    Normals,
}

impl FromStr for Integrator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "beauty" => Ok(Integrator::Beauty),
            "normals" => Ok(Integrator::Normals),
            _ => Err(format!("Unknown integrator: {}", s)),
        }
    }
}

impl Integrator {
    pub fn default_samples(&self) -> usize {
        match self {
            Integrator::Beauty => 64,
            Integrator::Normals => 1,
        }
    }
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

pub fn render_beauty(mut ray: Ray, scene: &Scene, rng: &mut Pcg64Mcg) -> (Vec3A, Vec3A, Vec3A) {
    let mut color = Vec3A::ZERO;
    let mut throughput = Vec3A::ONE;
    let mut previous_bounce = PreviousBounce::None;
    let mut first_albedo = Vec3A::ZERO;
    let mut first_normal = Vec3A::ZERO;

    let bounces = 10;

    for bounce in 0..=bounces {
        let Some(hit_event) = scene.accelerator.hit(&ray, 1e-4, f32::MAX) else {
            color +=
                evaluate_miss(&ray, &previous_bounce, &scene, &throughput);
            break;
        };

        let material = &scene.materials[hit_event.material_id.index()];

        color += evaluate_emission(
            &ray, &hit_event, &material, &previous_bounce, &scene.lights, &throughput
        );

        let Some(scatter_event) = material.generate_response(&ray, &hit_event, rng) else { break };

        if bounce == 0 {
            first_albedo = scatter_event.attenuation;
            first_normal = hit_event.shading_normal;
        }

        if scatter_event.specular {
            throughput *= scatter_event.attenuation;
            ray = scatter_event.specular_ray;
            previous_bounce = PreviousBounce::Specular;
        } else {
            color += sample_direct_lighting(&ray, &hit_event, &material, &scatter_event, &scene, &throughput, rng);

            let Some((next_ray, throughput_factor, weight)) = prepare_next_ray(&ray, &hit_event, &material, &scatter_event, rng) else { break };
            throughput *= throughput_factor;
            ray = next_ray;
            previous_bounce = PreviousBounce::Diffuse(weight);
        }

        if bounce > 3 {
            let Some(new_throughput) = apply_roulette(&throughput, rng) else { break };
            throughput = new_throughput;
        }
    }

    (color, first_albedo, first_normal)
}

// store the material's weight so that we don't need to perform a best estimate
// hit which would cost another traversal
enum PreviousBounce {
    None,
    Specular,
    Diffuse(f32),
}

fn evaluate_miss(
    ray: &Ray,
    previous_bounce: &PreviousBounce,
    scene: &Scene,
    throughput: &Vec3A
) -> Vec3A {
    if let Some(environment) = &scene.environment {
        let environment_response = environment.sample_map(0.0, 0.0, &ray.direction);
        let environment_weight = environment.evaluate_sampling_weight(&ray.direction);
        let mut contribution = throughput * environment_response;

        match previous_bounce {
            PreviousBounce::Diffuse(previous_weight) => {
                if environment_weight > 0.0 {
                        let weight = power_heuristic(*previous_weight, environment_weight);
                        contribution *= weight;
                };
            },
            PreviousBounce::Specular | PreviousBounce::None => (),
        }

        return contribution;

    } else if let Some(atmosphere) = &scene.atmosphere {
        let atmosphere_color = atmosphere.compute_atmosphere_color(&ray.direction);
        return throughput * atmosphere_color;
    }

    Vec3A::ZERO
}

fn evaluate_emission(
    ray: &Ray,
    hit_event: &HitEvent,
    material: &Material,
    previous_bounce: &PreviousBounce,
    lights: &[Light],
    throughput: &Vec3A,
) -> Vec3A {
    let mut color = Vec3A::ZERO;

    let emission = material.evaluate_emission(&ray, &hit_event);
    if emission.length_squared() > 0.0 {
        match previous_bounce {
            PreviousBounce::Specular | PreviousBounce::None => {
                color += throughput * emission;
            }
            PreviousBounce::Diffuse(previous_weight) => {
                let light_weight: f32 = lights.iter()
                        .map(|light| light.evaluate_sampling_weight(ray.origin, ray.direction))
                        .sum();
                    let weight = power_heuristic(*previous_weight, light_weight);
                    color += throughput * weight * emission;
            }
        }
    }

    color
}

fn sample_direct_lighting(
    ray: &Ray,
    hit_event: &HitEvent,
    material: &Material,
    scatter_event: &ScatterEvent,
    scene: &Scene,
    throughput: &Vec3A,
    rng: &mut Pcg64Mcg,
) -> Vec3A {
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

    if let Some(environment) = &scene.environment {
        let environment_direction = environment.sample_direction_to_light(rng);
        let environment_weight = environment.evaluate_sampling_weight(&environment_direction);
        if environment_weight > 1e-7 {
            let shadow_origin = hit_event.point + hit_event.geometric_normal * 1e-3;
            let environment_shadow_ray = Ray::new(shadow_origin, environment_direction);
            if scene.accelerator.hit(&environment_shadow_ray, 1e-3, f32::MAX).is_none() {
                let environment_value = environment.sample_map(0.0, 0.0, &environment_direction);
                let material_weight = scatter_event.sampling_strategy.calculate_probability(environment_direction);
                let reflectance = material.compute_reflectance(&ray, &hit_event, &environment_shadow_ray);
                let weight = power_heuristic(environment_weight, material_weight);
                direct_light += (weight *
                    throughput *
                    environment_value *
                    scatter_event.attenuation *
                    reflectance) / environment_weight;
            }
        }
    }

    direct_light
}

fn prepare_next_ray(
    ray: &Ray,
    hit_event: &HitEvent,
    material: &Material,
    scatter_event: &ScatterEvent,
    rng: &mut Pcg64Mcg,
) -> Option<(Ray, Vec3A, f32)> {
    let scattered_direction = scatter_event.sampling_strategy.pick_direction(rng);
    let material_weight = scatter_event.sampling_strategy.calculate_probability(scattered_direction);
    if material_weight <= 0.0 { return None; }

    let offset_point = find_offset_point(hit_event.point, hit_event.geometric_normal);
    let scattered_ray = Ray::new(offset_point, scattered_direction);
    let reflectance = material.compute_reflectance(&ray, &hit_event, &scattered_ray);

    // if we're using a material with pre-weighted ggx vndf
    // then no need to compute the reflactance and weight
    let throughput = if scatter_event.pre_weighted {
        scatter_event.attenuation
    } else {
        (reflectance * scatter_event.attenuation) / material_weight
    };

    Some((scattered_ray, throughput, material_weight))
}

fn apply_roulette(throughput: &Vec3A, rng: &mut Pcg64Mcg) -> Option<Vec3A> {
    let roulette_factor = (1.0 - throughput.max_element()).max(0.05);

    if rng.random::<f32>() < roulette_factor {
        return None;
    }

    let new_throughput = throughput / (1.0 - roulette_factor);

    Some(new_throughput)
}
