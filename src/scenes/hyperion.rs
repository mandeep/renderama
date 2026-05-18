use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use materials::{Diffuse, Material, Plastic, Reflective, Refractive, Volumetric};
use plane::{Axis, Bounds2D, Plane};
use primitive::Primitive;
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::Color;
use transformations::TransformedMesh;
use triangle::TriangleMesh;
use volume::Volume;

use mat;


pub fn hyperion_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 6.0, 6.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 29.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(1024) as f32);
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let orange_color = Color::new(1.0, 0.32, 0.0);
    let floor_idx = mat!(materials, Diffuse::new(Color::new(0.5, 0.5, 0.5).into(), 0.0));
    let glass_idx = mat!(materials, Refractive::new(1.5, Vec3A::ONE.into()));
    let metal_idx = mat!(materials, Reflective::new(Vec3A::new(0.93, 0.93, 0.93), 0.12));
    let box_idx = mat!(materials, Diffuse::new(Color::new(0.75, 0.75, 0.75).into(), 0.0));
    // let vol_idx = mat!(materials, Volumetric::new(orange_color.into()));
    let orange_idx = mat!(materials, Plastic::new(orange_color.into(), 0.25, 1.45));
    let marble_idx = mat!(materials, Refractive::new(1.5, Vec3A::new(0.25, 0.48, 0.29)));

    let floor_plane = Plane::new(Axis::XZ, Bounds2D::new(-50.0..50.0, -50.0..50.0), 0.0, floor_idx);
    let layout_box = Rectangle::new(Vec3A::new(-3.5, 0.0, -4.0), Vec3A::new(3.5, 0.2, 1.0), box_idx);
    let glass_sphere = Sphere::new(Vec3A::new(2.0, 0.60, -1.0), 0.4, glass_idx);
    // let orange_sphere = Sphere::new(Vec3A::new(1.0, 0.56, -2.0), 0.4, glass_idx);
    let orange_sphere = Sphere::new(Vec3A::new(1.0, 0.55, -2.0), 0.35, orange_idx);
    let metal_sphere = Sphere::new(Vec3A::new(-2.0, 0.65, -1.0), 0.45, metal_idx);
    // let cloned_sphere = orange_sphere.clone();
    // let sphere_volume = Volume::new(3.0, cloned_sphere.into(), vol_idx);
    let large_marble = Sphere::new(Vec3A::new(0.35, 0.35, 0.0), 0.15, marble_idx);
    let small_marble = Sphere::new(Vec3A::new(-0.35, 0.30, 0.0), 0.10, marble_idx);

    let ring_mesh = TriangleMesh::from("docs/models/ring.obj", metal_idx).into();
    let ring_transformed = TransformedMesh::new(Vec3A::new(-0.5, 0.20, -1.25), Vec3A::new(0.0, 0.0, 0.0), 0.15, ring_mesh);

    objects.push(floor_plane.into());
    objects.push(layout_box.into());
    objects.push(glass_sphere.into());
    objects.push(metal_sphere.into());
    objects.push(orange_sphere.into());
    // objects.push(sphere_volume.into());
    objects.push(large_marble.into());
    objects.push(small_marble.into());
    objects.push(ring_transformed.into());

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("docs/textures/white_studio_03.exr").into();

    Scene::new(String::from("Hyperion"), bvh, materials, camera, vec![], Some(environment), None)
}