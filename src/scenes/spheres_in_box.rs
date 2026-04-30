use std::f32;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use geometry::{Geometry};
use materials::{Diffuse, Light, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::{SolidColor, ImageTexture};
use transformations::TransformedMesh;
use volume::Volume;
use world::World;
use mat;


pub fn spheres_in_box_scene(width: usize, height: usize) -> Scene {
    let origin = Vec3::new(478.0, 278.0, -600.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let white = mat!(materials, Diffuse::new(SolidColor::new(0.73, 0.73, 0.73).into(), 0.0));
    let red = mat!(materials, Diffuse::new(SolidColor::new(1.0, 0.10, 0.20).into(), 0.0));
    let big_light = mat!(materials, Light::new(SolidColor::new(7.0, 7.0, 7.0).into()));
    let snow = mat!(materials, Diffuse::new(SolidColor::new(0.48, 0.83, 0.53).into(), 0.0));

    let number_of_boxes = 20;

    for i in 0..number_of_boxes {
        for j in 0..number_of_boxes {
            let w = 100.0;
            let p0 = Vec3::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
            let p1 = p0 + Vec3::new(w, 100.0 * (rand::random::<f32>() + 0.01), w);
            world.add(Rectangle::new(p0, p1, snow).into());
        }
    }

    world.add(Geometry::ReverseOrientation(Box::new(Plane::new(Axis::XZ, Bounds2D::new(123.0..423.0, 147.0..412.0), 554.0, big_light).into())));

    world.add(Sphere::new(Vec3::new(400.0, 400.0, 200.0),
                          Vec3::new(430.0, 400.0, 200.0),
                          50.0,
                          red,
                          0.0,
                          1.0).into());

    let refr_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));
    world.add(Sphere::new(Vec3::new(260.0, 150.0, 45.0),
                          Vec3::new(260.0, 150.0, 45.0),
                          50.0,
                          refr_idx,
                          0.0,
                          1.0).into());

    let refl_idx = mat!(materials, Reflective::new(Vec3::new(0.8, 0.8, 0.9), 0.0));
    world.add(Sphere::new(Vec3::new(0.0, 150.0, 145.0),
                          Vec3::new(0.0, 150.0, 145.0),
                          50.0,
                          refl_idx,
                          0.0,
                          1.0).into());

    let boundary: Geometry = Sphere::new(Vec3::new(360.0, 150.0, 145.0),
                               Vec3::new(360.0, 150.0, 145.0),
                               70.0,
                               refr_idx,
                               0.0,
                               1.0).into();

    let cloned_boundary = boundary.clone();
    world.add(boundary);

    let vol_idx = mat!(materials, Diffuse::new(SolidColor::new(0.2, 0.4, 0.9).into(), 0.0));
    world.add(Volume::new(0.2, cloned_boundary, vol_idx).into());

    let fog = Sphere::new(Vec3::new(0.0, 0.0, 0.0),
                          Vec3::new(0.0, 0.0, 0.0),
                          5000.0,
                          refr_idx,
                          0.0,
                          1.0).into();

    let fog_idx = mat!(materials, Diffuse::new(SolidColor::new(1.0, 1.0, 1.0).into(), 0.0));
    world.add(Volume::new(0.0001, fog, fog_idx).into());

    // Image provided by NASA; details can be found here:
    // https://science.nasa.gov/earth/earth-observatory/blue-marble-next-generation/
    // The map used for this render is a Base Map with Topography and Bathymetry
    let topo_idx = mat!(materials, Diffuse::new(ImageTexture::new("models/world_topo_nasa.jpg", 1.0).into(), 0.0));
    world.add(Sphere::new(Vec3::new(400.0, 200.0, 400.0),
                          Vec3::new(400.0, 200.0, 400.0),
                          100.0,
                          topo_idx,
                          0.0,
                          1.0).into());

    let marble = mat!(materials, Diffuse::new(ImageTexture::new("models/marble.jpg", 2.0).into(), 0.0));
    world.add(Sphere::new(Vec3::new(220.0, 280.0, 300.0),
                          Vec3::new(220.0, 280.0, 300.0),
                          80.0,
                          marble,
                          0.0,
                          1.0).into());

    let number_of_spheres = 1000;
    for _ in 0..number_of_spheres {
        let center = Vec3::new(165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>());

        let sphere = Sphere::new(center, center, 10.0, white, 0.0, 1.0);
        let transformed_sphere = TransformedMesh::new(Vec3::new(-100.0, 270.0, 395.0), Vec3::new(0.0, 15.0, 0.0), 1.0, sphere.into());
        world.add(transformed_sphere.into());
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);
    let light = mat!(materials, Light::new(SolidColor::new(0.0, 0.0, 0.0).into()));
    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(213.0..343.0, 227.0..332.0), 554.0, light);

    Scene::new(String::from("Spheres in Box"), bvh, materials, camera, Some(light_shape))
}
