use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use io;
use materials::{Material, Diffuse, Plastic};
use plane::{Axis, Bounds2D, Plane};
use primitive::Primitive;
use scene::{Scene, SceneBuilder};
use texture::Color;
use transformations::TransformedMesh;

use mat;


pub fn batmobile_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-2.0, 0.5, 1.5);
    let lookat = Vec3A::new(0.0, 0.0, 0.25);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let sensor_height = 24.0;
    let focal_length = 74.0;
    let world_scale = 0.001;
    let f_stop = 2.8;
    let focus_distance = 2.25;

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

    let default_material = Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 0.01, 1.49).into();

    let (meshes, _) = io::load_obj("extras/models/batmobile.obj", &mut materials, None, default_material);

    let translation = Vec3A::new(0.0, 0.0, 0.0);
    let rotation = Vec3A::new(0.0, 0.0, 0.0);
    let scale = Vec3A::splat(0.25);

    for mesh in meshes {
        let transformed = TransformedMesh::new(
            translation, 
            rotation, 
            scale, 
            Primitive::TriangleMesh(Arc::new(mesh))
        );
        objects.push(transformed.into());
    }

    let grey = mat!(materials, Diffuse::new(Color::new(0.05, 0.05, 0.07).into(), 0.0));
    // floor plane
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(-1000.0..1000.0, -1000.0..1000.0), -0.4, grey).into_primitive());

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/pure_sky_05.exr", 1.0).into();

    SceneBuilder::new("Batmobile")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_environment(environment)
        .build()
        .expect("Failed to build Batmobile scene")
}