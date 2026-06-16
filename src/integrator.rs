use std::f32;

use clap::ValueEnum;
use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use crate::basis::OrthonormalBasis;
use crate::lights::Light;
use crate::materials::Material;
use crate::pdf::power_heuristic;
use crate::ray::{find_offset_point, Ray};
use crate::results::{HitResult, ScatterResult};
use crate::sampling::cosine_sample_hemisphere;
use crate::scene::Scene;
use crate::texture::Texture;

/// Integrator houses all of the possible integrators one can use
/// to render a scene.
#[derive(Copy, Clone, ValueEnum)]
pub enum Integrator {
    Beauty,
    Normals,
    AmbientOcclusion,
}

impl Integrator {
    /// Set a default number of samples in case the number of samples desired
    /// is not passed as a command line argument.
    pub fn default_samples(&self) -> usize {
        match self {
            Integrator::Beauty => 64,
            Integrator::Normals => 1,
            Integrator::AmbientOcclusion => 16,
        }
    }

    /// Dispatch the integrator chosen by the user in the command line interface
    pub fn render_scene(&self, ray: Ray, scene: &Scene, rng: &mut Pcg64Mcg) -> Vec3A {
        match self {
            Integrator::Beauty => render_beauty(ray, &scene, rng),
            Integrator::Normals => render_normals(ray, &scene, rng),
            Integrator::AmbientOcclusion => render_ambient_occlusion(ray, &scene, rng),
        }
    }
}

/// Render a normal pass.
///
/// Typically used in debugging whether or not the geometry in
/// the scene is setup correctly.
pub fn render_normals(ray: Ray, scene: &Scene, rng: &mut Pcg64Mcg) -> Vec3A {
    if let Some(hit_result) = scene.accelerator.hit(&ray, 1e-4, f32::MAX, rng) {
        let normal = hit_result.shading_normal;
        0.5 * Vec3A::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0)
    } else {
        let point = 0.5 * (ray.direction.y + 1.0);
        (1.0 - point) * Vec3A::new(1.0, 1.0, 1.0) + point * Vec3A::new(0.5, 0.7, 1.0)
    }
}

/// Render a scene with ambient occlusion.
///
/// This integrator renders all geometry as white and shadows as black.
/// Extremely useful in viewing how light interacts with the geometry in a scene
/// prior to running a beauty pass. A high number of samples can be rendered
/// relatively quickly as well.
///
/// References:
/// https://github.com/mmp/pbrt-v3/blob/master/src/integrators/ao.cpp#L71
/// https://rmanwiki-26.pixar.com/space/REN26/19661789/PxrOcclusion
/// https://developer.nvidia.com/gpugems/gpugems/part-iii-materials/chapter-17-ambient-occlusion
pub fn render_ambient_occlusion(ray: Ray, scene: &Scene, rng: &mut Pcg64Mcg) -> Vec3A {
    let mut color: f32 = 0.0;

    let Some(hit_result) = scene.accelerator.hit(&ray, 1e-4, f32::MAX, rng) else {
        return Vec3A::ONE;
    };

    let uvw = OrthonormalBasis::new(&hit_result.shading_normal);
    let local = cosine_sample_hemisphere(rng);
    let direction = uvw.local(&local);
    let offset_point = find_offset_point(hit_result.point, hit_result.geometric_normal);
    let ao_ray = Ray::new(offset_point, direction, ray.time);

    if !scene.accelerator.hits_anything(&ao_ray, 1e-3, f32::MAX, rng) {
        color += 1.0;
    }
    Vec3A::splat(color)
}

