use std::f32;

use glam::{Vec2, Vec3A};

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Material};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::{Scene, SceneBuilder};
use texture::{Color, ImageTexture};
use transformations::TransformedMesh;

use mat;

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
    let fov = 40.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             0.0, 10.0);

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let roughness = 0.0;
    let texture_id = mat!(materials, Diffuse::new(ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(1.0)).into(), roughness));
    let white_id = mat!(materials, Diffuse::new(Color::new(0.73, 0.73, 0.73).into(), roughness));
    let light_material = mat!(materials, Emissive::new(Color::new(1.0, 1.0, 1.0).into()));

    // add the walls of the cornell box to the world
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id).into_reversed());
    objects.push(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, texture_id).into_primitive());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, light_material).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id).into_reversed());
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, texture_id).into_primitive());
    objects.push(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, texture_id).into_reversed());

    // bounce wall to reflect light
    objects.push(Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, white_id).into_primitive());

    // add the boxes of the cornell box to the world
    let p0 = Vec3A::new(0.0, 0.0, 0.0);
    let p1 = Vec3A::new(165.0, 165.0, 165.0);
    let p2 = Vec3A::new(165.0, 330.0, 165.0);

    let small_scale = 165.0 / 555.0;
    let small_box_texture_id = mat!(materials, Diffuse::new(ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(small_scale)).into(), roughness));

    let large_scale_u = 165.0 / 555.0;
    let large_scale_v = 330.0 / 555.0;
    let large_scale = Vec2::new(large_scale_u, large_scale_v);
    let large_box_texture_id = mat!(materials, Diffuse::new(ImageTexture::new("extras/textures/uv_checker.jpg", large_scale).into(), roughness));

    objects.push(TransformedMesh::new(Vec3A::new(130.0, 0.0, 65.0), Vec3A::new(0.0, -18.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p1, small_box_texture_id).into()).into());
    objects.push(TransformedMesh::new(Vec3A::new(265.0, 0.0, 295.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p2, large_box_texture_id).into()).into());

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, light_material);
    let light_intensity = Vec3A::new(50.0, 50.0, 50.0);

    let fill_light_material = mat!(materials, Emissive::new(Color::new(0.2, 0.2, 0.2).into()));
    let fill_light_shape = Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, fill_light_material);

    objects.push(fill_light_shape.clone().into_primitive());

    let lights = vec![Light::new(light_shape.into(), light_intensity),];

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Cornell Box with UVs")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box with UVs scene")
}