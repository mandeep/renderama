use std::f32;
use std::f32::consts::PI;

use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::extensions::PushInto;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, ImageTexture, Texture};
use crate::transformations::TransformedMesh;

use crate::{mat, tex};

/// UV Checker images
/// https://subscription.packtpub.com/book/web-development/9781803233871/17/ch17lvl1sec79/custom-uv-modeling-in-blender
/// https://static.packt-cdn.com/products/9781803233871/graphics/image/Figure_13.35_B18726.jpg
///
/// Some other UV maps were tested from
/// https://www.pixelsham.com/2018/09/29/uv-maps/ and
/// https://uvchecker.atlux.one/
pub fn cornell_box_uv_scene(width: Option<usize>, height: Option<usize>) -> Scene {
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
    let mut textures: Vec<Texture> = Vec::new();

    let roughness = 0.0;
    let texture = tex!(textures, ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(1.0)));
    let texture_id = mat!(materials, Diffuse::new(roughness));
    let white = tex!(textures, Color::new(0.73, 0.73, 0.73));
    let white_id = mat!(materials, Diffuse::new(roughness));
    let light_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
    let light_material = mat!(materials, Emissive::new());

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id, texture).into_reversed());
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, texture_id, texture));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, light_material, light_id).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id, texture).into_reversed());
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, texture_id, texture));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id, texture).into_reversed());

    // bounce wall to reflect light
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, white_id, white));

    // add the boxes of the cornell box to the world
    let p0 = Vec3A::new(0.0, 0.0, 0.0);
    let p1 = Vec3A::new(165.0, 165.0, 165.0);
    let p2 = Vec3A::new(165.0, 330.0, 165.0);

    let small_scale = 165.0 / 555.0;
    let small_box_texture = tex!(textures, ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(small_scale)));
    let small_box_texture_id = mat!(materials, Diffuse::new(roughness));

    let large_scale_u = 165.0 / 555.0;
    let large_scale_v = 330.0 / 555.0;
    let large_scale = Vec2::new(large_scale_u, large_scale_v);
    let large_box_texture = tex!(textures, ImageTexture::new("extras/textures/uv_checker.jpg", large_scale));
    let large_box_texture_id = mat!(materials, Diffuse::new(roughness));

    objects.push_into(TransformedMesh::new(Vec3A::new(130.0, 0.0, 65.0), Vec3A::new(0.0, -18.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p1, small_box_texture_id, small_box_texture)));
    objects.push_into(TransformedMesh::new(Vec3A::new(265.0, 0.0, 295.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p2, large_box_texture_id, large_box_texture)));

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, light_material, light_id);
    let light_intensity = Vec3A::new(50.0, 50.0, 50.0);

    let fill_light_texture = tex!(textures, Color::new(0.2, 0.2, 0.2));
    let fill_light_material = mat!(materials, Emissive::new());
    let fill_light_shape = Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, fill_light_material, fill_light_texture);

    objects.push_into(fill_light_shape.clone());

    let lights = vec![Light::new(light_shape, light_intensity),];

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Cornell Box with UVs")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box with UVs scene")
}