use std::f32;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use geometry::{Geometry};
use lights::Light;
use materials::{Diffuse, Emissive, Plastic, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use scene::Scene;
use texture::{ImageTexture, SolidColor};
use transformations::TransformedMesh;
use triangle::TriangleMesh;
use world::World;
use mat;

pub fn cornell_box_bunny_scene(width: usize, height: usize) -> Scene {
    // Same camera as the classic Cornell box so the framing looks identical.
    let origin = Vec3::new(278.0, 278.0, -800.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             aperture, focus_distance);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let roughness = 0.0;
    let red_id = mat!(materials, Diffuse::new(SolidColor::new(0.65, 0.05, 0.05).into(), roughness));
    let green_id = mat!(materials, Diffuse::new(SolidColor::new(0.12, 0.45, 0.15).into(), roughness));
    let white_id = mat!(materials, Diffuse::new(SolidColor::new(0.73, 0.73, 0.73).into(), roughness));
    let light_material = mat!(materials, Emissive::new(SolidColor::new(25.0, 18.0, 10.0).into()));

    // add the walls of the cornell box to the world
    world.add(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    world.add(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id).into_geometry());
    world.add(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    world.add(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    world.add(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id).into_geometry());
    world.add(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let bunny_material = mat!(materials, Refractive::new(1.5, Vec3::ONE));
    let bunny_mesh = Geometry::TriangleMesh(Box::new(TriangleMesh::from("docs/models/bunny.obj", bunny_material)));
    world.add(Geometry::TransformedMesh(Box::new(TransformedMesh::new(Vec3::new(224.0, -66.0, 278.0), Vec3::new(0.0, 180.0, 0.0), 2000.0, bunny_mesh))));

    let buddha_material = mat!(materials, Reflective::new(Vec3::new(0.95, 0.64, 0.54), 0.05));
    let buddha = TriangleMesh::from("docs/models/happy_buddha.obj", buddha_material).into();
    world.add(TransformedMesh::new(Vec3::new(150.0, -175.0, 450.0), Vec3::new(0.0, 180.0, 0.0), 2600.0, buddha).into());

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);

    Scene::new(String::from("Cornell Box with Stanford Bunny"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3::new(25.0, 18.0, 10.0))], None, false)
}