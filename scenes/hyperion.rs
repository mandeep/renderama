use std::sync::Arc;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::load_obj;
use crate::materials::{Diffuse, Material, Plastic, Reflective, Refractive, Volumetric};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::primitive::Primitive;
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;
use crate::triangle::TriangleMesh;
use crate::volume::Volume;

use crate::{mat, tex};


pub fn hyperion_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 6.0, 6.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.5);
    let fov = 22.0;
    let f_stop = 6.17346477508544921875; // calculated from old aperture code

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_fstop(f_stop)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let orange_color = Color::new(1.0, 0.32, 0.0);
    let orange_bright_color = Color::new(1.0, 0.16, 0.0);
    let floor_id = tex!(textures, Color::new(0.63, 0.61, 0.59));
    let glass_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
    let metal_id = tex!(textures, Color::new(0.93, 0.93, 0.93));
    let dark_metal_id = tex!(textures, Color::new(0.757, 0.729, 0.694));
    let platform_id = tex!(textures, Color::new(0.76, 0.74, 0.72));
    let orange_id = tex!(textures, orange_color);
    let orange_rough_id = tex!(textures, orange_bright_color);
    let marble_vol_id = tex!(textures, Color::new(0.60, 0.71, 0.49));
    let pingpong_id = tex!(textures, Color::new(0.93, 0.89, 0.85));
    let white_id = tex!(textures, Color::new(1.0, 1.0, 1.0));

    let floor_idx = mat!(materials, Diffuse::new(floor_id, 0.0));
    let glass_idx = mat!(materials, Refractive::new(glass_id, 1.5));
    let metal_idx = mat!(materials, Reflective::new(metal_id, 0.2));
    let dark_metal_idx = mat!(materials, Reflective::new(dark_metal_id, 0.10));
    let platform_idx = mat!(materials, Diffuse::new(platform_id, 0.0));
    let orange_idx = mat!(materials, Plastic::new(orange_id, 0.20, 1.5));
    let orange_rough_idx = mat!(materials, Plastic::new(orange_rough_id, 0.25, 1.5).with_subsurface(0.80));
    let marble_vol_idx = mat!(materials, Volumetric::new(marble_vol_id));
    let pingpong_idx = mat!(materials, Plastic::new(pingpong_id, 0.60, 1.45).with_subsurface(0.40));
    let white_idx = mat!(materials, Plastic::new(white_id, 0.1, 1.45));

    let floor_plane = Plane::new(Axis::XZ, Bounds2D::new(-50.0..50.0, -50.0..50.0), 0.0, floor_idx);
    let platform = Rectangle::new(Vec3A::new(-3.5, 0.0, -4.0), Vec3A::new(3.5, 0.2, 0.5), platform_idx);
    let glass_sphere = Sphere::new(Vec3A::new(2.1, 0.60, -1.0), 0.4, glass_idx);
    let orange_sphere = Sphere::new(Vec3A::new(1.0, 0.54, -2.0), 0.35, orange_idx);
    let metal_sphere = Sphere::new(Vec3A::new(-2.25, 0.65, -1.2), 0.45, dark_metal_idx);
    let large_marble = Sphere::new(Vec3A::new(0.35, 0.325, 0.0), 0.125, glass_idx);
    let small_marble = Sphere::new(Vec3A::new(-0.35, 0.30, 0.0), 0.10, glass_idx);
    let large_marble_volume = Volume::new(15.0, large_marble.clone(), marble_vol_idx);
    let small_marble_volume = Volume::new(15.0, small_marble.clone(), marble_vol_idx);
    let orange_sphere_small = Sphere::new(Vec3A::new(-1.5, 0.35, -2.0), 0.15, orange_rough_idx);

    let ring_mesh = Arc::new(TriangleMesh::from("extras/models/ring.obj", metal_idx));
    let ring_left = TransformedMesh::new(Vec3A::new(-0.5, 0.20, -1.25), Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));
    let ring_center = TransformedMesh::new(Vec3A::new(0.20, 0.20, -0.80), Vec3A::new(0.0, 45.0, 0.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));
    let ring_right = TransformedMesh::new(Vec3A::new(0.375, 0.25, -1.0), Vec3A::new(-15.0, 0.0, -10.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));

    let (translation, rotation, scale) = (Vec3A::new(-0.6, 0.65, -2.5), Vec3A::new(-30.0, 0.0, 15.0), Vec3A::splat(0.25));
    let (meshes, _) = load_obj("extras/models/pokeball.obj", &mut materials, &mut textures);
    for mesh in meshes {
        let transformed_mesh = TransformedMesh::new(translation, rotation, scale, mesh);
        objects.push_into(transformed_mesh);
    }

    let pingpong_mesh = TriangleMesh::from("extras/models/pingpong.obj", pingpong_idx);
    let pingpong_sphere = TransformedMesh::new(Vec3A::new(-1.25, 0.475, -0.3), Vec3A::new(0.0, 0.0, 90.0), Vec3A::splat(0.28), pingpong_mesh);

    let golf_ball_mesh = TriangleMesh::from("extras/models/golf_ball.obj", white_idx);
    let golf_ball = TransformedMesh::new(Vec3A::new(1.70, 0.67, -0.35), Vec3A::new(-30.0, 0.0, 15.0), Vec3A::splat(0.23), golf_ball_mesh);

    objects.push_into(floor_plane);
    objects.push_into(platform);
    objects.push_into(glass_sphere);
    objects.push_into(metal_sphere);
    objects.push_into(orange_sphere);
    objects.push_into(large_marble);
    objects.push_into(small_marble);
    objects.push_into(large_marble_volume);
    objects.push_into(small_marble_volume);
    objects.push_into(ring_left);
    objects.push_into(ring_center);
    objects.push_into(ring_right);
    objects.push_into(pingpong_sphere);
    objects.push_into(golf_ball);
    objects.push_into(orange_sphere_small);

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/white_studio_03.exr", 0.6);

    SceneBuilder::new("Hyperion")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_environment(environment)
        .build()
        .expect("Failed to build Hyperion scene")
}