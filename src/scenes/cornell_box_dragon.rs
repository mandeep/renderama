use std::f32;
use std::f32::consts::PI;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Plastic, Material};
use plane::{Axis, Bounds2D, Plane};
use scene::{Scene, SceneBuilder};
use texture::Color;
use transformations::TransformedMesh;
use triangle::TriangleMesh;

use mat;

pub fn cornell_box_dragon_scene(width: Option<usize>, height: Option<usize>) -> Scene {
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

    let roughness = 0.0;
    let red_id = mat!(materials, Diffuse::new(Color::new(0.65, 0.05, 0.05).into(), roughness));
    let green_id = mat!(materials, Diffuse::new(Color::new(0.12, 0.45, 0.15).into(), roughness));
    let white_id = mat!(materials, Diffuse::new(Color::new(0.73, 0.73, 0.73).into(), roughness));
    let light_material = mat!(materials, Emissive::new(Color::new(20.0, 20.0, 20.0).into()));

    // add the walls of the cornell box to the world
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id).into_primitive());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id).into_primitive());
    objects.push(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let dragon_material = mat!(materials, Plastic::new(Color::new(0.0, 0.06, 0.18).into(), 0.15, 1.5));
    let dragon = TriangleMesh::from("extras/models/dragon.obj", dragon_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(283.0, 114.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), Vec3A::splat(425.0), dragon).into());

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);
    let light = vec![Light::new(light_shape.into(), Vec3A::new(25.0, 25.0, 25.0))];

    SceneBuilder::new("Cornell Box with Dragon")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(light)
        .build()
        .expect("Failed to build Cornell Box Dragon scene")
}