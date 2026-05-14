use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Plastic, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use scene::Scene;
use texture::{Color, ImageTexture};
use transformations::TransformedMesh;
use triangle::TriangleMesh;

use mat;

pub fn cornell_box_dragon_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             0.0, 10.0);

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
    let dragon = TriangleMesh::from("docs/models/dragon.obj", dragon_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(283.0, 114.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), 425.0, dragon).into());

    let bvh = BVH::new(&mut objects, 0.0, 1.0);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);

    Scene::new(String::from("Cornell Box with Dragon"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3A::new(25.0, 25.0, 25.0))], None, None)
}