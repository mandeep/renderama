use std::sync::Arc;

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

pub fn stormtrooper_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.25, -0.5, 5.5);
    let lookat = Vec3A::new(0.0, 0.0, -2.0);
    let fov = 22.0;
    let focus_distance = 4.0;
    let f_stop = 0.7;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_focus_distance(focus_distance)
        .with_fstop(f_stop)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    // an example of material overrides for the obj loader
    // Some(overrides) would be passed into the io::load_obj constructor
    //
    // let mut overrides: HashMap<String, Material> = HashMap::new();

    // overrides.insert(
    //     "white".to_string(), 
    //     Plastic::new(Color::new(0.9, 0.9, 0.9), 1.0, 1.49)
    // );

    // overrides.insert(
    //     "Material.001".to_string(), 
    //     Plastic::new(Color::new(0.01, 0.01, 0.01), 1.0, 1.49)
    // );

    let (meshes, _) = load_obj("extras/models/stormtrooper.obj", &mut materials, &mut textures);

    let translation = Vec3A::new(0.0, 0.0, 0.0);
    let rotation = Vec3A::new(0.0, 0.0, 0.0);
    let scale = Vec3A::splat(1.0);

    for mesh in meshes {
        let transformed = TransformedMesh::new(
            translation, 
            rotation, 
            scale, 
            Primitive::TriangleMesh(Arc::new(mesh))
        );
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/car_studio_lighting.exr", 0.7);

    SceneBuilder::new("Stormtrooper")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Stormtrooper scene")
}