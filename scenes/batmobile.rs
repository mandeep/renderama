use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::load_obj;
use crate::materials::{Material, Diffuse};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;

use crate::mat;
use crate::tex;


pub fn batmobile_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-2.0, 0.5, 1.5);
    let lookat = Vec3A::new(0.0, 0.0, 0.25);
    let focal_length = 74.0;
    let f_stop = 2.8;
    let focus_distance = 2.25;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_focal_length(focal_length)
        .with_fstop(f_stop)
        .with_focus_distance(focus_distance)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);


    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let (meshes, _) = load_obj("extras/models/batmobile.obj", &mut materials, &mut textures);

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
        objects.push_into(transformed);
    }

    let grey_id = tex!(textures, Color::new(0.05, 0.05, 0.07));
    let grey = mat!(materials, Diffuse::new(grey_id, 0.0));
    // floor plane
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(-1000.0..1000.0, -1000.0..1000.0), -0.4, grey));

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/pure_sky_05.exr", 1.0);

    SceneBuilder::new("Batmobile")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Batmobile scene")
}