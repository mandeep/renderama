use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddLight, AddMaterial, AddTexture, PushInto};
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material, Reflective, Refractive};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;
use crate::triangle::TriangleMesh;


pub fn cornell_box_bunny_scene(width: Option<usize>, height: Option<usize>) -> Scene {
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
    let light_id = textures.add_texture(Color::new(25.0, 18.0, 10.0));
    let red_id = materials.add_material(Diffuse::new(red, roughness));
    let green_id = materials.add_material(Diffuse::new(green, roughness));
    let white_id = materials.add_material(Diffuse::new(white, roughness));
    let light_material = materials.add_material(Emissive::new(light_id));

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let bunny_texture = textures.add_texture(Color::new(1.0, 1.0, 1.0));
    let bunny_material = materials.add_material(Refractive::new(bunny_texture, 1.5));
    let bunny_mesh = Primitive::TriangleMesh(Arc::new(TriangleMesh::from("extras/models/bunny.obj", bunny_material)));
    objects.push_into(Primitive::TransformedMesh(Arc::new(TransformedMesh::new(Vec3A::new(224.0, -66.0, 278.0), Vec3A::new(0.0, 180.0, 0.0), Vec3A::splat(2000.0), bunny_mesh))));

    let buddha_texture = textures.add_texture(Color::new(0.95, 0.64, 0.54));
    let buddha_material = materials.add_material(Reflective::new(buddha_texture, 0.05));
    let buddha = TriangleMesh::from("extras/models/happy_buddha.obj", buddha_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(150.0, -175.0, 450.0), Vec3A::new(0.0, 180.0, 0.0), Vec3A::splat(2600.0), buddha));

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);
    lights.add_light(Light::new(light_shape, Vec3A::new(25.0, 18.0, 10.0)));

    SceneBuilder::new("Cornell Box with Bunny and Buddha")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box Bunny and Buddha scene")
}