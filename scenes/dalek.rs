use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::load_obj;
use crate::materials::Material;
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::Texture;
use crate::transformations::TransformedMesh;

pub fn dalek_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-2.0, 2.5, 3.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let sensor_width = 56.0;
    let focal_length = 150.0;
    let f_stop = 8.0;
    let focus_distance = 3.5;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_sensor_width(sensor_width)
        .with_focal_length(focal_length)
        .with_fstop(f_stop)
        .with_focus_distance(focus_distance)
        .with_resolution(width.unwrap_or(1600), height.unwrap_or(1600));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let meshes = load_obj("extras/models/dalek_sec.obj", &mut materials, &mut textures, None);

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(objects);

    let environment = EnvironmentMap::new("extras/textures/studio_01.exr", 1.0);

    SceneBuilder::new("Dalek Sec")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Dalek Sec scene")
}