use std::collections::HashMap;
use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use io;
use materials::{Material, Diffuse, Plastic};
use primitive::Primitive;
use scene::{Scene, SceneBuilder};
use texture::Color;
use transformations::TransformedMesh;


pub fn stormtrooper_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.25, -0.5, 5.5);
    let lookat = Vec3A::new(0.0, 0.0, -2.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 22.0;
    let aspect_ratio = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let mut overrides: HashMap<String, Material> = HashMap::new();
    
    overrides.insert(
        "white".to_string(), 
        Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 0.025, 1.49).into()
    );

    overrides.insert(
        "Material.001".to_string(), 
        Diffuse::new(Color::new(0.01, 0.01, 0.01).into(), 0.0).into()
    );

    let default_material = Plastic::new(Color::new(0.9, 0.9, 0.9).into(), 0.01, 1.49).into();

    let meshes = io::load_obj("extras/models/stormtrooper.obj", &mut materials, &overrides, default_material);

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