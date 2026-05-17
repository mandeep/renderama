use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use materials::{Diffuse, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use primitive::Primitive;
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::Color;

use mat;


pub fn hyperion_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 3.0, 5.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.5);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 39.0;
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

    let metal_idx3 = mat!(materials, Refractive::new(1.5, Vec3A::ONE.into()));
    objects.push(Sphere::new(Vec3A::new(0.6, 0.70, -1.0), 0.5, metal_idx3).into());

    let metal_idx = mat!(materials, Reflective::new(Vec3A::new(0.93, 0.93, 0.93), 0.0));
    objects.push(Sphere::new(Vec3A::new(-0.6, 0.70, -1.0), 0.5, metal_idx).into());

    let metal_idx2 = mat!(materials, Reflective::new(Vec3A::new(0.6, 0.6, 0.6), 0.2));
    objects.push(Sphere::new(Vec3A::new(0.0, 0.70, -2.0), 0.5, metal_idx2).into());

    let floor_idx = mat!(materials, Diffuse::new(Color::new(0.5, 0.5, 0.5).into(), 0.0));
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(-50.0..50.0, -50.0..50.0), 0.0, floor_idx).into());
    
    let box_idx = mat!(materials, Diffuse::new(Color::new(0.75, 0.75, 0.75).into(), 0.0));
    let layout_box = Rectangle::new(Vec3A::new(-3.0, 0.0, -3.5), Vec3A::new(3.0, 0.2, 1.5), box_idx);
    objects.push(layout_box.into());

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("docs/textures/studio_small_08.exr").into();

    Scene::new(String::from("Hyperion"), bvh, materials, camera, vec![], Some(environment), None)
}