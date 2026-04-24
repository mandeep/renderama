use std::f32;
use std::sync::Arc;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use hitable::FlipNormals;
use materials::{Diffuse, Empty, Light, Reflective, Refractive};
use plane::{Axis, Plane};
use rectangle::Rectangle;
use sphere::Sphere;
use texture::{ConstantTexture, ImageTexture};
use transformations::{Rotate, Scale, Translate};
use triangle::TriangleMesh;
use volume::Volume;
use world::World;

pub fn three_spheres_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
    let origin = Vec3::new(0.0, 3.0, 6.0);
    let lookat = Vec3::new(0.0, 0.0, -1.5);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

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

    world.add(Sphere::new(Vec3::new(0.6, 0.0, -1.0),
                          Vec3::new(0.6, 0.0, -1.0),
                          0.5,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25), 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(-0.6, 0.0, -1.0),
                          Vec3::new(-0.6, 0.0, -1.0),
                          0.5,
                          Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.1),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(0.0, 0.1, -2.0),
                          Vec3::new(0.0, 0.1, -2.0),
                          0.5,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(0.0, -100.5, -1.0),
                          Vec3::new(0.0, -100.5, -1.0),
                          100.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5), 0.0),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = Plane::new(Axis::XY, 0.0, 0.0, 0.0, 0.0, 0.0, Empty::new());

    (String::from("Three Spheres"), camera, bvh, light)
}

pub fn random_spheres_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
    let origin = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

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

    world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
                          Vec3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5), 0.0),
                          0.0,
                          1.0));

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vec3 = Vec3::new(a as f32 + 0.9 * rand::random::<f32>(),
                                         0.2,
                                         b as f32 + 0.9 * rand::random::<f32>());

            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if material < 0.75 {
                    world.add(Sphere::new(center,
                                     center,
                                     0.2,
                                     Diffuse::new(ConstantTexture::new(rand::random::<f32>()
                                                                       * rand::random::<f32>(),
                                                                       rand::random::<f32>()
                                                                       * rand::random::<f32>(),
                                                                       rand::random::<f32>()
                                                                       * rand::random::<f32>()),
                                                  0.0),
                                     0.0,
                                     1.0));
                } else if material < 0.95 {
                    world.add(Sphere::new(center,
                                          center,
                                          0.2,
                                          Reflective::new(Vec3::new(0.5
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

    world.add(Sphere::new(Vec3::new(-2.0, 1.0, 0.0),
                          Vec3::new(-2.0, 1.0, 0.0),
                          1.0,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25), 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          1.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          -0.99,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(2.0, 1.0, 0.0),
                          Vec3::new(2.0, 1.0, 0.0),
                          1.0,
                          Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.05),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = Plane::new(Axis::XY, 0.0, 0.0, 0.0, 0.0, 0.0, Empty::new());

    (String::from("Random Spheres"), camera, bvh, light)
}

pub fn earth_scene(width: usize, height: usize) -> (String, Camera, World, Plane) {
    let origin = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
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

    world.add(Sphere::new(Vec3::new(0.0, 0.0, 0.0),
                          Vec3::new(0.0, 0.0, 0.0),
                          2.0,
                          Diffuse::new(ImageTexture::new("world_topo_nasa.jpg"), 0.0),
                          0.0,
                          1.0));

    let light = Plane::new(Axis::XY, 0.0, 0.0, 0.0, 0.0, 0.0, Empty::new());

    (String::from("Earth"), camera, world, light)
}

pub fn motion_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
    let origin = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = true;

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

    world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
                          Vec3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5), 0.0),
                          0.0,
                          1.0));

    let center: Vec3 = Vec3::new(0.9 * rand::random::<f32>(),
                                 0.2,
                                 0.9 * rand::random::<f32>());

    world.add(Sphere::new(center,
                          center + Vec3::new(0.0, 0.5 * rand::random::<f32>(), 0.0),
                          0.2,
                          Diffuse::new(ConstantTexture::new(rand::random::<f32>()
                                                            * rand::random::<f32>(),
                                                            rand::random::<f32>()
                                                            * rand::random::<f32>(),
                                                            rand::random::<f32>()
                                                            * rand::random::<f32>()),
                                       0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(-2.0, 1.0, 0.0),
                          Vec3::new(-2.0, 1.0, 0.0),
                          1.0,
                          Diffuse::new(ConstantTexture::new(0.75, 0.25, 0.25), 0.0),
                          0.0,
                          1.0));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = Plane::new(Axis::XY, 0.0, 0.0, 0.0, 0.0, 0.0, Empty::new());

    (String::from("Motion Blur"), camera, bvh, light)
}

