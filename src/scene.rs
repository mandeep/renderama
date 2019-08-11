use std::f32;
use std::sync::Arc;

use nalgebra::core::Vector3;

use bvh::BVH;
use camera::Camera;
use hitable::FlipNormals;
use materials::{Diffuse, Light, Reflective, Refractive};
use plane::{Axis, Plane};
use rectangle::Rectangle;
use sphere::Sphere;
use texture::{ConstantTexture, ImageTexture};
use transformations::{Rotate, Translate};
use volume::Volume;
use world::World;

pub fn three_spheres_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(0.0, 3.0, 6.0);
    let lookat = Vector3::new(0.0, 0.0, -1.5);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.6, 0.0, -1.0),
                          Vector3::new(0.6, 0.0, -1.0),
                          0.5,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25)),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(-0.6, 0.0, -1.0),
                          Vector3::new(-0.6, 0.0, -1.0),
                          0.5,
                          Reflective::new(Vector3::new(0.5, 0.5, 0.5), 0.1),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 0.1, -2.0),
                          Vector3::new(0.0, 0.1, -2.0),
                          0.5,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, -100.5, -1.0),
                          Vector3::new(0.0, -100.5, -1.0),
                          100.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}

pub fn random_spheres_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(13.0, 2.0, 3.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.0, -1000.0, 0.0),
                          Vector3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)),
                          0.0,
                          1.0));

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vector3<f32> = Vector3::new(a as f32 + 0.9 * rand::random::<f32>(),
                                                    0.2,
                                                    b as f32 + 0.9 * rand::random::<f32>());

            if (center - Vector3::new(4.0, 0.2, 0.0)).norm() > 0.9 {
                if material < 0.75 {
                    world.add(Sphere::new(center,
                                     center,
                                     0.2,
                                     Diffuse::new(ConstantTexture::new(rand::random::<f32>()
                                                                       * rand::random::<f32>(),
                                                                       rand::random::<f32>()
                                                                       * rand::random::<f32>(),
                                                                       rand::random::<f32>()
                                                                       * rand::random::<f32>())),
                                     0.0,
                                     1.0));
                } else if material < 0.95 {
                    world.add(Sphere::new(center,
                                          center,
                                          0.2,
                                          Reflective::new(Vector3::new(0.5
                                                                       * (1.0
                                                                          * rand::random::<f32>()),
                                                                       0.5
                                                                       * (1.0
                                                                          * rand::random::<f32>()),
                                                                       0.5
                                                                       * (1.0
                                                                          * rand::random::<f32>())),
                                                          0.5 * rand::random::<f32>()),
                                          0.0,
                                          1.0));
                } else {
                    world.add(Sphere::new(center, center, 0.2, Refractive::new(1.5), 0.0, 1.0));

                    world.add(Sphere::new(center, center, -0.19, Refractive::new(1.5), 0.0, 1.0));
                }
            }
        }
    }

    world.add(Sphere::new(Vector3::new(-2.0, 1.0, 0.0),
                          Vector3::new(-2.0, 1.0, 0.0),
                          1.0,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25)),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 1.0, 0.0),
                          Vector3::new(0.0, 1.0, 0.0),
                          1.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 1.0, 0.0),
                          Vector3::new(0.0, 1.0, 0.0),
                          -0.99,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(2.0, 1.0, 0.0),
                          Vector3::new(2.0, 1.0, 0.0),
                          1.0,
                          Reflective::new(Vector3::new(0.5, 0.5, 0.5), 0.05),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}

pub fn earth_scene(width: u32, height: u32) -> (Camera, World) {
    let origin = Vector3::new(13.0, 2.0, 3.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.0, 0.0, 0.0),
                          Vector3::new(0.0, 0.0, 0.0),
                          2.0,
                          Diffuse::new(ImageTexture::new("world_topo_nasa.jpg")),
                          0.0,
                          1.0));

    (camera, world)
}

pub fn motion_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(13.0, 2.0, 3.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.0, -1000.0, 0.0),
                          Vector3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)),
                          0.0,
                          1.0));

    let center: Vector3<f32> = Vector3::new(0.9 * rand::random::<f32>(),
                                            0.2,
                                            0.9 * rand::random::<f32>());

    world.add(Sphere::new(center,
                          center + Vector3::new(0.0, 0.5 * rand::random::<f32>(), 0.0),
                          0.2,
                          Diffuse::new(ConstantTexture::new(rand::random::<f32>()
                                                            * rand::random::<f32>(),
                                                            rand::random::<f32>()
                                                            * rand::random::<f32>(),
                                                            rand::random::<f32>()
                                                            * rand::random::<f32>())),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(-2.0, 1.0, 0.0),
                          Vector3::new(-2.0, 1.0, 0.0),
                          1.0,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25)),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}

