use std::f32;

use nalgebra::core::Vector3;

use bvh::BVH;
use hitable::FlipNormals;
use materials::{Diffuse, Light, Reflective, Refractive};
use rectangle::{Plane, Rectangle};

use sphere::Sphere;
use texture::{ConstantTexture, ImageTexture};
use transformations::{Rotate, Translate};
use world::World;

pub fn three_spheres_scene() -> BVH {
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
                          Refractive::new(Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, -100.5, -1.0),
                          Vector3::new(0.0, -100.5, -1.0),
                          100.0,
                          Diffuse::new(ConstantTexture::new(0.5, 0.5, 0.5)),
                          0.0,
                          1.0));

    BVH::new(&world.objects, 0.0, 1.0)
}

pub fn random_spheres_scene() -> BVH {
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
                    world.add(Sphere::new(center,
                                          center,
                                          0.2,
                                          Refractive::new(Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0),
                                          0.0,
                                          1.0));

                    world.add(Sphere::new(center,
                                          center,
                                          -0.19,
                                          Refractive::new(Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0),
                                          0.0,
                                          1.0));
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
                          Refractive::new(Vector3::new(1.0, 1.0, 1.0), 1.5, 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(0.0, 1.0, 0.0),
                          Vector3::new(0.0, 1.0, 0.0),
                          -0.99,
                          Refractive::new(Vector3::new(1.0, 1.0, 1.0), 1.5, 0.0),
                          0.0,
                          1.0));

    world.add(Sphere::new(Vector3::new(2.0, 1.0, 0.0),
                          Vector3::new(2.0, 1.0, 0.0),
                          1.0,
                          Reflective::new(Vector3::new(0.5, 0.5, 0.5), 0.05),
                          0.0,
                          1.0));

    BVH::new(&world.objects, 0.0, 1.0)
}

pub fn earth_scene() -> World {
    let mut world = World::new();

    world.add(Sphere::new(Vector3::new(0.0, 0.0, 0.0),
                          Vector3::new(0.0, 0.0, 0.0),
                          2.0,
                          Diffuse::new(ImageTexture::new("earthmap.png")),
                          0.0,
                          1.0));

    world
}

pub fn motion_scene() -> BVH {
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

    BVH::new(&world.objects, 0.0, 1.0)
}

pub fn simple_light_scene() -> BVH {
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

    world.add(Rectangle::new(Plane::XY,
                             3.0,
                             5.0,
                             1.0,
                             3.0,
                             -2.0,
                             Light::new(ConstantTexture::new(4.0, 4.0, 4.0))));

    BVH::new(&world.objects, 0.0, 1.0)
}

pub fn cornell_box_scene() -> BVH {
    let mut world = World::new();

    let red = Diffuse::new(ConstantTexture::new(0.65, 0.05, 0.05));
    let green = Diffuse::new(ConstantTexture::new(0.12, 0.45, 0.15));
    let white = Diffuse::new(ConstantTexture::new(0.73, 0.73, 0.73));
    let light = Light::new(ConstantTexture::new(15.0, 15.0, 15.0));

    // add the walls of the cornell box to the world
    world.add(FlipNormals::of(Rectangle::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, red)));

    world.add(Rectangle::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green));

    world.add(Rectangle::new(Plane::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light));

    world.add(FlipNormals::of(Rectangle::new(Plane::XZ,
                                             0.0,
                                             555.0,
                                             0.0,
                                             555.0,
                                             555.0,
                                             white.clone())));

    world.add(Rectangle::new(Plane::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));

    world.add(FlipNormals::of(Rectangle::new(Plane::XY,
                                             0.0,
                                             555.0,
                                             0.0,
                                             555.0,
                                             555.0,
                                             white.clone())));

    // add the boxes of the cornell box to the world
    let p0 = Vector3::new(0.0, 0.0, 0.0);
    let mut p1 = Vector3::new(165.0, 165.0, 165.0);

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         Rectangle::new(Plane::XY,
                                                        p0.x,
                                                        p1.x,
                                                        p0.y,
                                                        p1.y,
                                                        p1.z,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         FlipNormals::of(Rectangle::new(Plane::XY,
                                                                        p0.x,
                                                                        p1.x,
                                                                        p0.y,
                                                                        p1.y,
                                                                        p0.z,
                                                                        white.clone())))));

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         Rectangle::new(Plane::XZ,
                                                        p0.x,
                                                        p1.x,
                                                        p0.z,
                                                        p1.z,
                                                        p1.y,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         FlipNormals::of(Rectangle::new(Plane::XZ,
                                                                        p0.x,
                                                                        p1.x,
                                                                        p0.z,
                                                                        p1.z,
                                                                        p0.y,
                                                                        white.clone())))));

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         Rectangle::new(Plane::YZ,
                                                        p0.y,
                                                        p1.y,
                                                        p0.z,
                                                        p1.z,
                                                        p1.x,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(130.0, 0.0, 65.0),
                             Rotate::new(-18.0,
                                         FlipNormals::of(Rectangle::new(Plane::YZ,
                                                                        p0.y,
                                                                        p1.y,
                                                                        p0.z,
                                                                        p1.z,
                                                                        p0.x,
                                                                        white.clone())))));

    p1 = Vector3::new(165.0, 330.0, 165.0);

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         Rectangle::new(Plane::XY,
                                                        p0.x,
                                                        p1.x,
                                                        p0.y,
                                                        p1.y,
                                                        p1.z,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         FlipNormals::of(Rectangle::new(Plane::XY,
                                                                        p0.x,
                                                                        p1.x,
                                                                        p0.y,
                                                                        p1.y,
                                                                        p0.z,
                                                                        white.clone())))));

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         Rectangle::new(Plane::XZ,
                                                        p0.x,
                                                        p1.x,
                                                        p0.z,
                                                        p1.z,
                                                        p1.y,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         FlipNormals::of(Rectangle::new(Plane::XZ,
                                                                        p0.x,
                                                                        p1.x,
                                                                        p0.z,
                                                                        p1.z,
                                                                        p0.y,
                                                                        white.clone())))));

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         Rectangle::new(Plane::YZ,
                                                        p0.y,
                                                        p1.y,
                                                        p0.z,
                                                        p1.z,
                                                        p1.x,
                                                        white.clone()))));

    world.add(Translate::new(Vector3::new(265.0, 0.0, 295.0),
                             Rotate::new(15.0,
                                         FlipNormals::of(Rectangle::new(Plane::YZ,
                                                                        p0.y,
                                                                        p1.y,
                                                                        p0.z,
                                                                        p1.z,
                                                                        p0.x,
                                                                        white.clone())))));

    BVH::new(&world.objects, 0.0, 1.0)
}
