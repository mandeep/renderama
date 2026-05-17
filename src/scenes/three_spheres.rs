use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use materials::{Diffuse, Reflective, Refractive, Material, Plastic, Volumetric};
use primitive::Primitive;
use scene::Scene;
use sphere::Sphere;
use texture::Color;
use volume::Volume;

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

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let refr_idx = mat!(materials, Refractive::new(Color::new(1.0, 1.0, 1.0).into(), 1.5));
    let vol_idx = mat!(materials, Volumetric::new(Color::new(0.0, 0.4, 0.9).into()));

    let boundary: Primitive = Sphere::new(Vec3A::new(0.6, 0.0, -1.0), 0.5, refr_idx).into();
    let cloned_boundary = boundary.clone();

    objects.push(boundary);
    objects.push(Volume::new(4.0, cloned_boundary, vol_idx).into());

    let metal_idx = mat!(materials, Reflective::new(Color::new(0.93, 0.93, 0.93).into(), 0.0));
    objects.push(Sphere::new(Vec3A::new(-0.6, 0.0, -1.0), 0.5, metal_idx).into());

    let plastic_idx = mat!(materials, Plastic::new(Color::new(0.34, 0.57, 1.0).into(), 0.10, 1.5));
    objects.push(Sphere::new(Vec3A::new(0.0, 0.0, -2.0), 0.5, plastic_idx).into());

    let floor_idx = mat!(materials, Diffuse::new(Color::new(0.5, 0.5, 0.52).into(), 0.0));
    objects.push(Sphere::new(Vec3A::new(0.0, -100.5, -1.0), 100.0, floor_idx).into());

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("docs/textures/dusk_1_puresky.exr").into();

    Scene::new(String::from("Three Spheres"), bvh, materials, camera, vec![], Some(environment), None)
}