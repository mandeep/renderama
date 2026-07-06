use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::PushInto;
use crate::io::{LoadObjOptions, load_obj_with_options};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder, SceneContext};
use crate::transformations::TransformedMesh;

pub fn ocean_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    // camera settings exported from blender
    let camera_options = CameraOptions::from_disk("extras/models/off_the_coast_camera.json");
    let camera_options = camera_options.with_resolution(
        width.unwrap_or(camera_options.resolution.0 as usize),
        height.unwrap_or(camera_options.resolution.1 as usize),
    );
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut context = SceneContext::new();

    let options = LoadObjOptions::new()
        .with_lights(true);
    let meshes = load_obj_with_options("extras/models/off_the_coast.obj", &mut context, options);

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Off The Coast")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_context(context)
        .build()
        .expect("Failed to build Off The Coast scene")
}