use std::f32;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use materials::{Diffuse, Reflective, Refractive, Material};
use scene::Scene;
use sphere::Sphere;
use texture::{EnvironmentMap, SolidColor, Texture};
use world::World;
use mat;

pub fn random_spheres_scene(width: usize, height: usize) -> Scene {
    let origin = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let environment = EnvironmentMap::new("models/pure_sky_qwantani.exr").into();

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));

    world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
                          Vec3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          floor_idx,
                          0.0,
                          1.0).into());

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vec3 = Vec3::new(a as f32 + 0.9 * rand::random::<f32>(),
                                         0.2,
                                         b as f32 + 0.9 * rand::random::<f32>());

            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if material < 0.75 {
                    let material = SolidColor::new(rand::random::<f32>() * rand::random::<f32>(),
                                                   rand::random::<f32>() * rand::random::<f32>(),
                                                   rand::random::<f32>() * rand::random::<f32>());
                    // let roughness = rand::distributions::Uniform::new(0.0, 1.0);
                    let random_idx = mat!(materials, Diffuse::new(material.into(), 0.0));
                    world.add(Sphere::new(center, center, 0.2, random_idx, 0.0, 1.0).into());
                } else if material < 0.95 {
                    let material = Reflective::new(Vec3::new(0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>()),
                                                                    0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>()),
                                                                    0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>())),
                                                          0.5 * rand::random::<f32>());
                    let random_idx = mat!(materials, material);
                    world.add(Sphere::new(center,
                                          center,
                                          0.2,
                                          random_idx,
                                          0.0,
                                          1.0).into());
                } else {
                    let refl_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));
                    world.add(Sphere::new(center, center, 0.2, refl_idx, 0.0, 1.0).into());
                    let refr_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));
                    world.add(Sphere::new(center, center, -0.19,refr_idx, 0.0, 1.0).into());
                }
            }
        }
    }
    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));
    world.add(Sphere::new(Vec3::new(-2.0, 1.0, 0.0),
                          Vec3::new(-2.0, 1.0, 0.0),
                          1.0,
                          red_idx,
                          0.0,
                          1.0).into());

    let refr_idx1 = mat!(materials, Refractive::new(1.5, Vec3::ONE));
    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          1.0,
                          refr_idx1,
                          0.0,
                          1.0).into());

    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          -0.99,
                          refr_idx1,
                          0.0,
                          1.0).into());

    let refl_idx = mat!(materials, Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.05));
    world.add(Sphere::new(Vec3::new(2.0, 1.0, 0.0),
                          Vec3::new(2.0, 1.0, 0.0),
                          1.0,
                          refl_idx,
                          0.0,
                          1.0).into());

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = None;

    Scene::new(String::from("Random Spheres"), bvh, materials, camera, light, Some(environment))
}