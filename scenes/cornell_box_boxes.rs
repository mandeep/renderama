use std::f32;
use std::f32::consts::PI;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::extensions::PushInto;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;

use crate::mat;
use crate::tex;

pub fn cornell_box_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);
    let sensor_height = 24.0;
    let old_fov = 40.0;
    let fov_radians = old_fov * PI / 180.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let f_stop = std::f32::INFINITY;
    let focus_distance = (lookat - origin).length();
    let world_scale = 1.0;

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

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let roughness = 0.0;
    let red = tex!(textures, Color::new(0.65, 0.05, 0.05));
    let green = tex!(textures, Color::new(0.12, 0.45, 0.15));
    let white = tex!(textures, Color::new(0.73, 0.73, 0.73));
    let light_id = tex!(textures, Color::new(25.0, 18.0, 10.0));
    let red_id = mat!(materials, Diffuse::new(roughness));
    let green_id = mat!(materials, Diffuse::new(roughness));
    let white_id = mat!(materials, Diffuse::new(roughness));
    let light_material = mat!(materials, Emissive::new());

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id, red).into_reversed());
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id, green));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material, light_id).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id, white).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id, white));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id, white).into_reversed());

    // add the boxes of the cornell box to the world
    let p0 = Vec3A::new(0.0, 0.0, 0.0);
    let p1 = Vec3A::new(165.0, 165.0, 165.0);
    let p2 = Vec3A::new(165.0, 330.0, 165.0);

    objects.push_into(TransformedMesh::new(Vec3A::new(130.0, 0.0, 65.0), Vec3A::new(0.0, -18.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p1, white_id, white)));
    objects.push_into(TransformedMesh::new(Vec3A::new(265.0, 0.0, 295.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p2, white_id, white)));

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id, white);
    let lights = vec![Light::new(light_shape, Vec3A::new(25.0, 18.0, 10.0))];

    SceneBuilder::new("Cornell Box with Boxes")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box scene")
}