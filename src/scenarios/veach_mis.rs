use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use lights::Light;
use materials::{Diffuse, Emissive, Material, Reflective};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::SolidColor;
use transformations::TransformedMesh;
use world::World;
use mat;

/// Veach Multiple Importance Sampling test scene.
///
/// Four sphere lights of equal total power (4πr²×I = const) hang at Y=3.0.
/// Four reflective plates rise progressively in height and tilt — the front
/// plate is nearly horizontal (gentle tilt, low roughness) and each successive
/// plate is higher and more tilted (higher roughness). This demonstrates:
///   - Smooth/shallow plates: BSDF sampling is best (narrow specular lobe)
///   - Rough/steep plates: NEE / light sampling is best (wide lobe)
pub fn veach_mis_scene(width: usize, height: usize) -> Scene {
    let origin = Vec3::new(0.0, 1.2, -3.5);
    let lookat = Vec3::new(0.0, 1.9, 7.0);
    let view = Vec3::new(0.0, 1.0,  0.0);
    let fov = 35.0;
    let aspect_ratio = width as f32 / height as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let start_time = 0.0;
    let end_time = 1.0;

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
        aperture, focus_distance, start_time, end_time);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let grey = mat!(materials, Diffuse::new(SolidColor::new(0.99, 0.99, 0.99).into(), 0.0));

    // floor
    world.add(Plane::new(Axis::XZ, Bounds2D::new(-20.0..20.0, -5.0..25.0), 0.0, grey).into_geometry());

    // back wall
    world.add(Plane::new(Axis::XY, Bounds2D::new(-20.0..20.0, 0.0..15.0), 12.0, grey).into_reversed());

    // side walls, not sure if they do anything in this scene
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0),-20.0, grey).into_geometry());
    world.add(Plane::new(Axis::YZ, Bounds2D::new( 0.0..15.0, -5.0..25.0), 20.0, grey).into_reversed());

    let silver = Vec3::new(0.75, 0.75, 0.75);

    // use a cursor to place planes edge to edge
    let mut cursor = Vec3::new(0.0, 0.15, 2.0);
    let plate_length = 1.0;
    let plate_gap = 0.10;
    let visual_length = plate_length - plate_gap;

    let plate_configs: [(f32, f32); 5] = [
        (-14.0, 0.08),
        (-23.0, 0.04),
        (-32.0, 0.02),
        (-42.0, 0.01),
        (-52.0, 0.00),
    ];

    for (tilt_deg, fuzz) in plate_configs {
        let tilt_rad = tilt_deg.to_radians();

        // In Right-Handed, a negative X rotation tilts the Z-axis UP.
        // Direction vector: (0, -sin(theta), cos(theta))
        let direction = Vec3::new(0.0, -tilt_rad.sin(), tilt_rad.cos());

        // Center is half-way along that direction from the current cursor
        let center_pos = cursor + (direction * (plate_length * 0.5));

        let mat_id = mat!(materials, Reflective::new(silver, fuzz));
        let rot = Vec3::new(tilt_deg, 0.0, 0.0);
        let base = Rectangle::new(Vec3::new(-2.0, 0.1, -visual_length / 2.0),
            Vec3::new(2.0, 0.125, visual_length / 2.0), mat_id)
            .into();

        world.add(TransformedMesh::new(center_pos, rot, 1.0, base).into());

        cursor += direction * plate_length;
    }

    // After the plate loop, use the 'cursor' to find the midpoint so that the sphere
    // lights appear on all plates
    let chain_end = cursor; // Where the last plate ended
    let chain_start = Vec3::new(0.0, 0.15, 2.0);
    let chain_midpoint = (chain_start + chain_end) * 0.5;

    // Place lights relative to this midpoint
    let light_y = chain_midpoint.y + 2.5;
    let light_z = chain_midpoint.z + 1.5; // Offset slightly deeper for reflection math

    let sphere_lights: [(f32, f32, f32); 3] = [
        ( 2.0, 0.025, 100.0),
        // ( 0.75, 0.05,  25.0),
        (0.0, 0.20,   6.5),
        (-2.0, 0.50, 4.0)
    ];
    for (x, r, intensity) in sphere_lights {
        let mat = mat!(materials, Emissive::new(SolidColor::new(intensity, intensity, intensity).into()));
        world.add(Sphere::new(
            Vec3::new(x, light_y, light_z),
            Vec3::new(x, light_y, light_z),
            r, mat, 0.0, 1.0,
        ).into());
    }

    let fill_intensity = 0.005;
    let fill_mat = mat!(materials, Emissive::new(SolidColor::new(fill_intensity, fill_intensity, fill_intensity).into()));
    let fill_color = Vec3::splat(fill_intensity);

    // Left Fill Light (facing right toward the center)
    let left_light_geometry = Plane::new(
        Axis::YZ, 
        Bounds2D::new(0.0..10.0, -5.0..20.0), 
        -19.5, // Just inside the left wall
        fill_mat
    ).into_geometry();
    world.add(left_light_geometry.clone());

    // Right Fill Light (facing left toward the center)
    let right_light_geometry = Plane::new(
        Axis::YZ, 
        Bounds2D::new(0.0..10.0, -5.0..20.0), 
        19.5, // Just inside the right wall
        fill_mat
    ).into_reversed(); // Reverse normal to face inward
    world.add(right_light_geometry.clone());

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let mut light_sources: Vec<Light> = sphere_lights.iter().map(|&(x, r, intensity)| {
        Light::new(
            Sphere::new(Vec3::new(x, light_y, light_z), Vec3::new(x, light_y, light_z), r, grey, 0.0, 1.0).into(),
            Vec3::splat(intensity),
        )
    }).collect();

    light_sources.push(Light::new(left_light_geometry.into(), fill_color));
    light_sources.push(Light::new(right_light_geometry.into(), fill_color));

    Scene::new(
        String::from("Veach MIS"),
        bvh,
        materials,
        camera,
        light_sources,
        None,
        false,
    )
}
