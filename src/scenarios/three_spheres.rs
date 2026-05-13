use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use materials::{Diffuse, Reflective, Refractive, Material};
use primitive::Primitive;
use scene::Scene;
use sphere::Sphere;
use texture::{EnvironmentMap, SolidColor};
use world::World;
use mat;


pub fn three_spheres_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 3.0, 5.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.5);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(1024) as f32);
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));


    world.add(Primitive::Sphere(Sphere::new(Vec3A::new(0.6, 0.0, -1.0),
                          0.5,
                          red_idx)));

    let metal_idx = mat!(materials, Reflective::new(Vec3A::new(0.5, 0.5, 0.5), 0.01));

    world.add(Primitive::Sphere(Sphere::new(Vec3A::new(-0.6, 0.0, -1.0),
                          0.5,
                          metal_idx)));

    let metal_idx2 = mat!(materials, Reflective::new(Vec3A::new(0.5, 0.5, 0.5), 0.2));

    world.add(Primitive::Sphere(Sphere::new(Vec3A::new(0.0, 0.1, -2.0),
                          0.5,
                          metal_idx2)));

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));
    world.add(Primitive::Sphere(Sphere::new(Vec3A::new(0.0, -100.5, -1.0),
                          100.0,
                          floor_idx)));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let environment = EnvironmentMap::new("docs/textures/golden_gate_hills.exr").into();

    Scene::new(String::from("Three Spheres"), bvh, materials, camera, vec![], Some(environment), false)
}