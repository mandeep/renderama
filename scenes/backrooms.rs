use std::collections::HashMap;
use std::f32;

use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::extensions::{InsertInto, PushInto};
use crate::io;
use crate::materials::{Material, Diffuse, Emissive};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, ImageTexture, IntensityTexture};
use crate::transformations::TransformedMesh;

pub fn backrooms_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(-3.0, 1.0, -6.0);
    let lookat = Vec3A::new(1.0, 1.0, 1.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let sensor_height = 24.0;
    let focal_length = 35.0;
    let world_scale = 0.001;
    let f_stop = 0.7;
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

    let mut material_overrides: HashMap<String, Material> = HashMap::new();
    material_overrides.insert_into("NewCarpet", Emissive::new(IntensityTexture::new(ImageTexture::new("extras/models/backrooms/NewCarpet_baseColor.png", Vec2::splat(1.0)), 10.0)));
    material_overrides.insert_into("CeilingTile", Emissive::new(IntensityTexture::new(ImageTexture::new("extras/models/backrooms/CeilingTile_baseColor.png", Vec2::splat(1.0)), 10.0)));

    let default_material = Diffuse::new(Color::new(0.9, 0.9, 0.9), 0.1);

    let (meshes, lights) = io::load_obj("extras/models/backrooms/backrooms.obj", &mut materials, Some(material_overrides), default_material);

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Backrooms")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(lights)
        .build()
        .expect("Failed to build Backrooms scene")
}