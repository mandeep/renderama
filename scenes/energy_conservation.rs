use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddLight, AddMaterial, AddTexture, PushInto};
use crate::lights::{AreaLight, Light};
use crate::materials::{Emissive, Material, Reflective};
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};


pub fn energy_conservation_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -500.0);
    let lookat = Vec3A::new(278.0, 278.0, 300.0);
    let fov = 40.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(512));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let count = 10;
    let radius = 20.0;
    let spacing = 55.0;
    let start_x = 278.0 - ((count as f32 - 1.0) * spacing / 2.0);

    for i in 0..count {
        let roughness = i as f32 * 0.10;
        let x_pos = start_x + (i as f32 * spacing);

        let tex_id = textures.add_texture(Color::new(1.0, 1.0, 1.0));
        let mat_id = materials.add_material(Reflective::new(tex_id, roughness));
        objects.push_into(Sphere::new(Vec3A::new(x_pos, 278.0, 278.0), radius, mat_id));
    }

    let bvh = BVH::new(objects);

    let light_texture = textures.add_texture(Color::new(50.0, 50.0, 50.0));
    let light_material = materials.add_material(Emissive::new(light_texture));
    let light_plane = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 600.0, Orientation::Reversed, light_material);
    let light_id = textures.add_texture(Color::splat(50.0));
    lights.add_light(AreaLight::from(light_plane, light_id));

    SceneBuilder::new("Energy Conservation Test")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Energy Conservation Test scene")
}