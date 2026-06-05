use std::f32;
use std::f32::consts::PI;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Emissive, Material, Reflective};
use plane::{Axis, Bounds2D, Plane};
use scene::{Scene, SceneBuilder};
use sphere::Sphere;
use texture::Color;

use mat;

pub fn energy_conservation_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -500.0);
    let lookat = Vec3A::new(278.0, 278.0, 300.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(2048) as f32, height.unwrap_or(512) as f32);
    let sensor_height = 24.0;
    let old_fov = 10.0;
    let fov_radians = old_fov * PI / 180.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let f_stop = std::f32::INFINITY;
    let focus_distance = (lookat - origin).length();
    let world_scale = 1.0;

    let camera = Camera::new(
        origin,
        lookat,
        view,
        focal_length,
        f_stop,
        sensor_height,
        focus_distance,
        world_scale,
        (aspect_width, aspect_height)
    );

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let count = 10;
    let radius = 20.0;
    let spacing = 55.0;
    let start_x = 278.0 - ((count as f32 - 1.0) * spacing / 2.0);

    for i in 0..count {
        let roughness = i as f32 * 0.10;
        let x_pos = start_x + (i as f32 * spacing);

        let mat_id = mat!(materials, Reflective::new(Color::new(1.0, 1.0, 1.0).into(), roughness));
        objects.push(Sphere::new(Vec3A::new(x_pos, 278.0, 278.0), radius, mat_id).into());
    }

    let bvh = BVH::new(&mut objects);

    let light_material = mat!(materials, Emissive::new(Color::new(50.0, 50.0, 50.0).into()));
    let light_plane = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 600.0, light_material);
    let light = vec![Light::new(light_plane.into(), Vec3A::new(50.0, 50.0, 50.0))];

    SceneBuilder::new("Energy Conservation Test")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(light)
        .build()
        .expect("Failed to build Energy Conservation Test scene")
}