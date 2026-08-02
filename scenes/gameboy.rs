use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::{AddMaterial, AddTexture, PushInto};
use crate::io::load_obj;
use crate::materials::{Material, Plastic};
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;


pub fn gameboy_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-2.0, 3.5, 9.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let sensor_width = 56.0;
    let focal_length = 150.0;
    let world_scale = 0.001;
    let f_stop = 8.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_sensor_width(sensor_width)
        .with_focal_length(focal_length)
        .with_fstop(f_stop)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(1600), height.unwrap_or(1600));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let meshes = load_obj("extras/models/gameboy/gameboy.obj", &mut materials, &mut textures, None);

    let (translation, rotation, scale) = (Vec3A::ZERO, Vec3A::new(0.0, -50.0, 0.0), Vec3A::ONE);
    for mesh in meshes {
        let transformed = TransformedMesh::new(translation, rotation, scale, mesh);
        objects.push_into(transformed);
    }

    let grey_id = textures.add_texture(Color::new(0.796, 0.776, 0.746));
    let grey = materials.add_material(Plastic::new(grey_id, 0.75, 1.50));
    // floor plane
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(-1000.0..1000.0, -1000.0..1000.0), -0.35, Orientation::Forward, grey));

    let bvh = BVH::new(objects);

    let environment = EnvironmentMap::new("extras/textures/white_studio_03.exr", 1.0);

    SceneBuilder::new("Gameboy")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Gameboy scene")
}