pub fn simple_light_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
    let origin = Vec3::new(13.0, 3.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 50.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.1;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;
 
    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             aperture, focus_distance,
                             time0, time1, atmosphere);
 
    let mut world = World::new();
 
    // Ground
    world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
                          Vec3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5), 0.0),
                          0.0, 1.0));
 
    // Suzanne, at y = 2
    world.add(Translate::new(
        Vec3::new(0.0, 2.0, 0.0),
        Rotate::new(90.0, TriangleMesh::from(
            "suzanne.obj",
            Arc::new(Diffuse::new(ConstantTexture::new(1.0, 0.0, 0.0), 0.0))
        ))
    ));
 
    // Overhead rectangular light. Axis::XZ, y = 6 (above Suzanne's head).
    // Plane::hit for Axis::XZ sets normal = (0,1,0); but for lighting
    // we want the panel to illuminate downward, so we wrap in FlipNormals
    // so the useful side faces -y (down, toward the scene).
    // r0..r1 covers x in [-2, 2], s0..s1 covers z in [-2, 2]. k = 6.
    let rect_light_geometry = Plane::new(Axis::XZ,
                                         -2.0, 2.0,
                                         -2.0, 2.0,
                                          6.0,
                                          Light::new(ConstantTexture::new(4.0, 4.0, 4.0)));
 
    // Add the light to the world with its normals flipped so the
    // emissive side faces down. Without this, Light::emitted returns
    // zero for rays hitting it from below (the camera-facing side).
    world.add(FlipNormals::of(rect_light_geometry));
 
    // Optional extra accent: keep the sphere light too for fill.
    // Comment out if you want a single-light scene.
    world.add(Sphere::new(Vec3::new(0.0, 7.0, 4.0),
                          Vec3::new(0.0, 7.0, 4.0),
                          1.0,
                          Light::new(ConstantTexture::new(4.0, 4.0, 4.0)),
                          0.0, 1.0));
 
    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);
 
    // NEE target: the SAME geometry as the real rectangular light.
    // Emission value here is irrelevant — the integrator only uses this
    // Plane for sampling directions and PDFs, not for shading.
    // Note: do NOT FlipNormals this one — pdf_value/pdf_random don't
    // care about orientation the same way; they sample the rectangle area.
    let light_shape = Plane::new(Axis::XZ,
                                 -2.0, 2.0,
                                 -2.0, 2.0,
                                  6.0,
                                  Light::new(ConstantTexture::new(0.0, 0.0, 0.0)));
 
    (String::from("Simple Light"), camera, bvh, light_shape)
}

pub fn cornell_box_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
    let origin = Vec3::new(278.0, 278.0, -800.0);
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

    let roughness = 0.0;
    let red = Diffuse::new(ConstantTexture::new(0.65, 0.05, 0.05), roughness);
    let green = Diffuse::new(ConstantTexture::new(0.12, 0.45, 0.15), roughness);
    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73), roughness);
    let light = Light::new(ConstantTexture::new(35.0, 20.2, 5.6));

    // add the walls of the cornell box to the world
    world.add(FlipNormals::of(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, red)));

    world.add(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green));

    world.add(FlipNormals::of(Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light)));

    world.add(FlipNormals::of(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    world.add(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));

    world.add(FlipNormals::of(Plane::new(Axis::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    // add the boxes of the cornell box to the world
    let p0 = Vec3::new(0.0, 0.0, 0.0);
    let p1 = Vec3::new(165.0, 165.0, 165.0);

    world.add(Translate::new(Vec3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0, Rectangle::new(p0, p1, Arc::new(white.clone())))));

    let p2 = Vec3::new(165.0, 330.0, 165.0);

    world.add(Translate::new(Vec3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0, Rectangle::new(p0, p2, Arc::new(white.clone())))));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = Light::new(ConstantTexture::new(0.0, 0.0, 0.0));
    let light_shape = Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light);

    (String::from("Cornell Box"), camera, bvh, light_shape)
}

