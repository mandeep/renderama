use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddLight, AddMaterial, AddTexture, PushInto};
use crate::lights::{AreaLight, Light};
use crate::materials::{Diffuse, Emissive, Material, Plastic};
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;
use crate::triangle::TriangleMesh;


pub fn cornell_box_dragon_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let fov = 40.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(2048));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let roughness = 0.0;
    let red = textures.add_texture(Color::new(0.65, 0.05, 0.05));
    let green = textures.add_texture(Color::new(0.12, 0.45, 0.15));
    let white = textures.add_texture(Color::new(0.73, 0.73, 0.73));
    let light_id = textures.add_texture(Color::new(20.0, 20.0, 20.0));
    let red_id = materials.add_material(Diffuse::new(red, roughness));
    let green_id = materials.add_material(Diffuse::new(green, roughness));
    let white_id = materials.add_material(Diffuse::new(white, roughness));
    let light_material = materials.add_material(Emissive::new(light_id));

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed, red_id));
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, Orientation::Forward, green_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, Orientation::Reversed, light_material));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed, white_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, Orientation::Forward, white_id));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed,white_id));


    let dragon_texture = textures.add_texture(Color::new(0.0, 0.06, 0.18));
    let dragon_material = materials.add_material(Plastic::new(dragon_texture, 0.15, 1.5));
    let dragon = TriangleMesh::from("extras/models/dragon.obj", dragon_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(283.0, 114.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), Vec3A::splat(425.0), dragon));

    let bvh = BVH::new(objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, Orientation::Reversed, white_id);
    let light_intensity_id = textures.add_texture(Color::splat(25.0));
    lights.add_light(AreaLight::from(light_shape, light_intensity_id));

    SceneBuilder::new("Cornell Box with Dragon")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box Dragon scene")
}