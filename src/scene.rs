use std::f32;

use glam::Vec3;

use bvh::BVH;
use camera::Camera;
use geometry::{Geometry};
use materials::{Diffuse, Light, Plastic, Reflective, Refractive, Material};
use plane::{Axis, Plane};
use rectangle::Rectangle;
use sphere::Sphere;
use texture::{SolidColor, ImageTexture};
use transformations::{TransformedMesh, Rotate, Scale, Translate};
use triangle::{Triangle, TriangleMesh};
use volume::Volume;
use world::World;
use mat;


pub struct Scene {
    pub materials: Vec<Material>,
    pub world: BVH,
}

impl Scene {
    fn new(materials: Vec<Material>, accelerator: BVH) -> Scene {
        Scene { materials: materials, world: accelerator }
    }
}

pub fn three_spheres_scene(width: usize, height: usize) -> (String, Camera, Scene, Option<Plane>) {
    let origin = Vec3::new(0.0, 3.0, 6.0);
    let lookat = Vec3::new(0.0, 0.0, -1.5);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0;
    let aspect_ratio = (width / height) as f32;
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();
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
    let mut materials: Vec<Material> = Vec::new();

    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));


    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.6, 0.0, -1.0),
                          Vec3::new(0.6, 0.0, -1.0),
                          0.5,
                          red_idx,
                          0.0,
                          1.0)));

    let metal_idx = mat!(materials, Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.1));

    world.add(Geometry::Sphere(Sphere::new(Vec3::new(-0.6, 0.0, -1.0),
                          Vec3::new(-0.6, 0.0, -1.0),
                          0.5,
                          metal_idx,
                          0.0,
                          1.0)));

    let glass_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));

    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.0, 0.1, -2.0),
                          Vec3::new(0.0, 0.1, -2.0),
                          0.5,
                          glass_idx,
                          0.0,
                          1.0)));

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));
    world.add(Geometry::Sphere(Sphere::new(Vec3::new(0.0, -100.5, -1.0),
                          Vec3::new(0.0, -100.5, -1.0),
                          100.0,
                          floor_idx,
                          0.0,
                          1.0)));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = None;

    let scene = Scene::new(materials, bvh);

    (String::from("Three Spheres"), camera, scene, light)
}

pub fn random_spheres_scene(width: usize, height: usize) -> (String, Camera, Scene, Option<Plane>) {
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
    let mut materials: Vec<Material> = Vec::new();

    let floor_idx = mat!(materials, Diffuse::new(SolidColor::new(0.5, 0.5, 0.5).into(), 0.0));

    world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
                          Vec3::new(0.0, -1000.0, 0.0),
                          1000.0,
                          floor_idx,
                          0.0,
                          1.0).into());

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center: Vec3 = Vec3::new(a as f32 + 0.9 * rand::random::<f32>(),
                                         0.2,
                                         b as f32 + 0.9 * rand::random::<f32>());

            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if material < 0.75 {
                    let material = SolidColor::new(rand::random::<f32>() * rand::random::<f32>(),
                                                   rand::random::<f32>() * rand::random::<f32>(),
                                                   rand::random::<f32>() * rand::random::<f32>());
                    // let roughness = rand::distributions::Uniform::new(0.0, 1.0);
                    let random_idx = mat!(materials, Diffuse::new(material.into(), 0.0));
                    world.add(Sphere::new(center, center, 0.2, random_idx, 0.0, 1.0).into());
                } else if material < 0.95 {
                    let material = Reflective::new(Vec3::new(0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>()),
                                                                    0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>()),
                                                                    0.5
                                                                    * (1.0
                                                                       * rand::random::<f32>())),
                                                          0.5 * rand::random::<f32>());
                    let random_idx = mat!(materials, material);
                    world.add(Sphere::new(center,
                                          center,
                                          0.2,
                                          random_idx,
                                          0.0,
                                          1.0).into());
                } else {
                    let refl_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));
                    world.add(Sphere::new(center, center, 0.2, refl_idx, 0.0, 1.0).into());
                    let refr_idx = mat!(materials, Refractive::new(1.5, Vec3::ONE));
                    world.add(Sphere::new(center, center, -0.19,refr_idx, 0.0, 1.0).into());
                }
            }
        }
    }
    let red_idx = mat!(materials, Diffuse::new(SolidColor::new(0.75, 0.25, 0.25).into(), 0.0));
    world.add(Sphere::new(Vec3::new(-2.0, 1.0, 0.0),
                          Vec3::new(-2.0, 1.0, 0.0),
                          1.0,
                          red_idx,
                          0.0,
                          1.0).into());

    let refr_idx1 = mat!(materials, Refractive::new(1.5, Vec3::ONE));
    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          1.0,
                          refr_idx1,
                          0.0,
                          1.0).into());

    world.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0),
                          Vec3::new(0.0, 1.0, 0.0),
                          -0.99,
                          refr_idx1,
                          0.0,
                          1.0).into());

    let refl_idx = mat!(materials, Reflective::new(Vec3::new(0.5, 0.5, 0.5), 0.05));
    world.add(Sphere::new(Vec3::new(2.0, 1.0, 0.0),
                          Vec3::new(2.0, 1.0, 0.0),
                          1.0,
                          refl_idx,
                          0.0,
                          1.0).into());

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);
    let scene = Scene::new(materials, bvh);

    let light = None;

    (String::from("Random Spheres"), camera, scene, light)
}

