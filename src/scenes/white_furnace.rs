use std::f32;
use std::f32::consts::PI;

use glam::Vec3A;

use atmosphere::Atmosphere;
use bvh::BVH;
use camera::Camera;
use materials::{Material, Reflective};
use scene::{Scene, SceneBuilder};
use sphere::Sphere;
use texture::{Color};

use mat;

pub fn white_furnace_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -50.0);
    let lookat = Vec3A::new(278.0, 278.0, 300.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let old_fov = 25.0;
    let (aspect_width, aspect_height) = (width.unwrap_or(2048) as f32, height.unwrap_or(512) as f32);
    let fov_radians = old_fov * PI / 180.0;
    let sensor_height = 24.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let f_stop = std::f32::INFINITY;
    let focus_distance = 10.0;
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
        (aspect_width, aspect_height),
        0.0, 0.0,
    );

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let count = 10;
    let radius = 20.0;
    let spacing = 55.0;
    let start_x = 278.0 - ((count as f32 - 1.0) * spacing / 2.0);

    for i in 0..count {
        let roughness = i as f32 * 0.10 + 0.001; // add 0.001 so that we are testing the NEE path for roughness 0.0
        let x_pos = start_x + (i as f32 * spacing);
        
        let mat_id = mat!(materials, Reflective::new(Color::new(1.0, 1.0, 1.0).into(), roughness));
        
        objects.push(Sphere::new(Vec3A::new(x_pos, 278.0, 278.0), radius, mat_id).into());
    }

    let bvh = BVH::new(&mut objects);

    let atmosphere = Atmosphere::new(Vec3A::ONE, false);

    SceneBuilder::new("White Furnace Test")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_atmosphere(atmosphere)
        .build()
        .expect("Failed to build White Furnace Test scene")
}