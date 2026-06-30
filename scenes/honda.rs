use std::collections::HashMap;
use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::{InsertInto, PushInto};
use crate::io::{LoadObjOptions, load_obj_with_options};
use crate::materials::{Material, Diffuse, Plastic};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;

use crate::mat;
use crate::tex;


pub fn honda_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(2.5, 0.75, 4.0);
    let lookat = Vec3A::new(-0.1, 0.225, 0.0);
    let focal_length = 135.0;
    let f_stop = 16.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_focal_length(focal_length)
        .with_fstop(f_stop)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let mut material_overrides: HashMap<String, Material> = HashMap::new();
    let car_paint_id = tex!(textures, Color::new(0.568452, 0.0, 0.0));
    let car_paint_material = Plastic::new(car_paint_id, 0.04, 1.5).with_clearcoat(0.6, 0.025);
    material_overrides.insert_into("EXT_paint", car_paint_material);
    let obj_options = LoadObjOptions::new()
        .with_overrides(Some(material_overrides));

    let (meshes, _) = load_obj_with_options("extras/models/honda/honda.obj", &mut materials, &mut textures, obj_options);

    let translation = Vec3A::new(0.0, 0.0, 0.0);
    let rotation = Vec3A::new(0.0, 0.0, 0.0);
    let scale = Vec3A::splat(0.5);

    for mesh in meshes {
        let transformed = TransformedMesh::new(
            translation, 
            rotation, 
            scale, 
            Primitive::TriangleMesh(Arc::new(mesh))
        );
        objects.push_into(transformed);
    }

    let grey_id = tex!(textures, Color::new(0.05, 0.05, 0.05));
    let grey = mat!(materials, Diffuse::new(grey_id, 0.0));
    // floor plane
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(-1000.0..1000.0, -1000.0..1000.0), 0.0, grey));

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0);

    SceneBuilder::new("Honda S800")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Honda S800 scene")
}