use std::f32;
use std::sync::Arc;

use glam::Vec3A;

use bvh::BVH;
use camera::Camera;
use primitive::{Primitive};
use lights::Light;
use materials::{Diffuse, Emissive, Isotropic, Reflective, Refractive, Material};
use plane::{Axis, Bounds2D, Plane};
use rectangle::Rectangle;
use scene::Scene;
use sphere::Sphere;
use texture::{SolidColor, ImageTexture};
use transformations::TransformedMesh;
use volume::Volume;

use mat;


pub fn spheres_in_box_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(478.0, 278.0, -600.0);
    let lookat = Vec3A::new(278.0, 278.0, 0.0);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width.unwrap_or(2048) as f32, height.unwrap_or(2048) as f32);
    let aperture = 0.0;
    let focus_distance = 10.0;

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut objects = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let white = mat!(materials, Diffuse::new(SolidColor::new(0.73, 0.73, 0.73).into(), 0.0));
    let red = mat!(materials, Diffuse::new(SolidColor::new(1.0, 0.10, 0.20).into(), 0.0));
    let big_light = mat!(materials, Emissive::new(SolidColor::new(7.0, 7.0, 7.0).into()));
    let snow = mat!(materials, Diffuse::new(SolidColor::new(0.48, 0.83, 0.53).into(), 0.0));

    let number_of_boxes = 20;

    for i in 0..number_of_boxes {
        for j in 0..number_of_boxes {
            let w = 100.0;
            let p0 = Vec3A::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
            let p1 = p0 + Vec3A::new(w, 100.0 * (rand::random::<f32>() + 0.01), w);
            objects.push(Rectangle::new(p0, p1, snow).into());
        }
    }

    objects.push(Primitive::ReverseOrientation(Arc::new(Plane::new(Axis::XZ, Bounds2D::new(123.0..423.0, 147.0..412.0), 554.0, big_light).into())));

    objects.push(Sphere::new(Vec3A::new(400.0, 400.0, 200.0),
                          50.0,
                          red).into());

    let refr_idx = mat!(materials, Refractive::new(1.5, Vec3A::ONE));
    objects.push(Sphere::new(Vec3A::new(260.0, 150.0, 45.0),
                          50.0,
                          refr_idx).into());

    let refl_idx = mat!(materials, Reflective::new(Vec3A::new(0.8, 0.8, 0.9), 0.0));
    objects.push(Sphere::new(Vec3A::new(0.0, 150.0, 145.0),
                          50.0,
                          refl_idx).into());

    let boundary: Primitive = Sphere::new(Vec3A::new(360.0, 150.0, 145.0),
                               70.0,
                               refr_idx).into();

    let cloned_boundary = boundary.clone();
    objects.push(boundary);

    let vol_idx = mat!(materials, Isotropic::new(SolidColor::new(0.2, 0.4, 0.9).into()));
    objects.push(Volume::new(0.2, cloned_boundary, vol_idx).into());

    let fog = Sphere::new(Vec3A::new(0.0, 0.0, 0.0),
                          5000.0,
                          refr_idx).into();

    let fog_idx = mat!(materials, Isotropic::new(SolidColor::new(1.0, 1.0, 1.0).into()));
    objects.push(Volume::new(0.0001, fog, fog_idx).into());

    // Image provided by NASA; details can be found here:
    // https://science.nasa.gov/earth/earth-observatory/blue-marble-next-generation/
    // The map used for this render is a Base Map with Topography and Bathymetry
    let topo_idx = mat!(materials, Diffuse::new(ImageTexture::new("docs/textures/world_topo_nasa.jpg", 1.0).into(), 0.0));
    objects.push(Sphere::new(Vec3A::new(400.0, 200.0, 400.0),
                          100.0,
                          topo_idx).into());

    let marble = mat!(materials, Diffuse::new(ImageTexture::new("docs/textures/marble.jpg", 2.0).into(), 0.0));
    objects.push(Sphere::new(Vec3A::new(220.0, 280.0, 300.0),
                          80.0,
                          marble).into());

    let number_of_spheres = 1000;
    for _ in 0..number_of_spheres {
        let center = Vec3A::new(165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>());

        let sphere = Sphere::new(center, 10.0, white);
        let transformed_sphere = TransformedMesh::new(Vec3A::new(-100.0, 270.0, 395.0), Vec3A::new(0.0, 15.0, 0.0), 1.0, sphere.into());
        objects.push(transformed_sphere.into());
    }

    let bvh = BVH::new(&mut objects, 0.0, 1.0);
    let light_shape = Plane::new(Axis::XZ, Bounds2D::new(123.0..423.0, 147.0..412.0), 554.0, white);

    Scene::new(String::from("Spheres in Box"), bvh, materials, camera, vec![Light::new(light_shape.into(), Vec3A::splat(7.0))], None, false)
}
