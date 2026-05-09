use std::f32;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use geometry::{Geometry};
use lights::Light;
use materials::{Diffuse, Emissive, Isotropic, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::{SolidColor, EnvironmentMap, ImageTexture};
use transformations::TransformedMesh;
use volume::Volume;
use world::World;
use mat;


pub fn three_spheres_scene(width: usize, height: usize) -> Scene {
    let origin = Vec3::new(0.0, 3.0, 6.0);
    let lookat = Vec3::new(0.0, 0.0, -1.5);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();
    let time0 = 0.0;
    let time1 = 1.0;

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));


    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.6, 0.0, -1.0),
                          Vec3::new(0.6, 0.0, -1.0),
                          0.5,
                          red_idx,
                          0.0,
                          1.0)));

    let metal_idx = mat!(materials, Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.0));

    world.add(Geometry::Sphere(Sphere::new(Vec3::new(-0.6, 0.0, -1.0),
                          Vec3::new(-0.6, 0.0, -1.0),
                          0.5,
                          metal_idx,
                          0.0,
                          1.0)));

    let glass_idx = mat!(materials, Reflective::new(Vec3::new(0.75, 0.75, 0.75), 0.1));

    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.0, 0.1, -2.0),
                          Vec3::new(0.0, 0.1, -2.0),
                          0.5,
                          glass_idx,
                          0.0,
                          1.0)));

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));
    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.0, -100.5, -1.0),
                          Vec3::new(0.0, -100.5, -1.0),
                          100.0,
                          floor_idx,
                          0.0,
                          1.0)));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let environment = EnvironmentMap::new("docs/textures/pure_sky_qwantani.exr").into();
    Scene::new(String::from("Three Spheres"), bvh, materials, camera, vec![], Some(environment), false)
}
