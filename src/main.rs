extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rayon;

mod camera;
mod hitable;
mod materials;
mod ray;
mod sphere;
mod world;

use std::env;
use std::f64;

use nalgebra::core::Vector3;
use rand::thread_rng;
use rayon::prelude::*;

use camera::Camera;
use materials::{Refractive, Diffuse, Reflective};
use sphere::Sphere;
use world::World;


fn main() {
    let (width, height): (u32, u32) = (1920, 960);
    let args: Vec<String> = env::args().collect();
    let samples: u32 = args[1].parse().unwrap();

    let origin = Vector3::new(0.0, 3.0, 6.0);
    let lookat = Vector3::new(0.0, 0.0, -1.5);
    let camera = Camera::new(origin,
                             lookat,
                             Vector3::new(0.0, 1.0, 0.0),
                             20.0,
                             (width / height) as f64,
                             0.01,
                             10.0);

    let mut world = World::new();

    world.add(Sphere::new(
        Vector3::new(0.6, 0.0, -1.0),
        0.5,
        Diffuse::new(Vector3::new(0.75, 0.25, 0.25))
    ));

    world.add(Sphere::new(
        Vector3::new(-0.6, 0.0, -1.0),
        0.5,
        Reflective::new(Vector3::new(0.5, 0.5, 0.5), 0.1),
    ));

    world.add(Sphere::new(
        Vector3::new(0.0, 0.1, -2.0),
        0.5,
        Refractive::new(Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0))
    );

    world.add(Sphere::new(
        Vector3::new(0.0, 0.1, -2.0),
        -0.49,
        Refractive::new(Vector3::new(0.9, 0.9, 0.9), 1.5, 0.0))
    );

    world.add(Sphere::new(
        Vector3::new(0.0, -100.5, -1.0),
        100.0,
        Diffuse::new(Vector3::new(0.5, 0.5, 0.5)))
    );

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let mut coordinate: Vector3<f64> = Vector3::zeros();
        let x = i % width as usize;
        let y = i / width as usize;

        let mut rng = thread_rng();

        (0..samples).for_each(|_| {
            let u = (x as f64 + rand::random::<f64>()) / width as f64;
            let v = (y as f64 + rand::random::<f64>()) / height as f64;
            let ray = camera.get_ray(u, v);
            coordinate += ray::compute_color(&ray, &world, 0, &mut rng);
        });

        coordinate /= samples as f64;
        (0..3).for_each(|i| coordinate[i] = 255.0 * coordinate[i].sqrt());
        *pixel = image::Rgb([coordinate.x as u8, coordinate.y as u8, coordinate.z as u8]);
    });


    let mut buffer = image::ImageBuffer::new(width, height);

    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let index = (y * width + x) as usize;
        *pixel = pixels[index];
    });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();
}
