use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use io;
use materials::{Material, Plastic};
use primitive::Primitive;
use scene::{Scene, SceneBuilder};
use texture::Color;
use transformations::TransformedMesh;


pub fn subway_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-0.25, 1.0, 4.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let sensor_height = 24.0;
    let old_fov = 22.0;
    let fov_radians = old_fov * std::f32::consts::PI / 180.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let old_aperture = 0.01;
    let world_scale = 0.001;
    let f_stop = (focal_length * world_scale) / old_aperture;
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
        (aspect_width, aspect_height)
    );

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let default_material = Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 0.025, 1.49).into();

    let (meshes, lights) = io::load_obj("extras/models/subway/subway.obj", &mut materials, None, default_material);

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

    let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0).into();

    SceneBuilder::new("Subway Train Interior")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_environment(environment)
        .with_lights(lights)
        .build()
        .expect("Failed to build Subway Train Interior scene")
}