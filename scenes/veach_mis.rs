use std::f32::consts::PI;

use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::Camera;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material, Reflective};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::Color;
use crate::transformations::TransformedMesh;

use crate::mat;


pub fn veach_mis_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 1.2, -3.5);
    let lookat = Vec3A::new(0.0, 1.9, 7.0);
    let view = Vec3A::new(0.0, 1.0,  0.0);
    let old_fov = 35.0;
    let (aspect_width, aspect_height) = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let fov_radians = old_fov * PI / 180.0;
    let sensor_height = 24.0;
    let focal_length = (sensor_height / 2.0) / (fov_radians / 2.0).tan();
    let f_stop = std::f32::INFINITY;
    let focus_distance = 10.0;
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

    let grey = mat!(materials, Diffuse::new(Color::new(0.99, 0.99, 0.99).into(), 0.0));

    // floor
    objects.push(Plane::new(Axis::XZ, Bounds2D::new(-20.0..20.0, -5.0..25.0), 0.0, grey).into_primitive());

    // back wall
    objects.push(Plane::new(Axis::XY, Bounds2D::new(-20.0..20.0, 0.0..15.0), 12.0, grey).into_reversed());

    // side walls, not sure if they do anything in this scene
    objects.push(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0),-20.0, grey).into_primitive());
    objects.push(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0), 20.0, grey).into_reversed());

    let silver = Color::new(0.75, 0.75, 0.75);

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

    for (tilt_deg, fuzz) in plate_configs {
        let tilt_rad = tilt_deg.to_radians();

        let direction = Vec3A::new(0.0, -tilt_rad.sin(), tilt_rad.cos());

        let center_pos = cursor + (direction * (plate_length * 0.5));

        let mat_id = mat!(materials, Reflective::new(silver.into(), fuzz));
        let rot = Vec3A::new(tilt_deg, 0.0, 0.0);
        let base = Rectangle::new(Vec3A::new(-2.25, 0.1, -visual_length / 2.0),
            Vec3A::new(2.25, 0.125, visual_length / 2.0), mat_id)
            .into();

        objects.push(TransformedMesh::new(center_pos, rot, Vec3A::ONE, base).into());

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
    for (x, r, intensity) in sphere_lights {
        let mat = mat!(materials, Emissive::new(Color::new(intensity, intensity, intensity).into()));
        objects.push(Sphere::new(
            Vec3A::new(x, light_y, light_z),
            r, mat,
        ).into());
    }

    // added two plane lights on each side wall just in case
    let fill_intensity = 0.005;
    let fill_mat = mat!(materials, Emissive::new(Color::new(fill_intensity, fill_intensity, fill_intensity).into()));
    let fill_color = Vec3A::splat(fill_intensity);

    let left_light_primitive = Plane::new(
        Axis::YZ, 
        Bounds2D::new(0.0..10.0, -5.0..20.0), 
        -19.5,
        fill_mat
    ).into_primitive();
    objects.push(left_light_primitive.clone());

    let right_light_primitive = Plane::new(
        Axis::YZ, 
        Bounds2D::new(0.0..10.0, -5.0..20.0), 
        19.5,
        fill_mat
    ).into_reversed();
    objects.push(right_light_primitive.clone());

    let bvh = BVH::new(&mut objects);

    let mut light_sources: Vec<Light> = sphere_lights.iter().map(|&(x, r, intensity)| {
        Light::new(
            Sphere::new(Vec3A::new(x, light_y, light_z), r, grey).into(),
            Vec3A::splat(intensity),
        )
    }).collect();

    light_sources.push(Light::new(left_light_primitive.into(), fill_color));
    light_sources.push(Light::new(right_light_primitive.into(), fill_color));

    SceneBuilder::new("Veach MIS")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_lights(light_sources)
        .build()
        .expect("Failed to build Veach MIS scene")
}
