use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::{AddLight, AddMaterial, AddTexture, PushInto};
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material, Reflective};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::primitive::Primitive;
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, Texture};
use crate::transformations::TransformedMesh;




pub fn veach_mis_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 1.2, -3.5);
    let lookat = Vec3A::new(0.0, 1.9, 7.0);
    let focal_length = 32.0;
    let focus_distance = 10.0;
    let world_scale = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_focal_length(focal_length)
        .with_focus_distance(focus_distance)
        .with_world_scale(world_scale)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    let grey_id = textures.add_texture(Color::new(0.99, 0.99, 0.99));
    let grey = materials.add_material(Diffuse::new(grey_id, 0.0));

    // floor
    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(-20.0..20.0, -5.0..25.0), 0.0, grey));

    // back wall
    objects.push_into(Plane::new(Axis::XY, Bounds2D::new(-20.0..20.0, 0.0..15.0), 12.0, grey).into_reversed());

    // side walls, not sure if they do anything in this scene
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0),-20.0, grey));
    objects.push_into(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0), 20.0, grey).into_reversed());

    let silver = textures.add_texture(Color::new(0.75, 0.75, 0.75));

    // use a cursor to place planes edge to edge
    let mut cursor = Vec3A::new(0.0, 0.15, 2.0);
    let plate_length = 1.0;
    let plate_gap = 0.10;
    let visual_length = plate_length - plate_gap;

    let plate_configs: [(f32, f32); 5] = [
        (-20.0, 0.28),
        (-26.0, 0.20),
        (-32.0, 0.14),
        (-40.0, 0.10),
        (-52.0, 0.00),
    ];

    for (tilt_angle, fuzz) in plate_configs {
        let tilt_radians = tilt_angle.to_radians();

        let direction = Vec3A::new(0.0, -tilt_radians.sin(), tilt_radians.cos());

        let center_pos = cursor + (direction * (plate_length * 0.5));

        let mat_id = materials.add_material(Reflective::new(silver, fuzz));
        let rot = Vec3A::new(tilt_angle, 0.0, 0.0);
        let base = Rectangle::new(
            Vec3A::new(-2.25, 0.1, -visual_length / 2.0),
            Vec3A::new(2.25, 0.125, visual_length / 2.0), mat_id
        );

        objects.push_into(TransformedMesh::new(center_pos, rot, Vec3A::ONE, base));

        cursor += direction * plate_length;
    }

    let chain_end = cursor;
    let chain_start = Vec3A::new(0.0, 0.15, 2.0);
    let chain_midpoint = (chain_start + chain_end) * 0.5;

    let light_y = chain_midpoint.y + 2.5;
    let light_z = chain_midpoint.z + 1.5;

    let sphere_lights: [(f32, f32, f32); 3] = [
        ( 2.0, 0.025, 100.0),
        // ( 0.75, 0.05,  25.0),
        (0.0, 0.20,   6.5),
        (-2.0, 0.50, 4.0)
    ];
    for (light_x, roughness, _) in sphere_lights {
        let texture_id = textures.add_texture(Color::new(1.0, 1.0, 1.0));
        let material_id = materials.add_material(Emissive::new(texture_id));
        objects.push_into(Sphere::new(
            Vec3A::new(light_x, light_y, light_z),
            roughness, material_id
        ));
    }

    // added two plane lights on each side wall just in case
    let fill_intensity = 0.005;
    let fill_tex = textures.add_texture(Color::new(fill_intensity, fill_intensity, fill_intensity));
    let fill_mat = materials.add_material(Emissive::new(fill_tex));
    let fill_color = Vec3A::splat(fill_intensity);

    let left_light_primitive = Plane::new(
        Axis::YZ,
        Bounds2D::new(0.0..10.0, -5.0..20.0),
        -19.5,
        fill_mat,
    );
    objects.push_into(left_light_primitive.clone());

    let right_light_primitive = Plane::new(
        Axis::YZ,
        Bounds2D::new(0.0..10.0, -5.0..20.0),
        19.5,
        fill_mat,
    ).into_reversed();
    objects.push_into(right_light_primitive.clone());

    let bvh = BVH::new(&mut objects);

    for (light_x, roughness, intensity) in sphere_lights {
        let light = Light::new(
            Sphere::new(Vec3A::new(light_x, light_y, light_z), roughness, grey),
            Vec3A::splat(intensity),
        );
        lights.add_light(light);
    }

    lights.add_light(Light::new(left_light_primitive, fill_color));
    lights.add_light(Light::new(right_light_primitive, fill_color));

    SceneBuilder::new("Veach MIS")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .build()
        .expect("Failed to build Veach MIS scene")
}
