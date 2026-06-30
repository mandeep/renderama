use glam::Vec3A;

use crate::atmosphere::Atmosphere;
use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::PushInto;
use crate::materials::{Material, Reflective};
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};

use crate::{mat, tex};

pub fn white_furnace_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -50.0);
    let lookat = Vec3A::new(278.0, 278.0, 300.0);
    let focal_length = 20.0;
    let focus_distance = 10.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_focal_length(focal_length)
        .with_focus_distance(focus_distance)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(512));
    let camera = Camera::new(&camera_options);

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let count = 10;
    let radius = 20.0;
    let spacing = 55.0;
    let start_x = 278.0 - ((count as f32 - 1.0) * spacing / 2.0);

    for i in 0..count {
        let roughness = i as f32 * 0.10 + 0.001; // add 0.001 so that we are testing the NEE path for roughness 0.0
        let x_pos = start_x + (i as f32 * spacing);

        let tex_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
        let mat_id = mat!(materials, Reflective::new(tex_id, roughness));
        
        objects.push_into(Sphere::new(Vec3A::new(x_pos, 278.0, 278.0), radius, mat_id));
    }

    let bvh = BVH::new(&mut objects);

    let atmosphere = Atmosphere::new(Vec3A::ONE, false);

    SceneBuilder::new("White Furnace Test")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_atmosphere(atmosphere)
        .build()
        .expect("Failed to build White Furnace Test scene")
}