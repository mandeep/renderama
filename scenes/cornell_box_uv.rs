use glam::{Vec2, Vec3A};

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddLight, AddMaterial, AddTexture, PushInto};
use crate::lights::{AreaLight, Light};
use crate::materials::{Diffuse, Emissive, Material};
use crate::plane::{Axis, Bounds2D, Orientation, Plane};
use crate::primitive::Primitive;
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::{Color, ImageTexture, Texture};
use crate::transformations::TransformedMesh;


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
    let fov = 40.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(2048));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let roughness = 0.0;
    let texture = textures.add_texture(ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(1.0)));
    let texture_id = materials.add_material(Diffuse::new(texture, roughness));
    let white = textures.add_texture(Color::new(0.73, 0.73, 0.73));
    let white_id = materials.add_material(Diffuse::new(white, roughness));
    let light_id = textures.add_texture(Color::new(1.0, 1.0, 1.0));
    let light_material = materials.add_material(Emissive::new(light_id));

    // add the walls of the cornell box to the world
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed, texture_id));
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, Orientation::Forward, texture_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, Orientation::Reversed, light_material));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed, texture_id));
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(0.0..555.0, 0.0..555.0), 0.0, Orientation::Forward, texture_id));
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(0.0..555.0, 0.0..555.0), 555.0, Orientation::Reversed, texture_id));

    // bounce wall to reflect light
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, Orientation::Forward, white_id));

    // add the boxes of the cornell box to the world
    let p0 = Vec3A::new(0.0, 0.0, 0.0);
    let p1 = Vec3A::new(165.0, 165.0, 165.0);
    let p2 = Vec3A::new(165.0, 330.0, 165.0);

    let small_scale = 165.0 / 555.0;
    let small_box_texture = textures.add_texture(ImageTexture::new("extras/textures/uv_checker.jpg", Vec2::splat(small_scale)));
    let small_box_texture_id = materials.add_material(Diffuse::new(small_box_texture, roughness));

    let large_scale_u = 165.0 / 555.0;
    let large_scale_v = 330.0 / 555.0;
    let large_scale = Vec2::new(large_scale_u, large_scale_v);
    let large_box_texture = textures.add_texture(ImageTexture::new("extras/textures/uv_checker.jpg", large_scale));
    let large_box_texture_id = materials.add_material(Diffuse::new(large_box_texture, roughness));

    objects.push_into(TransformedMesh::new(Vec3A::new(130.0, 0.0, 65.0), Vec3A::new(0.0, -18.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p1, small_box_texture_id)));
    objects.push_into(TransformedMesh::new(Vec3A::new(265.0, 0.0, 295.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::splat(1.0), Rectangle::new(p0, p2, large_box_texture_id)));

    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.98, Orientation::Reversed, light_material);
    let light_intensity = Color::new(50.0, 50.0, 50.0);
    let light_intensity_id = textures.add_texture(light_intensity);

    let fill_light_texture = textures.add_texture(Color::new(0.2, 0.2, 0.2));
    let fill_light_material = materials.add_material(Emissive::new(fill_light_texture));
    let fill_light_shape = Plane::new(Axis::XY, Bounds2D::new(-1000.0..1555.0, -1000.0..1555.0), -805.0, Orientation::Forward, fill_light_material);

    objects.push_into(fill_light_shape.clone());

    lights.add_light(AreaLight::from(light_shape, light_intensity_id));

    let bvh = BVH::new(&mut objects);

    SceneBuilder::new("Cornell Box with UVs")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Cornell Box with UVs scene")
}