// pub fn motion_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
//     let origin = Vec3::new(13.0, 2.0, 3.0);
//     let lookat = Vec3::new(0.0, 0.0, 0.0);
//     let view = Vec3::new(0.0, 1.0, 0.0);
//     let fov = 20.0;
//     let aspect_ratio = (width / height) as f32;
//     let aperture = 0.1;
//     let focus_distance = 10.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//     let atmosphere = true;

//     let camera = Camera::new(origin,
//                              lookat,
//                              view,
//                              fov,
//                              aspect_ratio,
//                              aperture,
//                              focus_distance,
//                              time0,
//                              time1,
//                              atmosphere);

//     let mut world = World::new();

//     world.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0),
//                           Vec3::new(0.0, -1000.0, 0.0),
//                           1000.0,
//                           Diffuse::new(SolidColor::new(0.5, 0.5, 0.5), 0.0),
//                           0.0,
//                           1.0));

//     let center: Vec3 = Vec3::new(0.9 * rand::random::<f32>(),
//                                  0.2,
//                                  0.9 * rand::random::<f32>());

//     world.add(Sphere::new(center,
//                           center + Vec3::new(0.0, 0.5 * rand::random::<f32>(), 0.0),
//                           0.2,
//                           Diffuse::new(SolidColor::new(rand::random::<f32>()
//                                                             * rand::random::<f32>(),
//                                                             rand::random::<f32>()
//                                                             * rand::random::<f32>(),
//                                                             rand::random::<f32>()
//                                                             * rand::random::<f32>()),
//                                        0.0),
//                           0.0,
//                           1.0));

//     world.add(Sphere::new(Vec3::new(-2.0, 1.0, 0.0),
//                           Vec3::new(-2.0, 1.0, 0.0),
//                           1.0,
//                           Diffuse::new(SolidColor::new(0.75, 0.25, 0.25), 0.0),
//                           0.0,
//                           1.0));

//     let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

//     let light = Plane::new(Axis::XY, 0.0, 0.0, 0.0, 0.0, 0.0, Empty::new());

//     (String::from("Motion Blur"), camera, bvh, light)
// }

// pub fn cornell_box_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
//     let origin = Vec3::new(278.0, 278.0, -800.0);
//     let lookat = Vec3::new(278.0, 278.0, 0.0);
//     let view = Vec3::new(0.0, 1.0, 0.0);
//     let fov = 40.0;
//     let aspect_ratio = (width / height) as f32;
//     let aperture = 0.0;
//     let focus_distance = 10.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//     let atmosphere = false;

//     let camera = Camera::new(origin,
//                              lookat,
//                              view,
//                              fov,
//                              aspect_ratio,
//                              aperture,
//                              focus_distance,
//                              time0,
//                              time1,
//                              atmosphere);

//     let mut world = World::new();

//     let roughness = 0.0;
//     let red = Diffuse::new(SolidColor::new(0.65, 0.05, 0.05), roughness);
//     let green = Diffuse::new(SolidColor::new(0.12, 0.45, 0.15), roughness);
//     let white = Diffuse::new(SolidColor::new(0.73, 0.73, 0.73), roughness);
//     let light = Light::new(SolidColor::new(35.0, 20.2, 5.6));

//     // add the walls of the cornell box to the world
//     world.add(FlipNormals::of(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, red)));

//     world.add(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green));

//     world.add(FlipNormals::of(Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light)));

//     world.add(FlipNormals::of(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

//     world.add(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));

//     world.add(FlipNormals::of(Plane::new(Axis::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

//     // add the boxes of the cornell box to the world
//     let p0 = Vec3::new(0.0, 0.0, 0.0);
//     let p1 = Vec3::new(165.0, 165.0, 165.0);

//     world.add(Translate::new(Vec3::new(130.0, 0.0, 65.0),
//                              Rotate::new(0.0, -18.0, 0.0, Rectangle::new(p0, p1, Arc::new(white.clone())))));

//     let p2 = Vec3::new(165.0, 330.0, 165.0);

