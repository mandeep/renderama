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

pub fn cornell_box_object_scene(width: Option<usize>, height: Option<usize>) -> Scene {
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

    let lucy_material = mat!(materials, Diffuse::new(Color::new(0.92, 0.88, 0.82).into(), 0.05));
    let lucy = TriangleMesh::from("docs/models/lucy.obj", lucy_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(200.0, 180.0, 364.0), Vec3A::new(0.0, 0.0, 0.0), Vec3A::splat(0.30), lucy).into());

    let dragon_material = mat!(materials, Plastic::new(Color::new(0.7, 0.85, 0.45).into(), 0.05, 1.5));
    let dragon = TriangleMesh::from("docs/models/dragon.obj", dragon_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(283.0, 96.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), Vec3A::splat(350.0), dragon).into());

    let bunny_material = mat!(materials, Refractive::new(Color::new(1.0, 1.0, 1.0).into(), 1.5));
    let bunny = TriangleMesh::from("docs/models/bunny.obj", bunny_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(110.0, -25.0, 140.0), Vec3A::new(0.0, 180.0, 0.0), Vec3A::splat(750.0), bunny).into());

    let buddha_texture = ImageTexture::new("docs/textures/buddha_relief_diffuse.jpeg", 1.0).into();
    let buddha_material = mat!(materials, Diffuse::new(buddha_texture, 0.0));
    let buddha = TriangleMesh::from("docs/models/buddha_relief.obj", buddha_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(273.0, 180.0, 530.0), Vec3A::new(-90.0, 180.0, 0.0), Vec3A::splat(24.0), buddha).into());

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);

    Scene::new(String::from("Cornell Box with Multiple Objects"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3A::new(25.0, 18.0, 10.0))], None, None)
}