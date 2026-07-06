use glam::Vec3A;
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::{AddMaterial, AddTexture, PushInto};
use crate::materials::{Diffuse, Material, Reflective, Refractive};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};



pub fn random_spheres_scene(width: Option<usize>, height: Option<usize>, rng: &mut Pcg64Mcg) -> Scene {
    let origin = Vec3A::new(13.0, 2.0, 3.0);
    let lookat = Vec3A::new(0.0, 0.0, 0.0);
    let fov = 39.0;
    let f_stop = 0.7;
    let focus_distance = 10.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_focus_distance(focus_distance)
        .with_fstop(f_stop)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(1024));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let environment = EnvironmentMap::new("extras/textures/pure_sky_qwantani.exr", 1.0);

    let floor_id = textures.add_texture(Color::new(0.5, 0.5, 0.5));
    let floor_idx = materials.add_material(Diffuse::new(floor_id, 0.0));
    let refr_id = textures.add_texture(Color::new(1.0, 1.0, 1.0));
    let refr_idx = materials.add_material(Refractive::new(refr_id, 1.5));

    objects.push_into(Sphere::new(Vec3A::new(0.0, -1000.0, 0.0),
                          1000.0,
                          floor_idx));

    let red_center = Vec3A::new(-2.0, 1.0, 0.0);
    let glass_center = Vec3A::new(0.0, 1.0, 0.0);
    let metal_center = Vec3A::new(2.0, 1.0, 0.0);

    let mut placed: Vec<(Vec3A, f32)> = vec![
        (red_center, 1.0),
        (glass_center, 1.0),
        (metal_center, 1.0),
    ];

    for a in -11..11 {
        for b in -11..11 {
            let material = rng.random::<f32>();
            let center: Vec3A = Vec3A::new(
                a as f32 + 0.9 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.random::<f32>());

            let overlaps = placed.iter().any(|(c, r)| {
                (center - *c).length() < 0.2 + r + 0.05
            });
            if overlaps { continue; }
            placed.push((center, 0.2));

            if material < 0.75 {
                let material = Color::new(
                    rng.random::<f32>() * rng.random::<f32>(),
                    rng.random::<f32>() * rng.random::<f32>(),
                    rng.random::<f32>() * rng.random::<f32>());
                let random_id = textures.add_texture(material);
                let random_idx = materials.add_material(Diffuse::new(random_id, 0.0));
                objects.push_into(Sphere::new(center, 0.2, random_idx));
            } else if material < 0.95 {
                let random_id = textures.add_texture(Color::new(
                    0.5 * (1.0 * rng.random::<f32>()),
                    0.5 * (1.0 * rng.random::<f32>()),
                    0.5 * (1.0 * rng.random::<f32>())));
                let random_idx = materials.add_material(Reflective::new(random_id, 0.5 * rng.random::<f32>()));
                objects.push_into(
                    Sphere::new(center, 0.2, random_idx));
            } else {
                    objects.push_into(Sphere::new(center, 0.2, refr_idx));
                    objects.push_into(Sphere::new(center, -0.19, refr_idx));
            }
        }
    }

    let red_id = textures.add_texture(Color::new(0.75, 0.25, 0.25));
    let red_idx = materials.add_material(Diffuse::new(red_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(-2.0, 1.0, 0.0),
                          1.0,
                          red_idx));

    objects.push_into(Sphere::new(Vec3A::new(0.0, 1.0, 0.0),
                          1.0,
                          refr_idx));

    objects.push_into(Sphere::new(Vec3A::new(0.0, 1.0, 0.0),
                          -0.99,
                          refr_idx));

    let refl_id = textures.add_texture(Color::new(0.5, 0.5, 0.5));
    let refl_idx = materials.add_material(Reflective::new(refl_id, 0.065));
    objects.push_into(Sphere::new(Vec3A::new(2.0, 1.0, 0.0),
                          1.0,
                          refl_idx));

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Random Spheres")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Random Spheres scene")
}