pub fn spheres_in_box_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
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

    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73), 0.0);
    let orange = Diffuse::new(ConstantTexture::new(1.0, 0.10, 0.0), 0.0);
    let light = Light::new(ConstantTexture::new(7.0, 7.0, 7.0));
    let ground = Diffuse::new(ConstantTexture::new(0.48, 0.83, 0.53), 0.0);

    let number_of_boxes = 20;

    for i in 0..number_of_boxes {
        for j in 0..number_of_boxes {
            let w = 100.0;
            let p0 = Vec3::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
            let p1 = p0 + Vec3::new(w, 100.0 * (rand::random::<f32>() + 0.01), w);
            world.add(Rectangle::new(p0, p1, Arc::new(ground.clone())));
        }
    }

    world.add(Plane::new(Axis::XZ, 123.0, 423.0, 147.0, 412.0, 554.0, light));

    world.add(Sphere::new(Vec3::new(400.0, 400.0, 200.0),
                          Vec3::new(430.0, 400.0, 200.0),
                          50.0,
                          orange,
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(260.0, 150.0, 45.0),
                          Vec3::new(260.0, 150.0, 45.0),
                          50.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(0.0, 150.0, 145.0),
                          Vec3::new(0.0, 150.0, 145.0),
                          50.0,
                          Reflective::new(Vec3::new(0.8, 0.8, 0.9), 1.0),
                          0.0,
                          1.0));

    let boundary = Sphere::new(Vec3::new(360.0, 150.0, 145.0),
                               Vec3::new(360.0, 150.0, 145.0),
                               70.0,
                               Refractive::new(1.5),
                               0.0,
                               1.0);

    world.add(boundary.clone());

    world.add(Volume::new(0.2, boundary.clone(), ConstantTexture::new(0.2, 0.4, 0.9)));

    let fog = Sphere::new(Vec3::new(0.0, 0.0, 0.0),
                          Vec3::new(0.0, 0.0, 0.0),
                          5000.0,
                          Refractive::new(1.5),
                          0.0,
                          1.0);

    world.add(Volume::new(0.0001, fog, ConstantTexture::new(1.0, 1.0, 1.0)));

    world.add(Sphere::new(Vec3::new(400.0, 200.0, 400.0),
                          Vec3::new(400.0, 200.0, 400.0),
                          100.0,
                          Diffuse::new(ImageTexture::new("world_topo_nasa.jpg"), 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vec3::new(220.0, 280.0, 300.0),
                          Vec3::new(220.0, 280.0, 300.0),
                          80.0,
                          Diffuse::new(ConstantTexture::new(0.6, 0.6, 0.6), 0.0),
                          0.0,
                          1.0));

    let number_of_spheres = 1000;
    for _ in 0..number_of_spheres {
        let center = Vec3::new(165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>(),
                               165.0 * rand::random::<f32>());

        let sphere = Sphere::new(center, center, 10.0, white.clone(), 0.0, 1.0);

        world.add(Translate::new(Vec3::new(-100.0, 270.0, 395.0), Rotate::new(15.0, sphere)));
    }

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = Light::new(ConstantTexture::new(0.0, 0.0, 0.0));
    let light_shape = Plane::new(Axis::XZ, 123.0, 423.0, 147.0, 412.0, 554.0, light);

    (String::from("Spheres in Box"), camera, bvh, light_shape)
}

