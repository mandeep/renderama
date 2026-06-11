use std::f32;
use std::f32::consts::PI;

use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::extensions::PushInto;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Plastic, Refractive, Material};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, ImageTexture};
use crate::transformations::TransformedMesh;
use crate::triangle::TriangleMesh;

use crate::mat;

pub fn cornell_box_object_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let (aspect_width, aspect_height) = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);
    let sensor_height = 24.0;
    let old_fov = 40.0;
    let fov_radians = old_fov * PI / 180.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let f_stop = std::f32::INFINITY;
    let focus_distance = (lookat - origin).length();
    let world_scale = 1.0;

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

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let roughness = 0.0;
    let red_id = mat!(materials, Diffuse::new(Color::new(0.65, 0.05, 0.05), roughness));
    let green_id = mat!(materials, Diffuse::new(Color::new(0.12, 0.45, 0.15), roughness));
    let white_id = mat!(materials, Diffuse::new(Color::new(0.73, 0.73, 0.73), roughness));
    let light_material = mat!(materials, Emissive::new(Color::new(25.0, 18.0, 10.0)));

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let lucy_material = mat!(materials, Diffuse::new(Color::new(0.92, 0.88, 0.82), 0.05));
    let lucy = TriangleMesh::from("extras/models/lucy.obj", lucy_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(200.0, 180.0, 364.0), Vec3A::new(0.0, 0.0, 0.0), Vec3A::splat(0.30), lucy));

    let dragon_material = mat!(materials, Plastic::new(Color::new(0.7, 0.85, 0.45), 0.05, 1.5));
    let dragon = TriangleMesh::from("extras/models/dragon.obj", dragon_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(283.0, 96.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), Vec3A::splat(350.0), dragon));

    let bunny_material = mat!(materials, Refractive::new(Color::new(1.0, 1.0, 1.0), 1.5));
    let bunny = TriangleMesh::from("extras/models/bunny.obj", bunny_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(110.0, -25.0, 140.0), Vec3A::new(0.0, 180.0, 0.0), Vec3A::splat(750.0), bunny));

    let buddha_texture = ImageTexture::new("extras/textures/buddha_relief_diffuse.jpeg", Vec2::splat(1.0));
    let buddha_material = mat!(materials, Diffuse::new(buddha_texture, 0.0));
    let buddha = TriangleMesh::from("extras/models/buddha_relief.obj", buddha_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(273.0, 180.0, 530.0), Vec3A::new(-90.0, 180.0, 0.0), Vec3A::splat(24.0), buddha));

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);
    let light_intensity = Vec3A::new(37.5, 27.0, 15.0);
    let light = vec![Light::new(light_shape, light_intensity)];

    SceneBuilder::new("Cornell Box with Multiple Objects")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(light)
        .build()
        .expect("Failed to build Cornell Box Objects scene")
}