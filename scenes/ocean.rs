use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::PushInto;
use crate::io::load_obj;
use crate::lights::Light;
use crate::materials::Material;
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::Texture;
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
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let meshes = load_obj("extras/models/off_the_coast.obj", &mut materials, &mut textures, Some(&mut lights));

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Off The Coast")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Off The Coast scene")
}