pub fn cornell_box_bunny_scene(width: usize, height: usize)
                                 -> (String, Camera, BVH, Plane) {
    // Same camera as the classic Cornell box so the framing looks identical.
    let origin = Vec3::new(278.0, 278.0, -800.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.0;
    let focus_distance = 10.0;
    let time0 = 0.0;
    let time1 = 1.0;
    let atmosphere = false;
 
    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             aperture, focus_distance,
                             time0, time1, atmosphere);
 
    let mut world = World::new();
 
    let roughness = 0.0;
    let red   = Diffuse::new(ConstantTexture::new(0.65, 0.05, 0.05), roughness);
    let green = Diffuse::new(ConstantTexture::new(0.12, 0.45, 0.15), roughness);
    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73), roughness);
    let light = Light::new(ConstantTexture::new(25.0, 18.0, 10.0));
 
    // Cornell box walls — identical to the classic scene.
    //
    // Right wall (red) at x = 555, facing -x (into the room).
    world.add(FlipNormals::of(Plane::new(Axis::YZ,
                                         0.0, 555.0,
                                         0.0, 555.0,
                                         555.0, red)));
 
    // Left wall (green) at x = 0, facing +x.
    world.add(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green));
 
    // Ceiling light — a rectangle cut into the top.
    world.add(FlipNormals::of(Plane::new(Axis::XZ,
                                         213.0, 343.0,
                                         227.0, 332.0,
                                         554.0, light)));
 
    // Ceiling (white) at y = 555, facing -y.
    world.add(FlipNormals::of(Plane::new(Axis::XZ,
                                         0.0, 555.0,
                                         0.0, 555.0,
                                         555.0, white.clone())));
 
    // Floor (white) at y = 0, facing +y.
    world.add(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
 
    // Back wall (white) at z = 555, facing -z (toward camera).
    world.add(FlipNormals::of(Plane::new(Axis::XY,
                                         0.0, 555.0,
                                         0.0, 555.0,
                                         555.0, white.clone())));
 
    // -------------------------------------------------------------
    // Suzanne in the middle of the room.
    //
    // The mesh is centered near the origin in local space, roughly
    // 2.7 x 2.0 x 1.7 units in size. We:
    //   1. Scale by 120 so she's appropriately sized for the 555-unit room
    //      (about 240 units tall, so a bit less than half the room height).
    //   2. Rotate 180 degrees around Y so she faces the camera (the default
    //      Suzanne faces +Z, but the camera looks toward +Z from -z=-800,
    //      meaning the camera sees her from the +Z side — flipping orients
    //      her face toward us). Tweak this angle to taste.
    //   3. Translate to the center of the room in X and Z, and to y=170
    //      so she sits roughly mid-height with a bit of floor below her.
    //
    // Transform composition in this codebase is outside-in: the outermost
    // wrapper applies last. So Translate(Rotate(Scale(mesh))) scales first,
    // then rotates, then translates — which is what we want.
    // -------------------------------------------------------------
    let bunny_material = Arc::new(
        Refractive::new(2.4)
        // Diffuse::new(ConstantTexture::new(0.06, 0.25, 1.0), 0.0)
    );
 
    let bunny_mesh = TriangleMesh::from("bunny.obj", bunny_material);
 
    world.add(Translate::new(
        Vec3::new(224.0, -66.0, 278.0),
        Rotate::new(180.0,
            Scale::new(2000.0, bunny_mesh))
    ));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);
 
    // NEE target: same geometry as the ceiling light. Emission is zero
    // because the integrator only uses this Plane for sampling directions
    // and PDFs, not for shading contributions.
    let light = Light::new(ConstantTexture::new(0.0, 0.0, 0.0));
    let light_shape = Plane::new(Axis::XZ,
                                 213.0, 343.0,
                                 227.0, 332.0,
                                 554.0, light);
 
    (String::from("Cornell Box with Stanford Bunny"), camera, bvh, light_shape)
}
 