/// Render a beauty pass.
///
/// This integrator renders your typical beauty render pass. It also outputs albedo and
/// normal buffers for the denoiser.
///
/// A combination of BSDF sampling and many light sampling is used to provide
/// physically correct results. Bounces are set to a default of 10 though russian
/// roulette is applied after 3 bounces.
pub fn render_beauty(mut ray: Ray, scene: &Scene, rng: &mut Pcg64Mcg) -> Vec3A {
    let mut color = Vec3A::ZERO;
    let mut throughput = Vec3A::ONE;
    let mut previous_bounce = PreviousBounce::None;

    let bounces = 10;

    for bounce in 0..=bounces {
        let Some(hit_result) = scene.accelerator.hit(&ray, 1e-4, f32::MAX, rng) else {
            color +=
                evaluate_missed_ray(&ray, &previous_bounce, &scene, &throughput);
            break;
        };

        let material = &scene.materials[hit_result.material_id.index()];

        color += evaluate_emission(
            &ray, &hit_result, &material, &scene.textures, &previous_bounce, &scene.lights, &throughput
        );

        let Some(scatter_result) = material.generate_response(&ray, &hit_result, &scene.textures, rng) else { break };

        let pre_weighted = scatter_result.sampling_strategy.is_delta();
        if pre_weighted {
            throughput *= scatter_result.contribution;
            ray = scatter_result.scattered_ray;
            previous_bounce = PreviousBounce::Specular;
        } else {
            color += evaluate_direct_lighting(&ray, &hit_result, &material, &scatter_result, &scene, &throughput, rng);

            let Some((next_ray, throughput_factor, weight)) = prepare_next_ray(&ray, &hit_result, &material, &scene.textures, &scatter_result, rng) else { break };
            throughput *= throughput_factor;
            ray = next_ray;
            previous_bounce = PreviousBounce::Diffuse(weight);
        }

        if bounce > 3 {
            let Some(new_throughput) = apply_roulette(&throughput, rng) else { break };
            throughput = new_throughput;
        }
    }

    color
}

// Store the material's weight so that we don't need to perform a best estimate
// hit which would cost another traversal
enum PreviousBounce {
    None,
    Specular,
    Diffuse(f32),
}

