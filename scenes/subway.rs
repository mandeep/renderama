use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::{LoadObjOptions, load_obj_with_options};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder, SceneContext};
use crate::transformations::TransformedMesh;

pub fn subway_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-0.25, 1.0, 4.0);
    let lookat = Vec3A::new(0.0, 1.0, 0.0);
    let f_stop = 5.6;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fstop(f_stop)
        .with_focal_length(52.0)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut context = SceneContext::new();

    let obj_options = LoadObjOptions::new()
        .with_lights(true)
        .with_emissive_scale(10.0);
    let meshes = load_obj_with_options("extras/models/subway/subway.obj", &mut context, obj_options);

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
        .with_context(context)
        .with_environment(environment)
        .build()
        .expect("Failed to build Subway Train Interior scene")
}