use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use primitive::{Primitive};
use lights::Light;
use materials::{Diffuse, Emissive, Plastic, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use scene::Scene;
use texture::{ImageTexture, SolidColor};
use transformations::TransformedMesh;
use triangle::TriangleMesh;

use mat;

pub fn cornell_box_bunny_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    // Same camera as the classic Cornell box so the framing looks identical.
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);
    let aperture = 0.0;
    let focus_distance = 10.0;

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             aperture, focus_distance);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let roughness = 0.0;
    let red_id = mat!(materials, Diffuse::new(SolidColor::new(0.65, 0.05, 0.05).into(), roughness));
    let green_id = mat!(materials, Diffuse::new(SolidColor::new(0.12, 0.45, 0.15).into(), roughness));
    let white_id = mat!(materials, Diffuse::new(SolidColor::new(0.73, 0.73, 0.73).into(), roughness));
    let light_material = mat!(materials, Emissive::new(SolidColor::new(25.0, 18.0, 10.0).into()));

    // add the walls of the cornell box to the world
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id).into_primitive());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id).into_primitive());
    objects.push(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let bunny_material = mat!(materials, Refractive::new(1.5, Vec3A::ONE));
    let bunny_mesh = Primitive::TriangleMesh(Arc::new(TriangleMesh::from("docs/models/bunny.obj", bunny_material)));
    objects.push(Primitive::TransformedMesh(Arc::new(TransformedMesh::new(Vec3A::new(224.0, -66.0, 278.0), Vec3A::new(0.0, 180.0, 0.0), 2000.0, bunny_mesh))));

    let buddha_material = mat!(materials, Reflective::new(Vec3A::new(0.95, 0.64, 0.54), 0.05));
    let buddha = TriangleMesh::from("docs/models/happy_buddha.obj", buddha_material).into();
    objects.push(TransformedMesh::new(Vec3A::new(150.0, -175.0, 450.0), Vec3A::new(0.0, 180.0, 0.0), 2600.0, buddha).into());

    let bvh = BVH::new(&mut objects, 0.0, 1.0);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);

    Scene::new(String::from("Cornell Box with Stanford Bunny"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3A::new(25.0, 18.0, 10.0))], None, false)
}