/// Evaluate what happens when a ray misses all geometry in the scene.
///
/// Currently environment maps and atmospheres are supported. If neither are
/// given, the integrator will render a black background for all missed rays.
fn evaluate_missed_ray(
    ray: &Ray,
    previous_bounce: &PreviousBounce,
    scene: &Scene,
    throughput: &Vec3A
) -> Vec3A {
    if let Some(environment) = &scene.environment {
        let (environment_response, environment_weight) =
            environment.evaluate_sampling_weight(&ray.direction);
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

/// Evaluate the emission from the light source's Emissive material.
///
/// While the intensity and contribution of the light in the scene
/// is handled by the Light type, here we can sample the light's
/// material for cases when rays miss the light source.
fn evaluate_emission(
    ray: &Ray,
    hit_result: &HitResult,
    material: &Material,
    textures: &[Texture],
    previous_bounce: &PreviousBounce,
    lights: &[Light],
    throughput: &Vec3A,
) -> Vec3A {
    let mut color = Vec3A::ZERO;

    let emission = material.evaluate_emission(&ray, &hit_result, &textures);
    if emission.length_squared() > 0.0 {
        match previous_bounce {
            PreviousBounce::Specular | PreviousBounce::None => {
                color += throughput * emission;
            }
            PreviousBounce::Diffuse(previous_weight) => {
                let light_weight: f32 = lights.iter()
                        .map(|light| light.evaluate_sampling_weight(ray))
                        .sum();
                    let weight = power_heuristic(*previous_weight, light_weight);
                    color += throughput * weight * emission;
            }
        }
    }

    color
}

/// Evaluate the contribution from the lights in the scene.
///
/// Shadow rays are sent to the light sources to determine the contribution
/// they directly add to the scene. In the case of environment maps, shadow rays
/// are sent to the areas of highest luminance.
fn evaluate_direct_lighting(
    ray: &Ray,
    hit_result: &HitResult,
    material: &Material,
    scatter_result: &ScatterResult,
    scene: &Scene,
    throughput: &Vec3A,
    rng: &mut Pcg64Mcg,
) -> Vec3A {
    let mut direct_light = Vec3A::ZERO;

    // using a manual offset instead of the find_offset_point for now as it
    // gives better results on shadow rays. seems like changes in the plane's light
    // calculation fixed issues with offset points, so changing this back to using the
    // find_offset_point function
    // let shadow_origin = hit_result.point + hit_result.geometric_normal * 1e-3;
    let shadow_origin = find_offset_point(hit_result.point, hit_result.geometric_normal);

    for light_source in &scene.lights {
        let light_direction_vector = light_source.sample_direction_to_light(shadow_origin, rng);
        let light_distance = light_direction_vector.length();
        let light_direction = light_direction_vector.normalize();

        let shadow_ray = Ray::new(shadow_origin, light_direction, ray.time);
        let end_distance = light_source.calculate_distance_from(light_distance);

        if !scene.accelerator.hits_anything(&shadow_ray, 1e-3, end_distance, rng) {
            let light_weight = light_source.evaluate_sampling_weight(&shadow_ray);
            if light_weight > 1e-7 {
                let reflectance = material.compute_reflectance(&ray, &shadow_ray, &hit_result, &scene.textures);
                let material_weight = scatter_result.sampling_strategy.calculate_probability(light_direction);
                let weight = power_heuristic(light_weight, material_weight);
                direct_light += (weight * throughput * light_source.intensity * scatter_result.contribution * reflectance) / light_weight;
            }
        }
    }

    if let Some(environment) = &scene.environment {
        let (environment_direction, environment_value, environment_weight) =
            environment.sample_direction_to_light(rng);
        if environment_weight > 1e-7 {
            let shadow_origin = find_offset_point(hit_result.point, hit_result.geometric_normal);
            let environment_shadow_ray = Ray::new(shadow_origin, environment_direction, ray.time);
            if !scene.accelerator.hits_anything(&environment_shadow_ray, 1e-3, f32::MAX, rng) {
                let material_weight = scatter_result.sampling_strategy.calculate_probability(environment_direction);
                let reflectance = material.compute_reflectance(&ray, &environment_shadow_ray, &hit_result, &scene.textures);
                let weight = power_heuristic(environment_weight, material_weight);
                direct_light += (
                    weight *
                    throughput *
                    environment_value *
                    scatter_result.contribution *
                    reflectance) / environment_weight;
            }
        }
    }

    direct_light
}

/// Prepare the next bounce of the ray currently being bounced around the scene.
///
/// In addition to generating a scattered ray, this function also determines the
/// throughput to carry on to the next bounce. The weight of the material is also
/// returned so that it can be passed to the PreviousBounce enum.
fn prepare_next_ray(
    ray: &Ray,
    hit_result: &HitResult,
    material: &Material,
    textures: &[Texture],
    scatter_result: &ScatterResult,
    rng: &mut Pcg64Mcg,
) -> Option<(Ray, Vec3A, f32)> {
    let scattered_direction = scatter_result.sampling_strategy.pick_direction(rng);
    let material_weight = scatter_result.sampling_strategy.calculate_probability(scattered_direction);
    if material_weight <= 0.0 { return None; }

    let offset_point = find_offset_point(hit_result.point, hit_result.geometric_normal);
    let scattered_ray = Ray::new(offset_point, scattered_direction, ray.time);
    let reflectance = material.compute_reflectance(&ray, &scattered_ray, &hit_result, &textures);

    // if we're using a material with pre-weighted ggx vndf
    // then no need to compute the reflectance and weight
    let pre_weighted = scatter_result.sampling_strategy.is_delta();
    let throughput = if pre_weighted {
        scatter_result.contribution
    } else {
        (reflectance * scatter_result.contribution) / material_weight
    };

    Some((scattered_ray, throughput, material_weight))
}

/// Terminate rays with low throughput based on russian roulette.
///
/// Reference:
/// https://pbr-book.org/3ed-2018/Monte_Carlo_Integration/Russian_Roulette_and_Splitting
fn apply_roulette(throughput: &Vec3A, rng: &mut Pcg64Mcg) -> Option<Vec3A> {
    let roulette_factor = (1.0 - throughput.max_element()).max(0.05);

    if rng.random::<f32>() < roulette_factor {
        return None;
    }

    let new_throughput = throughput / (1.0 - roulette_factor);

    Some(new_throughput)
}
