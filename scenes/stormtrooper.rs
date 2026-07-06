use glam::Vec3A;

use crate::bvh::BVH;
use crate::camera::{Camera, CameraOptions};
use crate::environment::EnvironmentMap;
use crate::extensions::PushInto;
use crate::io::load_obj;
use crate::lights::Light;
use crate::materials::Material;
use crate::primitive::Primitive;
use crate::scene::{Scene, SceneBuilder};
use crate::texture::Texture;
use crate::transformations::TransformedMesh;

pub fn stormtrooper_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    // let location = Vec3A::new(0.28389, -9.13327, -0.79440);
    // let rotation = Vec3A::new(93.276, 5.3508, 6.2166);
    let origin = Vec3A::new(0.25, -0.5, 5.5);
    let lookat = Vec3A::new(0.0, 0.0, -2.0);
    let focal_length = 52.0;
    let focus_distance = 4.0;
    let f_stop = 0.7;
    // let focus_distance = 8.0;
    // let f_stop = 0.3;

    let camera_options = CameraOptions::new()
        .with_origin(origin)
        .with_lookat(lookat)
        // .with_origin(location)
        // .with_rotation(rotation)
        .with_focal_length(focal_length)
        .with_focus_distance(focus_distance)
        .with_fstop(f_stop)
        // .with_sensor_width(32.0)
        // .with_up_axis(UpAxis::Z)
        .with_resolution(width.unwrap_or(1920), height.unwrap_or(1080));
    let camera = Camera::new(&camera_options);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut textures: Vec<Texture> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    // an example of material overrides for the obj loader
    // Some(overrides) would be passed into the io::load_obj constructor
    //
    // let mut overrides: HashMap<String, Material> = HashMap::new();

    // overrides.insert(
    //     "white".to_string(),
    //     Plastic::new(Color::new(0.9, 0.9, 0.9), 1.0, 1.49)
    // );

    // overrides.insert(
    //     "Material.001".to_string(),
    //     Plastic::new(Color::new(0.01, 0.01, 0.01), 1.0, 1.49)
    // );

    let meshes = load_obj("extras/models/stormtrooper.obj", &mut materials, &mut textures, Some(&mut lights));

    for mesh in meshes {
        let transformed = TransformedMesh::from(mesh);
        objects.push_into(transformed);
    }

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/car_studio_lighting.exr", 0.7);

    SceneBuilder::new("Stormtrooper")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials, textures)
        .with_lights(lights)
        .with_environment(environment)
        .build()
        .expect("Failed to build Stormtrooper scene")
}