pub fn simple_light_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(13.0, 2.0, 3.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.0, -1000.0, 0.0),
                          Vector3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 2.0, 0.0),
                          Vector3::new(0.0, 2.0, 0.0),
                          2.0,
                          Diffuse::new(ConstantTexture::new(1.0, 0.0, 0.0)),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 7.0, 0.0),
                          Vector3::new(0.0, 7.0, 0.0),
                          2.0,
                          Light::new(ConstantTexture::new(4.0, 4.0, 4.0)),
                          0.0,
                          1.0));

    world.add(Plane::new(Axis::XY,
                         3.0,
                         5.0,
                         1.0,
                         3.0,
                         -2.0,
                         Light::new(ConstantTexture::new(4.0, 4.0, 4.0))));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}

pub fn cornell_box_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(278.0, 278.0, -800.0);
    let lookat = Vector3::new(278.0, 278.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    let red = Diffuse::new(ConstantTexture::new(0.65, 0.05, 0.05));
    let green = Diffuse::new(ConstantTexture::new(0.12, 0.45, 0.15));
    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73));
    let light = Light::new(ConstantTexture::new(15.0, 15.0, 15.0));

    // add the walls of the cornell box to the world
    world.add(FlipNormals::of(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, red)));

    world.add(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green));

    world.add(Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light));

    world.add(FlipNormals::of(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    world.add(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));

    world.add(FlipNormals::of(Plane::new(Axis::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    // add the boxes of the cornell box to the world
    let p0 = Vector3::new(0.0, 0.0, 0.0);
    let p1 = Vector3::new(165.0, 165.0, 165.0);

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0, Rectangle::new(p0, p1, Arc::new(white.clone())))));

    let p2 = Vector3::new(165.0, 330.0, 165.0);

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0, Rectangle::new(p0, p2, Arc::new(white.clone())))));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}

pub fn spheres_in_box_scene(width: u32, height: u32) -> (Camera, BVH) {
    let origin = Vector3::new(478.0, 278.0, -600.0);
    let lookat = Vector3::new(278.0, 278.0, 0.0);
    let view = Vector3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;

    let camera = Camera::new(origin,
                             &lookat,
                             &view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance,
                             time0,
                             time1,
                             atmosphere);

    let mut world = World::new();

    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73));
    let orange = Diffuse::new(ConstantTexture::new(1.0, 0.10, 0.0));
    let light = Light::new(ConstantTexture::new(7.0, 7.0, 7.0));
    let ground = Diffuse::new(ConstantTexture::new(0.48, 0.83, 0.53));

    let number_of_boxes = 20;

    for i in 0..number_of_boxes {
        for j in 0..number_of_boxes {
            let w = 100.0;
            let p0 = Vector3::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
            let p1 = p0 + Vector3::new(w, 100.0 * (rand::random::<f32>() + 0.01), w);
            world.add(Rectangle::new(p0, p1, Arc::new(ground.clone())));
        }
    }

    world.add(Plane::new(Axis::XZ, 123.0, 423.0, 147.0, 412.0, 554.0, light));

    world.add(Sphere::new(Vector3::new(400.0, 400.0, 200.0),
                          Vector3::new(430.0, 400.0, 200.0),
                          50.0,
                          orange,
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(260.0, 150.0, 45.0),
                          Vector3::new(260.0, 150.0, 45.0),
                          50.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 150.0, 145.0),
                          Vector3::new(0.0, 150.0, 145.0),
                          50.0,
                          Reflective::new(Vector3::new(0.8, 0.8, 0.9), 1.0),
                          0.0,
                          1.0));

    let boundary = Sphere::new(Vector3::new(360.0, 150.0, 145.0),
                               Vector3::new(360.0, 150.0, 145.0),
                               70.0,
                               Refractive::new(1.5),
                               0.0,
                               1.0);

    world.add(boundary.clone());

    world.add(Volume::new(0.2, boundary.clone(), ConstantTexture::new(0.2, 0.4, 0.9)));

    let fog = Sphere::new(Vector3::new(0.0, 0.0, 0.0),
                          Vector3::new(0.0, 0.0, 0.0),
                          5000.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0);

    world.add(Volume::new(0.0001, fog, ConstantTexture::new(1.0, 1.0, 1.0)));

    world.add(Sphere::new(Vector3::new(400.0, 200.0, 400.0),
                          Vector3::new(400.0, 200.0, 400.0),
                          100.0,
                          Diffuse::new(ImageTexture::new("world_topo_nasa.jpg")),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(220.0, 280.0, 300.0),
                          Vector3::new(220.0, 280.0, 300.0),
                          80.0,
                          Diffuse::new(ConstantTexture::new(0.6, 0.6, 0.6)),
                          0.0,
                          1.0));

    let number_of_spheres = 1000;
    for _ in 0..number_of_spheres {
        let center = Vector3::new(165.0 * rand::random::<f32>(),
                                  165.0 * rand::random::<f32>(),
                                  165.0 * rand::random::<f32>());

        let sphere = Sphere::new(center, center, 10.0, white.clone(), 0.0, 1.0);

        world.add(Translate::new(Vector3::new(-100.0, 270.0, 395.0),
                                 Rotate::new(15.0, sphere)));
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    (camera, bvh)
}
