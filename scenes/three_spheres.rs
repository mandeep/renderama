use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::materials::{Diffuse, Reflective, Refractive, Material, Plastic, Volumetric};
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};
use crate::volume::Volume;

use crate::{mat, tex};


pub fn three_spheres_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 3.0, 5.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.5);
    let f_stop = 5.6;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fstop(f_stop)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(1024));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let refr_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
    let refr_idx = mat!(materials, Refractive::new(refr_id, 1.5));
    let vol_id = tex!(textures, Color::new(0.0, 0.4, 0.9));
    let vol_idx = mat!(materials, Volumetric::new(vol_id));

    let boundary = Sphere::new(Vec3A::new(0.6, 0.0, -1.0), 0.5, refr_idx);
    let cloned_boundary = boundary.clone();

    objects.push_into(boundary);
    objects.push_into(Volume::new(4.0, cloned_boundary, vol_idx));

    let metal_id = tex!(textures, Color::new(0.93, 0.93, 0.93));
    let metal_idx = mat!(materials, Reflective::new(metal_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(-0.6, 0.0, -1.0), 0.5, metal_idx));

    let plastic_id = tex!(textures, Color::new(0.34, 0.57, 1.0));
    let plastic_idx = mat!(materials, Plastic::new(plastic_id, 0.10, 1.5));
    objects.push_into(Sphere::new(Vec3A::new(0.0, 0.0, -2.0), 0.5, plastic_idx));

    let floor_id = tex!(textures, Color::new(0.5, 0.5, 0.52));
    let floor_idx = mat!(materials, Diffuse::new(floor_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(0.0, -100.5, -1.0), 100.0, floor_idx));

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/dusk_1_puresky.exr", 1.0);

    SceneBuilder::new("Three Spheres")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Three Spheres scene")
}