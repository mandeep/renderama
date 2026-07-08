use std::collections::HashMap;

use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddTexture, InsertInto, PushInto};
use crate::io::{LoadObjOptions, load_obj_with_options};
use crate::lights::Light;
use crate::materials::{Emissive, Material};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{ImageTexture, Texture};
use crate::transformations::TransformedMesh;

pub fn backrooms_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let camera_options = CameraOptions::from_disk("extras/models/backrooms/camera.json");
    let camera_options = camera_options
        .with_resolution(
            width.unwrap_or(camera_options.resolution.0 as usize),
            height.unwrap_or(camera_options.resolution.1 as usize),
    );

    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let mut material_overrides: HashMap<String, Material> = HashMap::new();
    let carpet_texture_id = textures.add_texture(ImageTexture::srgb("extras/models/backrooms/NewCarpet_baseColor.png", Vec2::splat(1.0)));
    let ceiling_tile_id = textures.add_texture(ImageTexture::srgb("extras/models/backrooms/CeilingTile_baseColor.png", Vec2::splat(1.0)));
    let vents_id = textures.add_texture(ImageTexture::srgb("extras/models/backrooms/Vent_baseColor.png", Vec2::splat(1.0)));
    material_overrides.insert_into("NewCarpet", Emissive::new(carpet_texture_id).with_intensity(10.0));
    material_overrides.insert_into("CeilingTile", Emissive::new(ceiling_tile_id).with_intensity(10.0));
    material_overrides.insert_into("Vent", Emissive::new(vents_id).with_intensity(10.0));

    let options = LoadObjOptions::new()
        .with_overrides(material_overrides);
    let meshes = load_obj_with_options("extras/models/backrooms/backrooms.obj", &mut materials, &mut textures, &mut lights, options);

    // need to scale down the scene since it's larger in the obj than in the blend file
    let (translation, rotation, scale) = (Vec3A::ZERO, Vec3A::ZERO, Vec3A::splat(0.85));
    for mesh in meshes {
        let transformed = TransformedMesh::new(translation, rotation, scale, mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Backrooms")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Backrooms scene")
}