use glam::{Vec2, Vec3A};
use rand::RngExt;
use rand_pcg::Pcg64Mcg;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::extensions::PushInto;
use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material, Reflective, Refractive, Volumetric};
use crate::plane::{Axis, Bounds2D, Plane};
use crate::rectangle::Rectangle;
use crate::scene::{Scene, SceneBuilder};
use crate::sphere::Sphere;
use crate::texture::{Color, ImageTexture, Texture};
use crate::transformations::TransformedMesh;
use crate::volume::Volume;

use crate::{mat, tex};


pub fn spheres_in_box_scene(width: Option<usize>, height: Option<usize>, rng: &mut Pcg64Mcg) -> Scene {
    let origin = Vec3A::new(478.0, 278.0, -600.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let fov = 40.0;
    let focus_distance = 10.0;
    let world_scale = 1.0;
    let fps = 24.0;
    let frame_duration = 1.0 / fps;
    let shutter_speed = 1.0;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        .with_fov(fov)
        .with_focus_distance(focus_distance)
        .with_world_scale(world_scale)
        .with_frame_duration(frame_duration)
        .with_shutter_speed(shutter_speed)
        .with_resolution(width.unwrap_or(2048), height.unwrap_or(2048));
    let camera = Camera::new(&camera_options);

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();

    let white_id = tex!(textures, Color::new(0.73, 0.73, 0.73));
    let white = mat!(materials, Diffuse::new(white_id, 0.0));
    let red_id = tex!(textures, Color::new(1.0, 0.10, 0.20));
    let red = mat!(materials, Diffuse::new(red_id, 0.0));
    let light_id = tex!(textures, Color::new(7.0, 7.0, 7.0));
    let big_light = mat!(materials, Emissive::new(light_id));
    let snow_id = tex!(textures, Color::new(0.48, 0.83, 0.53));
    let snow = mat!(materials, Diffuse::new(snow_id, 0.0));

    let number_of_boxes = 20;

    for i in 0..number_of_boxes {
        for j in 0..number_of_boxes {
            let w = 100.0;
            let p0 = Vec3A::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
            let p1 = p0 + Vec3A::new(w, 100.0 * (rng.random::<f32>() + 0.01), w);
            objects.push_into(Rectangle::new(p0, p1, snow));
        }
    }

    objects.push_into(Plane::new(Axis::XZ, Bounds2D::new(123.0..423.0, 147.0..412.0), 554.0, big_light).into_reversed());

    let sphere = Sphere::new(Vec3A::new(0.0, 0.0, 0.0), 1.0, red);
    let transformed_sphere = TransformedMesh::new(Vec3A::new(400.0, 400.0, 200.0), Vec3A::ZERO, Vec3A::splat(50.0), sphere);
    let motion_mesh = transformed_sphere.into_motion()
        .with_translation(Vec3A::new(430.0, 400.0, 200.0))
        .with_time_range(frame_duration, shutter_speed)
        .build();
    objects.push_into(motion_mesh);

    let refr_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
    let refr_idx = mat!(materials, Refractive::new(refr_id, 1.5));
    objects.push_into(Sphere::new(Vec3A::new(260.0, 150.0, 45.0), 50.0, refr_idx));

    let refl_id = tex!(textures, Color::new(0.8, 0.8, 0.9));
    let refl_idx = mat!(materials, Reflective::new(refl_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(0.0, 150.0, 145.0), 50.0, refl_idx));

    let boundary = Sphere::new(Vec3A::new(360.0, 150.0, 145.0), 70.0, refr_idx);

    let cloned_boundary = boundary.clone();
    objects.push_into(boundary);

    let vol_id = tex!(textures, Color::new(0.2, 0.4, 0.9));
    let vol_idx = mat!(materials, Volumetric::new(vol_id));
    objects.push_into(Volume::new(0.2, cloned_boundary, vol_idx));

    let fog = Sphere::new(Vec3A::new(0.0, 0.0, 0.0), 5000.0, refr_idx);

    let fog_id = tex!(textures, Color::new(1.0, 1.0, 1.0));
    let fog_idx = mat!(materials, Volumetric::new(fog_id));
    objects.push_into(Volume::new(0.0001, fog, fog_idx));

    // Image provided by NASA; details can be found here:
    // https://science.nasa.gov/earth/earth-observatory/blue-marble-next-generation/
    // The map used for this render is a Base Map with Topography and Bathymetry
    let topo_id = tex!(textures, ImageTexture::new("extras/textures/world_topo_nasa.jpg", Vec2::splat(1.0)));
    let topo_idx = mat!(materials, Diffuse::new(topo_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(400.0, 200.0, 400.0), 100.0, topo_idx));

    let marble_id = tex!(textures, ImageTexture::new("extras/textures/marble.jpg", Vec2::splat(2.0)));
    let marble = mat!(materials, Diffuse::new(marble_id, 0.0));
    objects.push_into(Sphere::new(Vec3A::new(220.0, 280.0, 300.0), 80.0, marble));

    let number_of_spheres = 1000;
    for _ in 0..number_of_spheres {
        let center = Vec3A::new(165.0 * rng.random::<f32>(),
                               165.0 * rng.random::<f32>(),
                               165.0 * rng.random::<f32>());

        let sphere = Sphere::new(center, 10.0, white);
        let transformed_sphere = TransformedMesh::new(Vec3A::new(-100.0, 270.0, 395.0), Vec3A::new(0.0, 15.0, 0.0), Vec3A::ONE, sphere);
        objects.push_into(transformed_sphere);
    }

    let bvh = BVH::new(&mut objects);
    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(123.0..423.0, 147.0..412.0), 554.0, white);
    let light = vec![Light::new(light_shape, Vec3A::splat(7.0))];

    SceneBuilder::new("Spheres in Box")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(light)
        .build()
        .expect("Failed to build Spheres in Box scene")
}