//     world.add(Translate::new(Vec3::new(265.0, 0.0, 295.0),
//                              Rotate::new(0.0, 15.0, 0.0, Rectangle::new(p0, p2, Arc::new(white.clone())))));

//     let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

//     let light = Light::new(SolidColor::new(0.0, 0.0, 0.0));
//     let light_shape = Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light);

//     (String::from("Cornell Box"), camera, bvh, light_shape)
// }

// pub fn spheres_in_box_scene(width: usize, height: usize) -> (String, Camera, BVH, Plane) {
//     let origin = Vec3::new(478.0, 278.0, -600.0);
//     let lookat = Vec3::new(278.0, 278.0, 0.0);
//     let view = Vec3::new(0.0, 1.0, 0.0);
//     let fov = 40.0;
//     let aspect_ratio = (width / height) as f32;
//     let aperture = 0.0;
//     let focus_distance = 10.0;
//     let time0 = 0.0;
//     let time1 = 1.0;
//     let atmosphere = false;

//     let camera = Camera::new(origin,
//                              lookat,
//                              view,
//                              fov,
//                              aspect_ratio,
//                              aperture,
//                              focus_distance,
//                              time0,
//                              time1,
//                              atmosphere);

//     let mut world = World::new();

//     let white = Diffuse::new(SolidColor::new(0.73, 0.73, 0.73), 0.0);
//     let orange = Diffuse::new(SolidColor::new(1.0, 0.10, 0.0), 0.0);
//     let light = Light::new(SolidColor::new(7.0, 7.0, 7.0));
//     let ground = Diffuse::new(SolidColor::new(0.48, 0.83, 0.53), 0.0);

//     let number_of_boxes = 20;

//     for i in 0..number_of_boxes {
//         for j in 0..number_of_boxes {
//             let w = 100.0;
//             let p0 = Vec3::new(-1000.0 + i as f32 * w, 0.0, -1000.0 + j as f32 * w);
//             let p1 = p0 + Vec3::new(w, 100.0 * (rand::random::<f32>() + 0.01), w);
//             world.add(Rectangle::new(p0, p1, Arc::new(ground.clone())));
//         }
//     }

//     world.add(Plane::new(Axis::XZ, 123.0, 423.0, 147.0, 412.0, 554.0, light));

//     world.add(Sphere::new(Vec3::new(400.0, 400.0, 200.0),
//                           Vec3::new(430.0, 400.0, 200.0),
//                           50.0,
//                           orange,
//                           0.0,
//                           1.0));

//     world.add(Sphere::new(Vec3::new(260.0, 150.0, 45.0),
//                           Vec3::new(260.0, 150.0, 45.0),
//                           50.0,
//                           Refractive::new(1.5, Vec3::ONE),
//                           0.0,
//                           1.0));

//     world.add(Sphere::new(Vec3::new(0.0, 150.0, 145.0),
//                           Vec3::new(0.0, 150.0, 145.0),
//                           50.0,
//                           Reflective::new(Vec3::new(0.8, 0.8, 0.9), 1.0),
//                           0.0,
//                           1.0));

//     let boundary = Sphere::new(Vec3::new(360.0, 150.0, 145.0),
//                                Vec3::new(360.0, 150.0, 145.0),
//                                70.0,
//                                Refractive::new(1.5, Vec3::ONE),
//                                0.0,
//                                1.0);

//     world.add(boundary.clone());

//     world.add(Volume::new(0.2, boundary.clone(), SolidColor::new(0.2, 0.4, 0.9)));

//     let fog = Sphere::new(Vec3::new(0.0, 0.0, 0.0),
//                           Vec3::new(0.0, 0.0, 0.0),
//                           5000.0,
//                           Refractive::new(1.5, Vec3::ONE),
//                           0.0,
//                           1.0);

//     world.add(Volume::new(0.0001, fog, SolidColor::new(1.0, 1.0, 1.0)));

//     world.add(Sphere::new(Vec3::new(400.0, 200.0, 400.0),
//                           Vec3::new(400.0, 200.0, 400.0),
//                           100.0,
//                           Diffuse::new(ImageTexture::new("world_topo_nasa.jpg"), 0.0),
//                           0.0,
//                           1.0));

//     world.add(Sphere::new(Vec3::new(220.0, 280.0, 300.0),
//                           Vec3::new(220.0, 280.0, 300.0),
//                           80.0,
//                           Diffuse::new(SolidColor::new(0.6, 0.6, 0.6), 0.0),
//                           0.0,
//                           1.0));

//     let number_of_spheres = 1000;
//     for _ in 0..number_of_spheres {
//         let center = Vec3::new(165.0 * rand::random::<f32>(),
//                                165.0 * rand::random::<f32>(),
//                                165.0 * rand::random::<f32>());

