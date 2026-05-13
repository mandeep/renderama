use std::f32;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Material, Reflective};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::{SolidColor};
use transformations::TransformedMesh;
use world::World;
use mat;

pub fn white_furnace_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -500.0); 
    let lookat = Vec3A::new(278.0, 278.0, 300.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio, 0.0, 10.0);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let count = 10;
    let radius = 20.0;
    let spacing = 55.0;
    let start_x = 278.0 - ((count as f32 - 1.0) * spacing / 2.0);

    for i in 0..count {
        let roughness = i as f32 * 0.10;
        let x_pos = start_x + (i as f32 * spacing);
        
        let mat_id = mat!(materials, Reflective::new(Vec3A::ONE, roughness));
        
        world.add(Sphere::new(Vec3A::new(x_pos, 278.0, 278.0), radius, mat_id).into());
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    Scene::new(String::from("White Furnace Test"), bvh, materials, camera, vec![], None, true)
}