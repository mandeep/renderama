use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::{LoadObjOptions, load_obj_with_options};
use crate::materials::Material;
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::Texture;
use crate::transformations::TransformedMesh;

pub fn subway_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-0.25, 1.0, 4.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let sensor_height = 24.0;
    let focal_length = 61.73; // 61.73 is the calculation from the old 22.0 fov prior to camera change
    let world_scale = 0.001;
    let f_stop = 8.0;
    let focus_distance = (lookat - origin).length();

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

    let obj_options = LoadObjOptions::new().with_emissive_scale(10.0);
    let (meshes, lights) = load_obj_with_options("extras/models/subway/subway.obj", &mut materials, &mut textures, obj_options);

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

    let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0);

    SceneBuilder::new("Subway Train Interior")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .with_lights(lights)
        .build()
        .expect("Failed to build Subway Train Interior scene")
}