//         let sphere = Sphere::new(center, center, 10.0, white.clone(), 0.0, 1.0);

//         world.add(Translate::new(Vec3::new(-100.0, 270.0, 395.0), Rotate::new(0.0, 15.0, 0.0, sphere)));
//     }

//     let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

//     let light = Light::new(SolidColor::new(0.0, 0.0, 0.0));
//     let light_shape = Plane::new(Axis::XZ, 123.0, 423.0, 147.0, 412.0, 554.0, light);

//     (String::from("Spheres in Box"), camera, bvh, light_shape)
// }

pub fn cornell_box_object_scene(width: usize, height: usize)
                              -> (String, Camera, Scene, Option<Plane>) {
    let origin = Vec3::new(278.0, 278.0, -800.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let fov = 40.0;
    let aspect_ratio = (width / height) as f32;

    let camera = Camera::new(origin, lookat, view, fov, aspect_ratio,
                             0.0, 10.0, 0.0, 1.0, false);

    let mut world = World::new();
    let mut materials: Vec<Material> = Vec::new();

    let roughness = 0.0;
    let red_id = mat!(materials, Diffuse::new(SolidColor::new(0.65, 0.05, 0.05).into(), roughness));
    let green_id = mat!(materials, Diffuse::new(SolidColor::new(0.12, 0.45, 0.15).into(), roughness));
    let white_id = mat!(materials, Diffuse::new(SolidColor::new(0.73, 0.73, 0.73).into(), roughness));
    let light_material = mat!(materials, Light::new(SolidColor::new(25.0, 18.0, 10.0).into()));

    // Cornell walls
    world.add(Geometry::ReverseOrientation(Box::new(Geometry::Plane(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 555.0, red_id)))));
    world.add(Geometry::Plane(Plane::new(Axis::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, green_id)));
    world.add(Geometry::ReverseOrientation(Box::new(Geometry::Plane(Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light_material)))));
    world.add(Geometry::ReverseOrientation(Box::new(Geometry::Plane(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 555.0, white_id)))));
    world.add(Geometry::Plane(Plane::new(Axis::XZ, 0.0, 555.0, 0.0, 555.0, 0.0, white_id)));
    world.add(Geometry::ReverseOrientation(Box::new(Geometry::Plane(Plane::new(Axis::XY, 0.0, 555.0, 0.0, 555.0, 555.0, white_id)))));

    let lucy_material = mat!(materials, Diffuse::new(SolidColor::new(0.92, 0.88, 0.82).into(), 0.05));
    let lucy = Geometry::TriangleMesh(Box::new(TriangleMesh::from("models/lucy.obj", lucy_material)));
    world.add(Geometry::TransformedMesh(Box::new(TransformedMesh::new(Vec3::new(200.0, 180.0, 364.0), Vec3::new(0.0, 0.0, 0.0), 0.30, lucy))));

    let dragon_material = mat!(materials, Plastic::new(SolidColor::new(0.7, 0.85, 0.45).into(), 0.3, 1.5));
    let dragon = Geometry::TriangleMesh(Box::new(TriangleMesh::from("models/dragon.obj", dragon_material)));
    world.add(Geometry::TransformedMesh(Box::new(TransformedMesh::new(Vec3::new(283.0, 96.0, 268.0), Vec3::new(0.0, -60.0, 0.0), 350.0, dragon))));

    let bunny_material = mat!(materials, Refractive::new(1.5, Vec3::ONE));
    let bunny = Geometry::TriangleMesh(Box::new(TriangleMesh::from("models/bunny.obj", bunny_material)));
    world.add(Geometry::TransformedMesh(Box::new(TransformedMesh::new(Vec3::new(110.0, -25.0, 140.0), Vec3::new(0.0, 180.0, 0.0), 750.0, bunny))));

    let buddha_texture = ImageTexture::new("models/buddha_relief_diffuse.jpeg").into();
    let buddha_material = mat!(materials, Diffuse::new(buddha_texture, 0.0));
    let buddha = Geometry::TriangleMesh(Box::new(TriangleMesh::from("models/buddha_relief.obj", buddha_material)));
    world.add(Geometry::TransformedMesh(Box::new(TransformedMesh::new(Vec3::new(273.0, 180.0, 582.0), Vec3::new(-90.0, 180.0, 0.0), 24.0, buddha))));

    let bvh = BVH::new(&mut world.objects, 0.0, 1.0);

    let light = mat!(materials, Light::new(SolidColor::new(0.0, 0.0, 0.0).into()));
    let light_shape = Some(Plane::new(Axis::XZ, 213.0, 343.0, 227.0, 332.0, 554.0, light));

    let scene = Scene::new(materials, bvh);

    (String::from("Cornell Box with Multiple Objects"), camera, scene, light_shape)
}