use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::PushInto;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Plastic, Refractive};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::scene::{Scene, SceneBuilder, SceneContext};
use crate::texture::{Color, ImageTexture};
use crate::transformations::TransformedMesh;
use crate::triangle::TriangleMesh;


pub fn cornell_box_object_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(278.0, 278.0, -800.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let fov = 40.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(2048));
    let camera = Camera::new(&camera_options);

    let mut objects = Vec::new();
    let mut context = SceneContext::new();

    let roughness = 0.0;
    let red = context.add_texture(Color::new(0.65, 0.05, 0.05));
    let green = context.add_texture(Color::new(0.12, 0.45, 0.15));
    let white = context.add_texture(Color::new(0.73, 0.73, 0.73));
    let light_id = context.add_texture(Color::new(25.0, 18.0, 10.0));
    let red_id = context.add_material(Diffuse::new(red, roughness));
    let green_id = context.add_material(Diffuse::new(green, roughness));
    let white_id = context.add_material(Diffuse::new(white, roughness));
    let light_material = context.add_material(Emissive::new(light_id));

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, red_id).into_reversed());
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, green_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light_material).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, white_id));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, white_id).into_reversed());

    let lucy_texture = context.add_texture(Color::new(0.92, 0.88, 0.82));
    let lucy_material = context.add_material(Diffuse::new(lucy_texture, 0.05));
    let lucy = TriangleMesh::from("extras/models/lucy.obj", lucy_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(200.0, 180.0, 364.0), Vec3A::new(0.0, 0.0, 0.0), Vec3A::splat(0.30), lucy));

    let dragon_texture = context.add_texture(Color::new(0.7, 0.85, 0.45));
    let dragon_material = context.add_material(Plastic::new(dragon_texture, 0.05, 1.5));
    let dragon = TriangleMesh::from("extras/models/dragon.obj", dragon_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(283.0, 96.0, 268.0), Vec3A::new(0.0, -60.0, 0.0), Vec3A::splat(350.0), dragon));

    let bunny_texture = context.add_texture(Color::new(1.0, 1.0, 1.0));
    let bunny_material = context.add_material(Refractive::new(bunny_texture, 1.5));
    let bunny = TriangleMesh::from("extras/models/bunny.obj", bunny_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(110.0, -25.0, 140.0), Vec3A::new(0.0, 180.0, 0.0), Vec3A::splat(750.0), bunny));

    let buddha_texture = context.add_texture(ImageTexture::new("extras/textures/buddha_relief_diffuse.jpeg", Vec2::splat(1.0)));
    let buddha_material = context.add_material(Diffuse::new(buddha_texture, 0.0));
    let buddha = TriangleMesh::from("extras/models/buddha_relief.obj", buddha_material);
    objects.push_into(TransformedMesh::new(Vec3A::new(273.0, 180.0, 530.0), Vec3A::new(-90.0, 180.0, 0.0), Vec3A::splat(24.0), buddha));

    let bvh = BVH::new(&mut objects);

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, white_id);
    let light_intensity = Vec3A::new(37.5, 27.0, 15.0);
    context.add_light(Light::new(light_shape, light_intensity));

    SceneBuilder::new("Cornell Box with Multiple Objects")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_context(context)
        .build()
        .expect("Failed to build Cornell Box Objects scene")
}