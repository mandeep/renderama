use std::f32;
use std::f32::consts::PI;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
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
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let old_fov = 20.0;
    let (aspect_width, aspect_height) = (width.unwrap_or(2048) as f32, height.unwrap_or(1024) as f32);
    let sensor_height = 24.0;
    let fov_radians = old_fov * PI / 180.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let old_aperture = 0.01;
    let world_scale = 0.001;
    let f_stop = (focal_length * world_scale) / old_aperture;
    let focus_distance = (lookat - origin).length();

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