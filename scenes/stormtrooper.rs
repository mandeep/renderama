use std::f32;
use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::environment::EnvironmentMap;
use crate::io;
use crate::materials::{Material, Plastic};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::Color;
use crate::transformations::TransformedMesh;


pub fn stormtrooper_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.25, -0.5, 5.5);
    let lookat = Vec3A::new(0.0, 0.0, -2.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let old_fov = 22.0;
    let fov_radians = old_fov * PI / 180.0;
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let sensor_height = 24.0;
    let focus_distance = 4.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let world_scale = 0.001;
    let f_stop = 0.7;

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

    // an example of material overrides for the obj loader
    // Some(overrides) would be passed into the io::load_obj constructor
    //
    // let mut overrides: HashMap<String, Material> = HashMap::new();

    // overrides.insert(
    //     "white".to_string(), 
    //     Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 1.0, 1.49).into()
    // );

    // overrides.insert(
    //     "Material.001".to_string(), 
    //     Plastic::new(Color::new(0.01, 0.01, 0.01).into(), 1.0, 1.49).into()
    // );

    let default_material = Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 0.025, 1.49).into();

    let (meshes, _) = io::load_obj("extras/models/stormtrooper.obj", &mut materials, None, default_material);

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
        objects.push(transformed.into());
    }

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/car_studio_lighting.exr", 0.7).into();

    SceneBuilder::new("Stormtrooper")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_environment(environment)
        .build()
        .expect("Failed to build Stormtrooper scene")
}