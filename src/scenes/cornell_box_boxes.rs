use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Material};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use texture::{Color};
use transformations::TransformedMesh;

use mat;

pub fn cornell_box_scene(width: Option<usize>, height: Option<usize>) -> Scene {
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
    let light_material = mat!(materials, Emissive::new(Color::new(25.0, 18.0, 10.0).into()));

    // add the walls of the cornell box to the world
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id).into_primitive());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id).into_primitive());
    objects.push(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    // add the boxes of the cornell box to the world
    let p0 = Vec3A::new(0.0, 0.0, 0.0);
    let p1 = Vec3A::new(165.0, 165.0, 165.0);
    let p2 = Vec3A::new(165.0, 330.0, 165.0);

    objects.push(TransformedMesh::new(Vec3A::new(130.0, 0.0, 65.0), Vec3A::new(0.0, -18.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p1, white_id).into()).into());
    objects.push(TransformedMesh::new(Vec3A::new(265.0, 0.0, 295.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p2, white_id).into()).into());

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);

    Scene::new(String::from("Cornell Box"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3A::new(25.0, 18.0, 10.0))], None, None)
}