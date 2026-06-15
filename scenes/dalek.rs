use std::f32;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io;
use crate::materials::{Material, Plastic};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;


pub fn dalek_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-2.0, 2.5, 3.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(1600) as f32, height.unwrap_or(1600) as f32);
    let sensor_height = 56.0;
    let focal_length = 150.0;
    let world_scale = 0.001;
    let f_stop = 8.0;
    let focus_distance = 3.5;

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

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let default_texture = Color::new(1.0, 0.0, 0.0);
    let default_material = Plastic::new(1.0, 1.49);

    let (meshes, lights) = io::load_obj("extras/models/dalek_sec.obj", &mut materials, &mut textures, None, default_material, default_texture);

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/studio_01.exr", 1.0);

    SceneBuilder::new("Dalek Sec")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .with_lights(lights)
        .build()
        .expect("Failed to build Dalek Sec scene")
}